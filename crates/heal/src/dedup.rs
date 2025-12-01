//! Deduplication module for heal remediation
//!
//! Prevents duplicate remediation `CodeRuns` and GitHub issues for the same alert/pod.

use anyhow::{Context, Result};
use std::process::Command;

/// Label used to exclude pods from heal monitoring
pub const EXCLUDE_LABEL: &str = "heal.platform/exclude";

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
