//! Naming conventions for heal remediation jobs.
//!
//! Heal remediation jobs follow the pattern:
//! `heal-remediation-task{task_id}-{alert_type}-{alert_id}`
//!
//! Example: `heal-remediation-task42-a7-0dc49936`

const MAX_K8S_NAME_LENGTH: usize = 63;
const HEAL_REMEDIATION_PREFIX: &str = "heal-remediation-";

/// Naming utilities for heal remediation resources.
pub struct HealNaming;

impl HealNaming {
    /// Generate a heal remediation job name.
    ///
    /// Format: `heal-remediation-task{task_id}-{alert_type}-{alert_id}`
    ///
    /// # Arguments
    /// * `task_id` - The task ID (numeric or string)
    /// * `alert_type` - The alert type (e.g., "a7", "a2")
    /// * `alert_id` - Unique alert identifier (typically first 8 chars of UUID)
    ///
    /// # Returns
    /// A K8s-compliant name, truncated if necessary.
    #[must_use]
    pub fn remediation_job_name(task_id: &str, alert_type: &str, alert_id: &str) -> String {
        let base_name = format!("{HEAL_REMEDIATION_PREFIX}task{task_id}-{alert_type}-{alert_id}");
        Self::ensure_k8s_name_length(&base_name)
    }

    /// Generate a worktree path for a heal remediation CodeRun.
    ///
    /// Format: `/workspace/worktrees/{coderun_name}`
    #[must_use]
    pub fn worktree_path(coderun_name: &str) -> String {
        format!("/workspace/worktrees/{coderun_name}")
    }

    /// Generate the log file path for a heal alert.
    ///
    /// Format: `/workspace/watch/logs/{alert_type}-{pod_name}-{timestamp}.log`
    #[must_use]
    pub fn log_file_path(alert_type: &str, pod_name: &str, timestamp: &str) -> String {
        format!(
            "/workspace/watch/logs/{}-{}-{}.log",
            alert_type.to_uppercase(),
            pod_name,
            timestamp
        )
    }

    /// Generate the prompt file path for a heal alert.
    ///
    /// Format: `/workspace/watch/alerts/{alert_id}.md`
    #[must_use]
    pub fn prompt_file_path(alert_id: &str) -> String {
        format!("/workspace/watch/alerts/{alert_id}.md")
    }

    /// Extract alert type from a heal remediation job name.
    ///
    /// Example: `heal-remediation-task42-a7-0dc49936` -> `Some("a7")`
    ///
    /// Note: This handles the case where alert_id may contain hyphens (e.g., UUIDs).
    /// The alert_type is always immediately after the task part and starts with 'a'.
    #[must_use]
    pub fn extract_alert_type(job_name: &str) -> Option<String> {
        // Format: heal-remediation-task{id}-{alert_type}-{alert_id}
        // Parts: ["heal", "remediation", "task42", "a7", "0dc49936"] or with hyphenated alert_id
        let parts: Vec<&str> = job_name.split('-').collect();

        if parts.len() >= 4 && parts[0] == "heal" && parts[1] == "remediation" {
            // The task part is at index 2 (e.g., "task42")
            // The alert_type is at index 3 and always starts with 'a' followed by digits
            let candidate = parts[3];
            if candidate.starts_with('a') && candidate.len() > 1 {
                let rest = &candidate[1..];
                if rest.chars().all(|c| c.is_ascii_digit()) {
                    return Some(candidate.to_string());
                }
            }
        }
        None
    }

    /// Extract task ID from a heal remediation job name.
    ///
    /// Example: `heal-remediation-task42-a7-0dc49936` -> `Some("42")`
    #[must_use]
    pub fn extract_task_id(job_name: &str) -> Option<String> {
        let parts: Vec<&str> = job_name.split('-').collect();
        if parts.len() >= 3 && parts[0] == "heal" && parts[1] == "remediation" {
            // task part is "task42" - extract the number
            let task_part = parts[2];
            if let Some(id) = task_part.strip_prefix("task") {
                return Some(id.to_string());
            }
        }
        None
    }

