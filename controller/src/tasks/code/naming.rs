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

        // Extract agent name if available
        let agent = code_run
            .spec
            .github_app
            .as_ref()
            .and_then(|app| Self::extract_agent_name(app).ok())
            .unwrap_or_else(|| "default".to_string());

        // Extract CLI type if available
        let cli = code_run
            .spec
            .cli_config
            .as_ref()
            .map(|config| config.cli_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let base_name = format!(
            "code-{agent}-{cli}-{namespace}-{name}-{uid_suffix}-t{task_id}-v{context_version}"
        );

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
            // Intelligent truncation: preserve the meaningful parts
            // Format: code-{agent}-{cli}-{namespace}-{name}-{uid}-t{task}-v{version}
            // Priority: agent, cli, uid, task_id, version > namespace, name
            let parts: Vec<&str> = name.split('-').collect();
            
            if parts.len() >= 8 {
                // New format with agent and CLI
                // Preserve: code-{agent}-{cli}-...-{uid}-t{task}-v{version}
                let agent = parts[1];
                let cli = parts[2];
                let uid = parts[parts.len() - 3];
                let task = parts[parts.len() - 2];
                let version = parts[parts.len() - 1];
                
                // Build compact name with hash for middle parts if needed
                let suffix = format!("{uid}-{task}-{version}");
                let prefix = format!("code-{agent}-{cli}");
                let available_space = MAX_K8S_NAME_LENGTH.saturating_sub(prefix.len() + suffix.len() + 2);
                
                if available_space > 8 {
                    // Room for some of the middle parts
                    let middle_parts = &parts[3..parts.len() - 3];
                    let middle = middle_parts.join("-");
                    let truncated_middle = if middle.len() > available_space {
                        format!("{}-{}", &middle[..available_space.saturating_sub(9)], Self::hash_string(&middle))
                    } else {
                        middle
                    };
                    format!("{prefix}-{truncated_middle}-{suffix}")
                } else {
                    // Very tight space, just use essential parts
                    format!("{prefix}-{suffix}")
                }
            } else if parts.len() >= 4 {
                // Legacy format without agent/CLI
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
