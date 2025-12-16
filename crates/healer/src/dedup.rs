//! Deduplication module for healer remediation
//!
//! Prevents duplicate remediation `CodeRuns` and GitHub issues for the same alert/pod.
//! Also provides alert-type level deduplication to prevent issue spam when multiple
//! pods fail with the same root cause.
//!
//! Deduplication is scoped by "workflow family" - pods from the same workflow (e.g.,
//! `play-task-4-*`) are grouped together, but pods from different workflows (e.g.,
//! `atlas-*` vs `play-*`) are treated separately.

use anyhow::{Context, Result};
use std::process::Command;

/// Label used to exclude pods from healer monitoring
pub const EXCLUDE_LABEL: &str = "healer.platform/exclude";

/// Time window (in minutes) for grouping similar alerts into one issue
const DEDUP_WINDOW_MINS: u64 = 30;

/// Extract the workflow family from a pod name.
///
/// Examples:
/// - `play-task-4-abc-step-123` -> `play-task-4`
/// - `atlas-conflict-monitor-xyz` -> `atlas-conflict-monitor`
/// - `healer-remediation-task1-a7-abc` -> `healer-remediation`
/// - `cto-tools-67db5dff7-hn8xh` -> `cto-tools`
///
/// The workflow family is used to group related pod failures together.
#[must_use]
pub fn extract_workflow_family(pod_name: &str) -> String {
    let parts: Vec<&str> = pod_name.split('-').collect();

    // Special cases for known workflow patterns
    if pod_name.starts_with("play-task-") && parts.len() >= 3 {
        // play-task-N-* -> play-task-N
        return format!("{}-{}-{}", parts[0], parts[1], parts[2]);
    }
    if pod_name.starts_with("healer-remediation-") && parts.len() >= 2 {
        // healer-remediation-* -> healer-remediation
        return format!("{}-{}", parts[0], parts[1]);
    }
    if pod_name.starts_with("atlas-") && parts.len() >= 2 {
        // atlas-conflict-monitor-* -> atlas-conflict-monitor
        // atlas-batch-integration-* -> atlas-batch-integration
        // atlas-guardian-* -> atlas-guardian
        if parts.len() >= 3 && (parts[1] == "conflict" || parts[1] == "batch") {
            return format!("{}-{}-{}", parts[0], parts[1], parts[2]);
        }
        return format!("{}-{}", parts[0], parts[1]);
    }

    // Default: take first 2 segments (handles cto-tools, cto-controller, etc.)
    if parts.len() >= 2 {
        format!("{}-{}", parts[0], parts[1])
    } else {
        pod_name.to_string()
    }
}

/// Check if a pod should be excluded from healer monitoring
pub fn should_exclude_pod(labels: &std::collections::HashMap<String, String>) -> bool {
    labels.get(EXCLUDE_LABEL).is_some_and(|v| v == "true")
}

/// Check if there's already an active remediation `CodeRun` for this alert+pod combination.
///
/// Returns `Some(coderun_name)` if a running/pending remediation exists, `None` otherwise.
pub fn check_existing_remediation(
    alert_type: &str,
    pod_name: &str,
    namespace: &str,
) -> Result<Option<String>> {
    // Query for CodeRuns with matching labels that are still active
    let label_selector = format!("alert-type={alert_type},target-pod={pod_name},remediation=true");

    let output = Command::new("kubectl")
        .args([
            "get",
            "coderuns",
            "-n",
            namespace,
            "-l",
            &label_selector,
            "-o",
            "jsonpath={.items[*].metadata.name}",
        ])
        .output()
        .context("Failed to query existing CodeRuns")?;

    if !output.status.success() {
        // If kubectl fails (e.g., CRD not installed), allow proceeding
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("NotFound") || stderr.contains("the server doesn't have") {
            return Ok(None);
        }
        // Log but don't fail - allow remediation to proceed
        eprintln!("⚠️  Warning: Failed to check existing CodeRuns: {stderr}");
        return Ok(None);
    }

    let names = String::from_utf8_lossy(&output.stdout);
    let names = names.trim();

    if names.is_empty() {
        return Ok(None);
    }

    // Found existing CodeRun(s) - return the first one
    let first_name = names.split_whitespace().next().unwrap_or("");
    if first_name.is_empty() {
        return Ok(None);
    }

    // Check if it's still running (not completed/failed)
    let phase_output = Command::new("kubectl")
        .args([
            "get",
            "coderun",
            first_name,
            "-n",
            namespace,
            "-o",
            "jsonpath={.status.phase}",
        ])
        .output()
        .context("Failed to get CodeRun phase")?;

    let phase = String::from_utf8_lossy(&phase_output.stdout);
    let phase = phase.trim();

    // Only block if the CodeRun is still active
    match phase {
        "Pending" | "Running" | "" => Ok(Some(first_name.to_string())),
        _ => Ok(None), // Completed, Failed, etc. - allow new remediation
    }
}

