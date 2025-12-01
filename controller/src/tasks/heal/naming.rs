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
        let base_name = format!(
            "{HEAL_REMEDIATION_PREFIX}task{task_id}-{alert_type}-{alert_id}"
        );
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
    #[must_use]
    pub fn extract_alert_type(job_name: &str) -> Option<String> {
        // Format: heal-remediation-task{id}-{alert_type}-{alert_id}
        let parts: Vec<&str> = job_name.split('-').collect();
        // Parts: ["heal", "remediation", "task42", "a7", "0dc49936"]
        if parts.len() >= 4 && parts[0] == "heal" && parts[1] == "remediation" {
            // alert_type is the 4th part (index 3)
            return Some(parts[3].to_string());
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
            if task_part.starts_with("task") {
                return Some(task_part[4..].to_string());
            }
        }
        None
    }

    /// Ensure name is within K8s limits (63 chars).
    fn ensure_k8s_name_length(name: &str) -> String {
        if name.len() <= MAX_K8S_NAME_LENGTH {
            name.to_string()
        } else {
            // Truncate but ensure it doesn't end with a hyphen
            let mut truncated: String = name.chars().take(MAX_K8S_NAME_LENGTH).collect();
            while truncated.ends_with('-') {
                truncated.pop();
            }
            truncated
        }
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
        assert_eq!(path, "/workspace/worktrees/heal-remediation-task42-a7-0dc49936");
    }

    #[test]
    fn test_log_file_path() {
        let path = HealNaming::log_file_path("a7", "my-pod-abc123", "20251201-134523");
        assert_eq!(path, "/workspace/watch/logs/A7-my-pod-abc123-20251201-134523.log");
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
        assert_eq!(
            HealNaming::extract_alert_type("some-other-job"),
            None
        );
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
        assert_eq!(
            HealNaming::extract_task_id("some-other-job"),
            None
        );
    }
}
