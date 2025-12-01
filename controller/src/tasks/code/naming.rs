use crate::crds::CodeRun;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_K8S_NAME_LENGTH: usize = 63;
const MAX_DNS_LABEL_LENGTH: usize = 63;
const CODERUN_JOB_PREFIX: &str = "play-coderun-";
const MONITOR_JOB_PREFIX: &str = "monitor-";
const REMEDIATION_JOB_PREFIX: &str = "remediation-";
const REVIEW_JOB_PREFIX: &str = "review-";
const REMEDIATE_JOB_PREFIX: &str = "remediate-";

pub struct ResourceNaming;

impl ResourceNaming {
    /// Generate job name with guaranteed length compliance.
    ///
    /// Format varies by type:
    /// - Play: `play-coderun-pr{pr}-t{task_id}-{agent}-{cli}-{uid}-v{version}`
    /// - Monitor: `monitor-t{task_id}-{agent}-{uid}-v{version}`
    /// - Remediation: `remediation-t{task_id}-{agent}-{uid}-v{version}`
    /// - Review: `review-pr{pr}-{agent}-{model}-{uid}-v{version}`
    /// - Remediate: `remediate-pr{pr}-{agent}-{model}-{uid}-v{version}`
    ///
    /// This is the single source of truth for job names.
    #[must_use]
    pub fn job_name(code_run: &CodeRun) -> String {
        let uid_suffix = code_run
            .metadata
            .uid
            .as_ref()
            .map_or("unknown", |uid| &uid[..8]);
        let task_id = code_run.spec.task_id.unwrap_or(0);
        let context_version = code_run.spec.context_version;

        // Extract PR number from labels first, then fall back to env var
        // This ensures we get PR number from both sensor-created CodeRuns (labels)
        // and from env var injection (for edge cases where labels weren't set)
        let pr_number = code_run
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("pr-number"))
            .cloned()
            .or_else(|| {
                // Fallback: check PR_NUMBER env var
                code_run
                    .spec
                    .env
                    .get("PR_NUMBER")
                    .filter(|v| !v.is_empty() && *v != "0")
                    .cloned()
            });

        // Extract agent name if available
        let agent = code_run
            .spec
            .github_app
            .as_ref()
            .and_then(|app| Self::extract_agent_name(app).ok())
            .unwrap_or_else(|| "default".to_string());

        // Extract model name (shortened for pod naming)
        let model_short = Self::shorten_model_name(&code_run.spec.model);

        // Check run type for review/remediate tasks
        let run_type = code_run.spec.run_type.as_str();

        // Handle review tasks (Stitch PR Review) and remediate tasks (Rex PR Remediation)
        // Both use the same naming pattern: {prefix}pr{pr}-{agent}-{model}-{uid}-v{version}
        if run_type == "review" {
            return Self::generate_pr_task_job_name(
                REVIEW_JOB_PREFIX,
                pr_number.as_ref(),
                &agent,
                &model_short,
                uid_suffix,
                context_version,
            );
        }

        if run_type == "remediate" {
            return Self::generate_pr_task_job_name(
                REMEDIATE_JOB_PREFIX,
                pr_number.as_ref(),
                &agent,
                &model_short,
                uid_suffix,
                context_version,
            );
        }

        // Check if this is a watch CodeRun (monitor or remediation from heal)
        let template_setting = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let watch_role = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("watchRole"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_watch = template_setting.starts_with("watch/")
            || code_run.spec.service.to_lowercase().contains("watch");

        // For watch CodeRuns, use simplified naming
        if is_watch {
            let prefix = if watch_role == "remediation" {
                REMEDIATION_JOB_PREFIX
            } else {
                MONITOR_JOB_PREFIX
            };
            let base_name = format!("t{task_id}-{agent}-{uid_suffix}-v{context_version}");
            let available = MAX_K8S_NAME_LENGTH.saturating_sub(prefix.len());
            let trimmed = Self::ensure_k8s_name_length(&base_name, available);
            return format!("{prefix}{trimmed}");
        }

        // Extract CLI type if available
        let cli = code_run.spec.cli_config.as_ref().map_or_else(
            || "unknown".to_string(),
            |config| config.cli_type.to_string(),
        );

        // Build name with PR number prefix if available for easy identification
        let base_name = if let Some(pr) = pr_number {
            format!("pr{pr}-t{task_id}-{agent}-{cli}-{uid_suffix}-v{context_version}")
        } else {
            format!("t{task_id}-{agent}-{cli}-{uid_suffix}-v{context_version}")
        };

        let available = MAX_K8S_NAME_LENGTH.saturating_sub(CODERUN_JOB_PREFIX.len());
        let trimmed = Self::ensure_k8s_name_length(&base_name, available);

        format!("{CODERUN_JOB_PREFIX}{trimmed}")
    }

