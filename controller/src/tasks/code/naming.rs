use crate::crds::CodeRun;
use kube::ResourceExt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_K8S_NAME_LENGTH: usize = 63;
const MAX_DNS_LABEL_LENGTH: usize = 63;

pub struct ResourceNaming;

impl ResourceNaming {
    /// Generate job name with guaranteed length compliance
    /// This is the single source of truth for job names
    pub fn job_name(code_run: &CodeRun) -> String {
        let namespace = code_run.namespace().unwrap_or("default".to_string());
        let name = code_run.name_any();
        let uid_suffix = code_run
            .metadata
            .uid
            .as_ref()
            .map(|uid| &uid[..8])
            .unwrap_or("unknown");
        let task_id = code_run.spec.task_id;
        let context_version = code_run.spec.context_version;

        let base_name =
            format!("code-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}");

        Self::ensure_k8s_name_length(&base_name)
    }

    /// Generate service name with length compliance
    /// Fixes the DNS label length violation that was causing reconciliation failures
    pub fn headless_service_name(job_name: &str) -> String {
        const BRIDGE_SUFFIX: &str = "-bridge";
        const MAX_BASE_LENGTH: usize = MAX_DNS_LABEL_LENGTH - BRIDGE_SUFFIX.len();

        if job_name.len() <= MAX_BASE_LENGTH {
            format!("{job_name}{BRIDGE_SUFFIX}")
        } else {
            // Use deterministic hash for long names
            let hash = Self::hash_string(job_name);
            let task_id = Self::extract_task_id_from_job_name(job_name);
            format!("bridge-t{task_id}-{hash}")
        }
    }

    /// Extract agent name from GitHub app (e.g., "5DLabs-Rex" -> "rex")
    pub fn extract_agent_name(github_app: &str) -> crate::tasks::types::Result<String> {
        github_app
            .split('-')
            .next_back()
            .map(|s| s.to_lowercase())
            .ok_or_else(|| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Invalid GitHub app format: {github_app}"
                ))
            })
    }

    // Private helper methods
    fn ensure_k8s_name_length(name: &str) -> String {
        if name.len() <= MAX_K8S_NAME_LENGTH {
            name.to_string()
        } else {
            // Intelligent truncation: preserve the meaningful suffix
            // Keep the pattern: {uid}-t{task}-v{version} (last 3 parts)
            let parts: Vec<&str> = name.split('-').collect();
            if parts.len() >= 4 {
                // Preserve the last 3 parts: {uid}-t{task}-v{version}
                let preserved_suffix = parts[parts.len() - 3..].join("-");
                let available_space =
                    MAX_K8S_NAME_LENGTH.saturating_sub(preserved_suffix.len() + 1);

                let prefix_len = available_space.min(name.len());
                if prefix_len > 0 {
                    format!("{}-{}", &name[..prefix_len], preserved_suffix)
                        .chars()
                        .take(MAX_K8S_NAME_LENGTH)
                        .collect()
                } else {
                    preserved_suffix.chars().take(MAX_K8S_NAME_LENGTH).collect()
                }
            } else {
                // Fallback: simple truncation
                name.chars().take(MAX_K8S_NAME_LENGTH).collect()
            }
        }
    }

    fn hash_string(input: &str) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())[..8].to_string()
    }

    fn extract_task_id_from_job_name(job_name: &str) -> String {
        // Extract task ID from job name pattern
        job_name
            .split('-')
            .find(|part| part.starts_with('t') && part[1..].chars().all(|c| c.is_ascii_digit()))
            .map(|part| part[1..].to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}