    /// Ensure name is within K8s limits (63 chars).
    ///
    /// Uses smart truncation to preserve important identifiers:
    /// - Always preserves the prefix "heal-remediation-"
    /// - Always preserves the alert_id suffix (last 8 chars after final hyphen)
    /// - Truncates the task_id in the middle if needed
    fn ensure_k8s_name_length(name: &str) -> String {
        if name.len() <= MAX_K8S_NAME_LENGTH {
            return name.to_string();
        }

        // Format: heal-remediation-task{task_id}-{alert_type}-{alert_id}
        // We want to preserve: prefix, alert_type, and alert_id
        let parts: Vec<&str> = name.split('-').collect();

        if parts.len() >= 5 {
            let prefix = format!("{}-{}", parts[0], parts[1]); // "heal-remediation"
            let alert_type = parts[parts.len() - 2]; // e.g., "a7"
            let alert_id = parts[parts.len() - 1]; // e.g., "0dc49936"

            // Calculate how much space we have for task part
            // Format: {prefix}-task{truncated_task}-{alert_type}-{alert_id}
            let suffix = format!("-{alert_type}-{alert_id}");
            let task_prefix = "task";
            let available_for_task = MAX_K8S_NAME_LENGTH
                .saturating_sub(prefix.len())
                .saturating_sub(1) // hyphen before task
                .saturating_sub(task_prefix.len())
                .saturating_sub(suffix.len());

            // Extract task_id from the task part (parts[2] is "task{id}")
            let task_part = parts[2];
            let task_id = task_part.strip_prefix("task").unwrap_or(task_part);

            let truncated_task: String = task_id.chars().take(available_for_task).collect();

            let result = format!("{prefix}-{task_prefix}{truncated_task}{suffix}");

            // Final safety check
            if result.len() <= MAX_K8S_NAME_LENGTH {
                return result;
            }
        }

        // Fallback: simple truncation ensuring no trailing hyphen
        let mut truncated: String = name.chars().take(MAX_K8S_NAME_LENGTH).collect();
        while truncated.ends_with('-') {
            truncated.pop();
        }
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remediation_job_name() {
        let name = HealNaming::remediation_job_name("42", "a7", "0dc49936");
        assert_eq!(name, "heal-remediation-task42-a7-0dc49936");
        assert!(name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn test_remediation_job_name_long_task_id() {
        let name = HealNaming::remediation_job_name("4010126410", "a7", "0dc49936");
        assert_eq!(name, "heal-remediation-task4010126410-a7-0dc49936");
        assert!(name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn test_worktree_path() {
        let path = HealNaming::worktree_path("heal-remediation-task42-a7-0dc49936");
        assert_eq!(
            path,
            "/workspace/worktrees/heal-remediation-task42-a7-0dc49936"
        );
    }

    #[test]
    fn test_log_file_path() {
        let path = HealNaming::log_file_path("a7", "my-pod-abc123", "20251201-134523");
        assert_eq!(
            path,
            "/workspace/watch/logs/A7-my-pod-abc123-20251201-134523.log"
        );
    }

    #[test]
    fn test_prompt_file_path() {
        let path = HealNaming::prompt_file_path("0dc49936");
        assert_eq!(path, "/workspace/watch/alerts/0dc49936.md");
    }

    #[test]
    fn test_extract_alert_type() {
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task42-a7-0dc49936"),
            Some("a7".to_string())
        );
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task100-a2-abcd1234"),
            Some("a2".to_string())
        );
        assert_eq!(HealNaming::extract_alert_type("some-other-job"), None);
    }

    #[test]
    fn test_extract_task_id() {
        assert_eq!(
            HealNaming::extract_task_id("heal-remediation-task42-a7-0dc49936"),
            Some("42".to_string())
        );
        assert_eq!(
            HealNaming::extract_task_id("heal-remediation-task4010126410-a7-0dc49936"),
            Some("4010126410".to_string())
        );
        assert_eq!(HealNaming::extract_task_id("some-other-job"), None);
    }

    #[test]
    fn test_smart_truncation_preserves_alert_id() {
        // Very long task ID that would exceed 63 chars
        let name = HealNaming::remediation_job_name(
            "99999999999999999999999999999999", // 32 digit task_id
            "a7",
            "abcd1234",
        );

        // Should be within limits
        assert!(name.len() <= MAX_K8S_NAME_LENGTH);

        // Should preserve the alert_id suffix
        assert!(
            name.ends_with("-a7-abcd1234"),
            "Name should preserve alert_type and alert_id: {name}"
        );

        // Should preserve the prefix
        assert!(
            name.starts_with("heal-remediation-task"),
            "Name should preserve prefix: {name}"
        );
    }

    #[test]
    fn test_extract_alert_type_validates_format() {
        // Valid alert types
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task42-a7-0dc49936"),
            Some("a7".to_string())
        );
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task42-a12-0dc49936"),
            Some("a12".to_string())
        );

        // Invalid - doesn't start with 'a'
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task42-xyz-0dc49936"),
            None
        );

        // Invalid - 'a' not followed by digits
        assert_eq!(
            HealNaming::extract_alert_type("heal-remediation-task42-abc-0dc49936"),
            None
        );
    }
}