/// Check if there's an open GitHub issue for this alert+pod combination.
///
/// Returns `Some(issue_number)` if an open issue exists, `None` otherwise.
#[allow(dead_code)] // Will be used in Phase 2 of deduplication
pub fn check_existing_github_issue(
    alert_type: &str,
    pod_name: &str,
    repo: &str,
) -> Result<Option<u64>> {
    // Query GitHub for open issues with matching labels
    let search_query = format!("[HEAL-{}] {} in:title", alert_type.to_uppercase(), pod_name);

    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--label",
            &format!("heal,{alert_type}"),
            "--search",
            &search_query,
            "--json",
            "number",
            "--jq",
            ".[0].number",
        ])
        .output()
        .context("Failed to query GitHub issues")?;

    if !output.status.success() {
        // gh CLI not available or auth issue - allow proceeding
        return Ok(None);
    }

    let number_str = String::from_utf8_lossy(&output.stdout);
    let number_str = number_str.trim();

    if number_str.is_empty() || number_str == "null" {
        return Ok(None);
    }

    Ok(number_str.parse::<u64>().ok())
}

/// Check if there's a recent open GitHub issue for this alert TYPE within the same workflow family.
///
/// This prevents issue spam when multiple pods from the SAME workflow fail with the same
/// root cause, while still allowing separate issues for unrelated workflows.
///
/// For example:
/// - Two `play-task-4-*` pods failing with A2 will share one issue
/// - An `atlas-*` pod and a `play-*` pod failing with A2 will get separate issues
///
/// Returns `Some((issue_number, title))` if a recent open issue exists for the same family.
pub fn check_recent_alert_type_issue(
    alert_type: &str,
    pod_name: &str,
    repo: &str,
) -> Result<Option<(u64, String)>> {
    // Extract workflow family for filtering
    let current_family = extract_workflow_family(pod_name);

    // Query for any open issues with this alert type created recently
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--label",
            &format!("heal,{alert_type}"),
            "--json",
            "number,title,createdAt",
            "--limit",
            "10",
        ])
        .output()
        .context("Failed to query GitHub issues")?;

    if !output.status.success() {
        return Ok(None);
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let json_str = json_str.trim();

    if json_str.is_empty() || json_str == "[]" {
        return Ok(None);
    }

    // Parse the JSON response
    let issues: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    // Find the most recent issue created within the dedup window for the same workflow family
    let now = chrono::Utc::now();
    for issue in issues {
        let number = issue["number"].as_u64();
        let title = issue["title"].as_str();
        let created_at = issue["createdAt"].as_str();

        if let (Some(num), Some(t), Some(created)) = (number, title, created_at) {
            // Parse the ISO timestamp
            if let Ok(created_time) = chrono::DateTime::parse_from_rfc3339(created) {
                let age_mins = (now - created_time.with_timezone(&chrono::Utc))
                    .num_minutes()
                    .unsigned_abs();

                // Only dedup if within time window AND same workflow family
                if age_mins <= DEDUP_WINDOW_MINS {
                    // Extract pod name from title: "[HEAL-A2] Silent Failure: pod-name-here"
                    // and check if it's from the same workflow family
                    if let Some(issue_pod) = extract_pod_from_title(t) {
                        let issue_family = extract_workflow_family(&issue_pod);
                        if issue_family == current_family {
                            return Ok(Some((num, t.to_string())));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Extract pod name from a heal issue title.
///
/// Titles follow the pattern: `[HEAL-A2] Silent Failure: pod-name-here`
#[must_use]
pub fn extract_pod_from_title(title: &str) -> Option<String> {
    // Look for ": " followed by the pod name
    if let Some(idx) = title.rfind(": ") {
        let pod_part = &title[idx + 2..];
        // Take until first space or end
        let pod_name = pod_part.split_whitespace().next()?;
        if !pod_name.is_empty() {
            return Some(pod_name.to_string());
        }
    }
    None
}

/// Check if there's an active remediation for this alert TYPE (any pod).
///
/// This prevents spawning multiple remediations for the same systemic failure.
/// Returns `Some(coderun_name)` if an active remediation exists.
#[allow(dead_code)] // Will be used for CodeRun-level deduplication
pub fn check_alert_type_remediation(alert_type: &str, namespace: &str) -> Result<Option<String>> {
    // Query for any active CodeRuns with this alert type
    let label_selector = format!("alert-type={alert_type},remediation=true");

    let output = Command::new("kubectl")
        .args([
            "get",
            "coderuns",
            "-n",
            namespace,
            "-l",
            &label_selector,
            "-o",
            "jsonpath={range .items[*]}{.metadata.name},{.status.phase}{\"\\n\"}{end}",
        ])
        .output()
        .context("Failed to query CodeRuns")?;

    if !output.status.success() {
        return Ok(None);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Check if any are still active
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let name = parts[0];
            let phase = parts[1];
            if matches!(phase, "Pending" | "Running" | "") && !name.is_empty() {
                return Ok(Some(name.to_string()));
            }
        }
    }

    Ok(None)
}

/// Sanitize a pod name for use as a Kubernetes label value.
/// Label values must be <= 63 chars, alphanumeric with hyphens/underscores/dots.
pub fn sanitize_label_value(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(63)
        .collect();

    // Trim trailing hyphens/dots
    sanitized.trim_end_matches(['-', '.', '_']).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude_pod() {
        let mut labels = std::collections::HashMap::new();
        assert!(!should_exclude_pod(&labels));

        labels.insert("healer.platform/exclude".to_string(), "false".to_string());
        assert!(!should_exclude_pod(&labels));

        labels.insert("healer.platform/exclude".to_string(), "true".to_string());
        assert!(should_exclude_pod(&labels));
    }

    #[test]
    fn test_sanitize_label_value() {
        assert_eq!(sanitize_label_value("simple-pod"), "simple-pod");
        assert_eq!(
            sanitize_label_value(
                "pod-with-very-long-name-that-exceeds-kubernetes-label-limits-definitely"
            ),
            "pod-with-very-long-name-that-exceeds-kubernetes-label-limits-de"
        );
        assert_eq!(sanitize_label_value("pod-name---"), "pod-name");
        assert_eq!(sanitize_label_value("pod@with#special"), "podwithspecial");
    }

    #[test]
    fn test_extract_workflow_family_play_tasks() {
        // Play task pods should group by task number
        assert_eq!(
            extract_workflow_family("play-task-4-abc-step-123"),
            "play-task-4"
        );
        assert_eq!(
            extract_workflow_family("play-task-4-xyz-determine-resume-point-456"),
            "play-task-4"
        );
        assert_eq!(extract_workflow_family("play-task-1-jqc6d"), "play-task-1");
    }

    #[test]
    fn test_extract_workflow_family_atlas() {
        // Atlas workflows should group by type
        assert_eq!(
            extract_workflow_family("atlas-conflict-monitor-xyz"),
            "atlas-conflict-monitor"
        );
        assert_eq!(
            extract_workflow_family("atlas-batch-integration-abc"),
            "atlas-batch-integration"
        );
        assert_eq!(
            extract_workflow_family("atlas-guardian-tcf6d"),
            "atlas-guardian"
        );
    }

    #[test]
    fn test_extract_workflow_family_heal() {
        // Heal remediations should group together
        assert_eq!(
            extract_workflow_family("healer-remediation-task1-a7-abc"),
            "healer-remediation"
        );
        assert_eq!(
            extract_workflow_family("healer-remediation-taskunknown-a2-xyz"),
            "healer-remediation"
        );
    }

    #[test]
    fn test_extract_workflow_family_services() {
        // Service pods should group by service
        assert_eq!(
            extract_workflow_family("cto-tools-67db5dff7-hn8xh"),
            "cto-tools"
        );
        assert_eq!(
            extract_workflow_family("cto-controller-794646b4c7-tg28b"),
            "cto-controller"
        );
    }

    #[test]
    fn test_extract_pod_from_title() {
        assert_eq!(
            extract_pod_from_title("[HEAL-A2] Silent Failure: atlas-batch-integration-nxp5c"),
            Some("atlas-batch-integration-nxp5c".to_string())
        );
        assert_eq!(
            extract_pod_from_title("[HEAL-A7] Pod Failure: play-task-4-abc-step-123"),
            Some("play-task-4-abc-step-123".to_string())
        );
        assert_eq!(extract_pod_from_title("No colon here"), None);
    }

    #[test]
    fn test_workflow_families_different() {
        // These should be different families (not deduped together)
        let play_family = extract_workflow_family("play-task-4-abc-step-123");
        let atlas_family = extract_workflow_family("atlas-conflict-monitor-xyz");
        assert_ne!(play_family, atlas_family);

        // Different task numbers are different families
        let task4 = extract_workflow_family("play-task-4-abc");
        let task6 = extract_workflow_family("play-task-6-xyz");
        assert_ne!(task4, task6);
    }
}