    /// Generate job name for PR-related tasks (review, remediate)
    ///
    /// Format: `{prefix}pr{pr}-{agent}-{model}-{uid}-v{version}`
    /// or without PR: `{prefix}{agent}-{model}-{uid}-v{version}`
    fn generate_pr_task_job_name(
        prefix: &str,
        pr_number: Option<&String>,
        agent: &str,
        model_short: &str,
        uid_suffix: &str,
        context_version: u32,
    ) -> String {
        let base_name = if let Some(pr) = pr_number {
            format!("pr{pr}-{agent}-{model_short}-{uid_suffix}-v{context_version}")
        } else {
            format!("{agent}-{model_short}-{uid_suffix}-v{context_version}")
        };
        let available = MAX_K8S_NAME_LENGTH.saturating_sub(prefix.len());
        let trimmed = Self::ensure_k8s_name_length(&base_name, available);
        format!("{prefix}{trimmed}")
    }

    /// Shorten model name for pod naming (e.g., "claude-opus-4-5-20251101" -> "opus45")
    fn shorten_model_name(model: &str) -> String {
        let model_lower = model.to_lowercase();

        // Map common model names to short versions
        // Order matters: check more specific patterns first
        if model_lower.contains("opus") {
            if model_lower.contains("4-5") || model_lower.contains("4.5") {
                return "opus45".to_string();
            }
            return "opus".to_string();
        }
        if model_lower.contains("sonnet") {
            // Check for 3.5/3-5 BEFORE checking for 4 (dates contain 4s)
            if model_lower.contains("3.5") || model_lower.contains("3-5") {
                return "sonnet35".to_string();
            }
            // Check for sonnet-4 specifically (not just any 4 which could be in dates)
            if model_lower.contains("sonnet-4") || model_lower.contains("sonnet4") {
                return "sonnet4".to_string();
            }
            return "sonnet".to_string();
        }
        if model_lower.contains("haiku") {
            return "haiku".to_string();
        }
        if model_lower.contains("gpt-4") || model_lower.contains("gpt4") {
            return "gpt4".to_string();
        }
        if model_lower.contains("gemini") {
            if model_lower.contains("pro") {
                return "gempro".to_string();
            }
            return "gemini".to_string();
        }

        // Default: take first 8 chars, sanitize for k8s
        model_lower
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(8)
            .collect()
    }

    /// Generate service name with length compliance
    /// Fixes the DNS label length violation that was causing reconciliation failures
    #[must_use]
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
            .map(str::to_lowercase)
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
            // New format: pr{pr_number}-t{task_id}-{agent}-{cli}-{uid}-v{version}
            // or: t{task_id}-{agent}-{cli}-{uid}-v{version}
            // Priority: pr_number, task_id, agent, uid, version > cli
            let parts: Vec<&str> = name.split('-').collect();

