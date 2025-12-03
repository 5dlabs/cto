//! Deduplication module for heal remediation
//!
//! Prevents duplicate remediation `CodeRuns` and GitHub issues for the same alert/pod.
//! Also provides alert-type level deduplication to prevent issue spam when multiple
//! pods fail with the same root cause.

use anyhow::{Context, Result};
use std::process::Command;

/// Label used to exclude pods from heal monitoring
pub const EXCLUDE_LABEL: &str = "heal.platform/exclude";

/// Time window (in minutes) for grouping similar alerts into one issue
const DEDUP_WINDOW_MINS: u64 = 30;

/// Check if a pod should be excluded from heal monitoring
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

/// Check if there's a recent open GitHub issue for this alert TYPE (regardless of pod).
///
/// This prevents issue spam when multiple pods fail with the same root cause.
/// Returns `Some((issue_number, title))` if a recent open issue exists.
pub fn check_recent_alert_type_issue(
    alert_type: &str,
    repo: &str,
) -> Result<Option<(u64, String)>> {
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

    // Find the most recent issue created within the dedup window
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

                // If issue was created within the dedup window, return it
                if age_mins <= DEDUP_WINDOW_MINS {
                    return Ok(Some((num, t.to_string())));
                }
            }
        }
    }

    Ok(None)
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

        labels.insert("heal.platform/exclude".to_string(), "false".to_string());
        assert!(!should_exclude_pod(&labels));

        labels.insert("heal.platform/exclude".to_string(), "true".to_string());
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
}
