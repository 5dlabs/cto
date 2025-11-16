use crate::crds::CodeRun;
use kube::ResourceExt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_K8S_NAME_LENGTH: usize = 63;
const MAX_DNS_LABEL_LENGTH: usize = 63;
const CODERUN_JOB_PREFIX: &str = "play-coderun-";

pub struct ResourceNaming;

impl ResourceNaming {
    /// Generate job name with guaranteed length compliance
    /// Format: task-{task_id}-{agent}-{cli}-{namespace}-{name}-{uid}-v{version}
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
        let cli = code_run.spec.cli_config.as_ref().map_or_else(
            || "unknown".to_string(),
            |config| config.cli_type.to_string(),
        );

        let base_name = format!(
            "task-{task_id}-{agent}-{cli}-{namespace}-{name}-{uid_suffix}-v{context_version}"
        );

        let available = MAX_K8S_NAME_LENGTH.saturating_sub(CODERUN_JOB_PREFIX.len());
        let trimmed = Self::ensure_k8s_name_length(&base_name, available);

        format!("{CODERUN_JOB_PREFIX}{trimmed}")
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
            let hashed_name = format!("{CODERUN_JOB_PREFIX}bridge-t{task_id}-{hash}");
            Self::ensure_k8s_name_length(&hashed_name, MAX_DNS_LABEL_LENGTH)
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
    fn ensure_k8s_name_length(name: &str, limit: usize) -> String {
        if name.len() <= limit {
            name.to_string()
        } else {
            // Intelligent truncation: preserve the meaningful parts
            // Format: task-{task_id}-{agent}-{cli}-{namespace}-{name}-{uid}-v{version}
            // Priority: task_id, agent, cli, uid, version > namespace, name
            let parts: Vec<&str> = name.split('-').collect();

            if parts.len() >= 8 {
                // New format: task-{task_id}-{agent}-{cli}-{namespace}-{name}-{uid}-v{version}
                // Preserve: task-{task_id}-{agent}-{cli}-...-{uid}-v{version}
                let task = parts[1]; // task_id is now at position 1
                let agent = parts[2];
                let cli = parts[3];
                let uid = parts[parts.len() - 2];
                let version = parts[parts.len() - 1];

                // Build compact name with hash for middle parts if needed
                let suffix = format!("{uid}-{version}");
                let prefix = format!("task-{task}-{agent}-{cli}");
                let available_space = limit.saturating_sub(prefix.len() + suffix.len() + 2);

                if available_space > 8 {
                    // Room for some of the middle parts
                    let middle_parts = &parts[4..parts.len() - 2];
                    let middle = middle_parts.join("-");
                    let truncated_middle = if middle.len() > available_space {
                        format!(
                            "{}-{}",
                            &middle[..available_space.saturating_sub(9)],
                            Self::hash_string(&middle)
                        )
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
                let available_space = limit.saturating_sub(preserved_suffix.len() + 1);

                let prefix_len = available_space.min(name.len());
                if prefix_len > 0 {
                    format!("{}-{}", &name[..prefix_len], preserved_suffix)
                        .chars()
                        .take(limit)
                        .collect()
                } else {
                    preserved_suffix.chars().take(limit).collect()
                }
            } else {
                // Fallback: simple truncation
                name.chars().take(limit).collect()
            }
        }
    }

    fn hash_string(input: &str) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())[..8].to_string()
    }

    fn extract_task_id_from_job_name(job_name: &str) -> String {
        // Extract task ID from job name pattern: task-{id}-...
        let mut parts = job_name.split('-').peekable();

        while let Some(part) = parts.next() {
            if part == "task" {
                if let Some(candidate) = parts.next() {
                    if candidate.chars().all(|c| c.is_ascii_digit()) {
                        return candidate.to_string();
                    }
                }
                break;
            }
        }

        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::coderun::CodeRunSpec;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::HashMap;

    fn build_code_run() -> CodeRun {
        CodeRun {
            metadata: ObjectMeta {
                name: Some("sample-run".to_string()),
                namespace: Some("agent-platform".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                cli_config: None,
                task_id: 42,
                service: "sample-service".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                docs_project_directory: None,
                working_directory: None,
                model: "sonnet".to_string(),
                github_user: None,
                github_app: Some("5DLabs-Rex".to_string()),
                context_version: 1,
                docs_branch: "main".to_string(),
                continue_session: false,
                overwrite_memory: false,
                env: HashMap::new(),
                env_from_secrets: vec![],
                enable_docker: Some(true),
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    #[test]
    fn job_name_has_play_coderun_prefix() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(job_name.contains("task-42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn extract_task_id_handles_prefixed_names() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);
        assert_eq!(
            ResourceNaming::extract_task_id_from_job_name(&job_name),
            "42"
        );
    }

    #[test]
    fn service_name_retains_prefix_when_hashed() {
        let mut long_job_name = String::from(CODERUN_JOB_PREFIX);
        long_job_name.push_str(&"x".repeat(80));

        let service_name = ResourceNaming::headless_service_name(&long_job_name);
        assert!(service_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(service_name.len() <= MAX_DNS_LABEL_LENGTH);
    }
}