            if parts.len() >= 5 {
                // Check if first part is pr{number}
                let has_pr = parts[0].starts_with("pr");
                let (pr_part, task_idx) = if has_pr {
                    (Some(parts[0]), 1)
                } else {
                    (None, 0)
                };

                let task = parts[task_idx];
                let agent = parts.get(task_idx + 1).unwrap_or(&"unknown");
                let uid = parts[parts.len() - 2];
                let version = parts[parts.len() - 1];

                // Build compact name
                let suffix = format!("{uid}-{version}");
                let prefix = if let Some(pr) = pr_part {
                    format!("{pr}-{task}-{agent}")
                } else {
                    format!("{task}-{agent}")
                };

                let base = format!("{prefix}-{suffix}");
                if base.len() <= limit {
                    base
                } else {
                    // Ultra compact: pr-task-uid-version
                    let ultra_compact = if let Some(pr) = pr_part {
                        format!("{pr}-{task}-{suffix}")
                    } else {
                        format!("{task}-{suffix}")
                    };
                    ultra_compact.chars().take(limit).collect()
                }
            } else {
                // Fallback: simple truncation preserving start
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
        // Extract task ID from job name patterns:
        // New format: play-coderun-pr{pr}-t{task_id}-... or play-coderun-t{task_id}-...
        // Legacy format: play-coderun-task-{task_id}-...
        for part in job_name.split('-') {
            // New compact format: t{number}
            if part.starts_with('t') && part.len() > 1 {
                let candidate = &part[1..];
                if candidate.chars().all(|c| c.is_ascii_digit()) {
                    return candidate.to_string();
                }
            }
        }

        // Legacy format: task-{number}
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

    /// Extract PR number from job name if present
    /// Format: play-coderun-pr{pr_number}-t{task_id}-...
    #[must_use]
    pub fn extract_pr_number_from_job_name(job_name: &str) -> Option<String> {
        for part in job_name.split('-') {
            if part.starts_with("pr") && part.len() > 2 {
                let candidate = &part[2..];
                if candidate.chars().all(|c| c.is_ascii_digit()) {
                    return Some(candidate.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::coderun::CodeRunSpec;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::{BTreeMap, HashMap};

    fn build_code_run() -> CodeRun {
        CodeRun {
            metadata: ObjectMeta {
                name: Some("sample-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(42),
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
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    fn build_code_run_with_pr_label(pr_number: &str) -> CodeRun {
        let mut labels = BTreeMap::new();
        labels.insert("pr-number".to_string(), pr_number.to_string());

        CodeRun {
            metadata: ObjectMeta {
                name: Some("sample-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(42),
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
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    fn build_code_run_with_pr_env(pr_number: &str) -> CodeRun {
        let mut env = HashMap::new();
        env.insert("PR_NUMBER".to_string(), pr_number.to_string());

        CodeRun {
            metadata: ObjectMeta {
                name: Some("sample-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(42),
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
                env,
                env_from_secrets: vec![],
                enable_docker: true,
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
        // New format uses t{task_id} instead of task-{task_id}
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn job_name_includes_pr_number_from_label() {
        let code_run = build_code_run_with_pr_label("1627");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(job_name.contains("pr1627"));
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn job_name_includes_pr_number_from_env_fallback() {
        let code_run = build_code_run_with_pr_env("1650");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(
            job_name.contains("pr1650"),
            "Expected job name to contain PR number from env var"
        );
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn job_name_label_takes_priority_over_env() {
        // Create CodeRun with both label and env var, label should win
        let mut labels = BTreeMap::new();
        labels.insert("pr-number".to_string(), "1627".to_string());
        let mut env = HashMap::new();
        env.insert("PR_NUMBER".to_string(), "9999".to_string());

        let code_run = CodeRun {
            metadata: ObjectMeta {
                name: Some("sample-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(42),
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
                env,
                env_from_secrets: vec![],
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        };

        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.contains("pr1627"),
            "Label should take priority over env: {job_name}"
        );
        assert!(
            !job_name.contains("pr9999"),
            "Env var PR should not appear when label exists: {job_name}"
        );
    }

    #[test]
    fn job_name_ignores_zero_pr_number_env() {
        let code_run = build_code_run_with_pr_env("0");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            !job_name.contains("pr0"),
            "Zero PR number should be ignored: {job_name}"
        );
    }

    #[test]
    fn job_name_ignores_empty_pr_number_env() {
        let code_run = build_code_run_with_pr_env("");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            !job_name.contains("pr-"),
            "Empty PR number should be ignored: {job_name}"
        );
    }

    #[test]
    fn extract_task_id_handles_new_format() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);
        assert_eq!(
            ResourceNaming::extract_task_id_from_job_name(&job_name),
            "42"
        );
    }

    #[test]
    fn extract_task_id_handles_pr_format() {
        let code_run = build_code_run_with_pr_label("1627");
        let job_name = ResourceNaming::job_name(&code_run);
        assert_eq!(
            ResourceNaming::extract_task_id_from_job_name(&job_name),
            "42"
        );
    }

    #[test]
    fn extract_pr_number_from_job_name_works() {
        let code_run = build_code_run_with_pr_label("1627");
        let job_name = ResourceNaming::job_name(&code_run);
        assert_eq!(
            ResourceNaming::extract_pr_number_from_job_name(&job_name),
            Some("1627".to_string())
        );
    }

    #[test]
    fn extract_pr_number_returns_none_without_pr() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);
        assert_eq!(
            ResourceNaming::extract_pr_number_from_job_name(&job_name),
            None
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

    fn build_review_code_run(pr_number: &str) -> CodeRun {
        let mut labels = BTreeMap::new();
        labels.insert("pr-number".to_string(), pr_number.to_string());

        CodeRun {
            metadata: ObjectMeta {
                name: Some("review-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("abcd1234efgh5678".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "review".to_string(),
                cli_config: None,
                task_id: None,
                service: "review-service".to_string(),
                repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_project_directory: None,
                working_directory: None,
                model: "claude-opus-4-5-20251101".to_string(),
                github_user: None,
                github_app: Some("5DLabs-Stitch".to_string()),
                context_version: 1,
                docs_branch: "main".to_string(),
                continue_session: false,
                overwrite_memory: false,
                env: HashMap::new(),
                env_from_secrets: vec![],
                enable_docker: false,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    fn build_remediate_code_run(pr_number: &str) -> CodeRun {
        let mut labels = BTreeMap::new();
        labels.insert("pr-number".to_string(), pr_number.to_string());

        CodeRun {
            metadata: ObjectMeta {
                name: Some("remediate-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("efgh5678abcd1234".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "remediate".to_string(),
                cli_config: None,
                task_id: None,
                service: "remediate-service".to_string(),
                repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_project_directory: None,
                working_directory: None,
                model: "claude-sonnet-4-20250514".to_string(),
                github_user: None,
                github_app: Some("5DLabs-Rex".to_string()),
                context_version: 1,
                docs_branch: "main".to_string(),
                continue_session: false,
                overwrite_memory: false,
                env: HashMap::new(),
                env_from_secrets: vec![],
                enable_docker: false,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    #[test]
    fn review_job_name_has_correct_prefix() {
        let code_run = build_review_code_run("1234");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.starts_with(REVIEW_JOB_PREFIX),
            "Review job should start with 'review-': {job_name}"
        );
        assert!(
            job_name.contains("pr1234"),
            "Review job should contain PR number: {job_name}"
        );
        assert!(
            job_name.contains("stitch"),
            "Review job should contain agent name: {job_name}"
        );
        assert!(
            job_name.contains("opus45"),
            "Review job should contain shortened model name: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn remediate_job_name_has_correct_prefix() {
        let code_run = build_remediate_code_run("5678");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.starts_with(REMEDIATE_JOB_PREFIX),
            "Remediate job should start with 'remediate-': {job_name}"
        );
        assert!(
            job_name.contains("pr5678"),
            "Remediate job should contain PR number: {job_name}"
        );
        assert!(
            job_name.contains("rex"),
            "Remediate job should contain agent name: {job_name}"
        );
        assert!(
            job_name.contains("sonnet4"),
            "Remediate job should contain shortened model name: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn shorten_model_name_handles_opus() {
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-opus-4-5-20251101"),
            "opus45"
        );
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-opus-4.5-20251101"),
            "opus45"
        );
        assert_eq!(ResourceNaming::shorten_model_name("opus"), "opus");
    }

    #[test]
    fn shorten_model_name_handles_sonnet() {
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-sonnet-4-20250514"),
            "sonnet4"
        );
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-3-5-sonnet-20241022"),
            "sonnet35"
        );
        assert_eq!(ResourceNaming::shorten_model_name("sonnet"), "sonnet");
    }

    #[test]
    fn shorten_model_name_handles_other_models() {
        assert_eq!(ResourceNaming::shorten_model_name("haiku"), "haiku");
        assert_eq!(ResourceNaming::shorten_model_name("gpt-4"), "gpt4");
        assert_eq!(
            ResourceNaming::shorten_model_name("gemini-pro"),
            "gempro"
        );
        assert_eq!(
            ResourceNaming::shorten_model_name("some-custom-model"),
            "somecust"
        );
    }
}
