use crate::crds::CodeRun;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MAX_K8S_NAME_LENGTH: usize = 63;
const MAX_DNS_LABEL_LENGTH: usize = 63;
const MONITOR_JOB_PREFIX: &str = "monitor-";
const REMEDIATION_JOB_PREFIX: &str = "remediation-";
const HEAL_REMEDIATION_JOB_PREFIX: &str = "heal-remediation-";
const REVIEW_JOB_PREFIX: &str = "review-";
const REMEDIATE_JOB_PREFIX: &str = "remediate-";
const INTAKE_JOB_PREFIX: &str = "intake-";
const PLAY_JOB_PREFIX: &str = "play-";
const MCP_JOB_PREFIX: &str = "mcp-";
const WORKSPACE_CLEANUP_JOB_SUFFIX: &str = "-workspace-cleanup";

pub struct ResourceNaming;

impl ResourceNaming {
    /// Generate job name with guaranteed length compliance.
    ///
    /// Format varies by type:
    /// - Play CodeRun: `play-coderun-pr{pr}-t{task_id}-{agent}-{cli}-{uid}-v{version}`
    /// - Play Trigger: `play-{service}-{uid}-v{version}` (Morgan starting workflow)
    /// - Intake: `intake-t{task_id}-{agent}-{cli}-{uid}-v{version}`
    /// - Heal Remediation: `heal-remediation-t{task_id}-{agent}-{uid}-v{version}`
    /// - Monitor: `monitor-t{task_id}-{agent}-{uid}-v{version}`
    /// - Remediation: `remediation-t{task_id}-{agent}-{uid}-v{version}`
    /// - Review: `review-pr{pr}-{agent}-{model}-{uid}-v{version}`
    /// - Remediate: `remediate-pr{pr}-{agent}-{model}-{uid}-v{version}`
    ///
    /// This is the single source of truth for job names.
    #[must_use]
    #[allow(clippy::too_many_lines)] // Complex function not easily split
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

        // Extract agent name: prefer implementation_agent, then github_app
        let agent = code_run
            .spec
            .implementation_agent
            .as_ref()
            .filter(|a| !a.is_empty())
            .cloned()
            .or_else(|| {
                code_run
                    .spec
                    .github_app
                    .as_ref()
                    .and_then(|app| Self::extract_agent_name(app).ok())
            })
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

        // Handle intake tasks (Morgan PRD processing)
        // Format: intake-t{task_id}-{agent}-{cli}-{uid}-v{version}
        if run_type == "intake" {
            // Extract CLI type if available
            let cli = code_run.spec.cli_config.as_ref().map_or_else(
                || "unknown".to_string(),
                |config| config.cli_type.to_string(),
            );
            let base_name = format!("t{task_id}-{agent}-{cli}-{uid_suffix}-v{context_version}");
            let available = MAX_K8S_NAME_LENGTH.saturating_sub(INTAKE_JOB_PREFIX.len());
            let trimmed = Self::ensure_k8s_name_length(&base_name, available);
            return format!("{INTAKE_JOB_PREFIX}{trimmed}");
        }

        // Handle play tasks (Morgan starting play workflow with project ConfigMap)
        // Format: play-{service}-{uid}-v{version}
        if run_type == "play" {
            let service = &code_run.spec.service;
            let base_name = format!("{service}-{uid_suffix}-v{context_version}");
            let available = MAX_K8S_NAME_LENGTH.saturating_sub(PLAY_JOB_PREFIX.len());
            let trimmed = Self::ensure_k8s_name_length(&base_name, available);
            return format!("{PLAY_JOB_PREFIX}{trimmed}");
        }

        // Check if this is a heal remediation CodeRun
        // Detected via: label, template prefix, or service name
        let heal_remediation_label = code_run
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("agents.platform/type"))
            .is_some_and(|v| v == "heal-remediation");
        let template_setting = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let is_heal = heal_remediation_label
            || template_setting.to_lowercase().starts_with("heal/")
            || code_run.spec.service.to_lowercase() == "heal";

        // For heal remediation CodeRuns, use heal-remediation- prefix
        if is_heal {
            let base_name = format!("t{task_id}-{agent}-{uid_suffix}-v{context_version}");
            let available = MAX_K8S_NAME_LENGTH.saturating_sub(HEAL_REMEDIATION_JOB_PREFIX.len());
            let trimmed = Self::ensure_k8s_name_length(&base_name, available);
            return format!("{HEAL_REMEDIATION_JOB_PREFIX}{trimmed}");
        }

        // Check if this is an MCP server management CodeRun
        // Detected via label: task-type starts with "mcp-server-"
        let mcp_task_type = code_run
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("task-type"))
            .filter(|v| v.starts_with("mcp-server-"));

        // For MCP CodeRuns, use mcp- prefix with task type
        // Format: mcp-{task}-{server_key}-{uid}-v{version}
        if let Some(task_type) = mcp_task_type {
            let mcp_task = task_type.strip_prefix("mcp-server-").unwrap_or("unknown");
            let server_key = code_run
                .metadata
                .labels
                .as_ref()
                .and_then(|labels| labels.get("mcp-server-key"))
                .map_or("unknown", String::as_str);
            let base_name = format!("{mcp_task}-{server_key}-{uid_suffix}-v{context_version}");
            let available = MAX_K8S_NAME_LENGTH.saturating_sub(MCP_JOB_PREFIX.len());
            let trimmed = Self::ensure_k8s_name_length(&base_name, available);
            return format!("{MCP_JOB_PREFIX}{trimmed}");
        }

        // Check if this is a watch CodeRun (monitor or remediation from watch service)
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

        // New naming: t{task_id}-{cli}-{model}-{provider}-{uid}-v{version}
        let cli_readable = Self::extract_cli_readable(&code_run.spec);
        let provider_readable = Self::extract_provider_readable(&code_run.spec);

        let base_name = if let Some(pr) = pr_number {
            format!("pr{pr}-t{task_id}-{cli_readable}-{model_short}-{provider_readable}-{uid_suffix}-v{context_version}")
        } else {
            format!("t{task_id}-{cli_readable}-{model_short}-{provider_readable}-{uid_suffix}-v{context_version}")
        };

        // No prefix for implementation CodeRuns (was play-coderun-)
        Self::ensure_k8s_name_length(&base_name, MAX_K8S_NAME_LENGTH)
    }

    /// Generate cleanup job name with length compliance.
    #[must_use]
    pub fn cleanup_job_name(code_run: &CodeRun) -> String {
        let base_name = Self::job_name(code_run);
        let available = MAX_K8S_NAME_LENGTH.saturating_sub(WORKSPACE_CLEANUP_JOB_SUFFIX.len());

        if base_name.len() <= available {
            return format!("{base_name}{WORKSPACE_CLEANUP_JOB_SUFFIX}");
        }

        let hash = Self::hash_string(&base_name);
        let max_prefix_len = available.saturating_sub(hash.len() + 1);
        let mut prefix: String = base_name.chars().take(max_prefix_len).collect();
        while prefix.ends_with('-') {
            prefix.pop();
        }

        if prefix.is_empty() {
            format!("{hash}{WORKSPACE_CLEANUP_JOB_SUFFIX}")
        } else {
            format!("{prefix}-{hash}{WORKSPACE_CLEANUP_JOB_SUFFIX}")
        }
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

    /// Shorten model name for pod naming.
    ///
    /// Produces readable, K8s-safe names like `sonnet-4`, `opus-4-6`, `gpt-4-1`.
    /// Generic algorithm — no hardcoded model list:
    /// 1. Strip trailing date suffixes (YYYYMMDD)
    /// 2. Strip leading path prefixes (accounts/fireworks/routers/…)
    /// 3. Normalize dots to dashes, keep dashes for readability
    /// 4. Collapse consecutive dashes, trim edges
    /// 5. Cap at 20 chars for K8s label safety
    fn shorten_model_name(model: &str) -> String {
        let lower = model.to_lowercase();

        // Strip trailing date suffixes like -20260205 or -20251101
        let stripped = if let Some(pos) = lower.rfind('-') {
            let suffix = &lower[pos + 1..];
            if suffix.len() == 8 && suffix.chars().all(|c| c.is_ascii_digit()) {
                &lower[..pos]
            } else {
                &lower
            }
        } else {
            &lower
        };

        // Strip leading path prefixes (e.g. "accounts/fireworks/routers/")
        let base = stripped.rsplit('/').next().unwrap_or(stripped);

        // Normalize: dots to dashes, keep alphanumeric + dashes
        let normalized: String = base
            .chars()
            .map(|c| if c == '.' { '-' } else { c })
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        // Collapse consecutive dashes and trim
        let mut result = String::with_capacity(normalized.len());
        let mut prev_dash = true; // start true to trim leading dash
        for ch in normalized.chars() {
            if ch == '-' {
                if !prev_dash {
                    result.push('-');
                }
                prev_dash = true;
            } else {
                result.push(ch);
                prev_dash = false;
            }
        }
        // Trim trailing dash
        while result.ends_with('-') {
            result.pop();
        }

        // Cap at 20 chars, don't break mid-dash
        if result.len() <= 20 {
            result
        } else {
            let truncated = &result[..20];
            truncated.trim_end_matches('-').to_string()
        }
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
            let hashed_name = format!("bridge-t{task_id}-{hash}");
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

    /// Extract readable CLI name from ACP or cli_config
    fn extract_cli_readable(spec: &crate::crds::coderun::CodeRunSpec) -> String {
        // Try ACP first entry
        if let Some(acp) = &spec.acp {
            if let Some(first) = acp.first() {
                return Self::shorten_cli_name(&first.cli);
            }
        }
        // Fallback to cli_config
        spec.cli_config.as_ref().map_or_else(
            || "unknown".to_string(),
            |config| Self::shorten_cli_name(&config.cli_type.to_string()),
        )
    }

    /// Extract readable provider name from ACP or cli_config
    fn extract_provider_readable(spec: &crate::crds::coderun::CodeRunSpec) -> String {
        // Try ACP first entry
        if let Some(acp) = &spec.acp {
            if let Some(first) = acp.first() {
                return Self::shorten_provider_name(&first.provider.name);
            }
        }
        // Fallback to cli_config.provider (Provider enum → Display string)
        spec.cli_config
            .as_ref()
            .and_then(|c| c.provider.as_ref())
            .map_or_else(
                || "default".to_string(),
                |p| Self::shorten_provider_name(&p.to_string()),
            )
    }

    /// Shorten CLI name for K8s naming (e.g., "Claude Code" → "claude-code")
    ///
    /// Generic: lowercase, spaces to dashes, strip non-alphanumeric.
    /// No hardcoded CLI list — any new CLI just works.
    pub fn shorten_cli_name(cli: &str) -> String {
        cli.to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    /// Shorten provider name for K8s naming (e.g., "Anthropic" → "anthropic")
    ///
    /// Generic: lowercase, spaces to dashes, normalize common variants.
    pub fn shorten_provider_name(provider: &str) -> String {
        let lower = provider.to_lowercase();
        // Normalize "open router" / "open-router" / "openrouter" variants
        let normalized = lower.replace("open router", "open-router").replace("openrouter", "open-router");
        normalized
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
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

    /// Legacy prefix — kept in tests to assert it is NOT produced.
    const CODERUN_JOB_PREFIX: &str = "play-coderun-";

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
                task_id: Some(42),
                service: "sample-service".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                model: "sonnet".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                ..Default::default()
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
                task_id: Some(42),
                service: "sample-service".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                model: "sonnet".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                ..Default::default()
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
                task_id: Some(42),
                service: "sample-service".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                model: "sonnet".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                env,
                ..Default::default()
            },
            status: None,
        }
    }

    #[test]
    fn job_name_has_new_format() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);

        // New format: no play-coderun- prefix
        assert!(!job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(job_name.starts_with("t42"));
        // No cli_config set, so cli is "unknown" and provider is "default"
        assert!(
            job_name.contains("unknown"),
            "Should contain unknown cli: {job_name}"
        );
        assert!(
            job_name.contains("sonnet"),
            "Should contain model: {job_name}"
        );
        assert!(
            job_name.contains("default"),
            "Should contain default provider: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn job_name_includes_pr_number_from_label() {
        let code_run = build_code_run_with_pr_label("1627");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(!job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(job_name.starts_with("pr1627-t42"));
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn job_name_includes_pr_number_from_env_fallback() {
        let code_run = build_code_run_with_pr_env("1650");
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(!job_name.starts_with(CODERUN_JOB_PREFIX));
        assert!(
            job_name.contains("pr1650"),
            "Expected job name to contain PR number from env var"
        );
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn cleanup_job_name_appends_suffix_when_base_fits() {
        let code_run = build_code_run();
        let job_name = ResourceNaming::job_name(&code_run);
        let cleanup_name = ResourceNaming::cleanup_job_name(&code_run);

        assert_eq!(
            cleanup_name,
            format!("{job_name}{WORKSPACE_CLEANUP_JOB_SUFFIX}")
        );
        assert!(cleanup_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn cleanup_job_name_truncates_with_hash_when_needed() {
        let mut code_run = build_code_run_with_pr_label("99999");
        // Use a very long model name to trigger truncation (new format doesn't include agent)
        code_run.spec.model =
            "some-extremely-long-custom-model-name-that-will-definitely-overflow-k8s-limits"
                .to_string();

        let job_name = ResourceNaming::job_name(&code_run);
        let available = MAX_K8S_NAME_LENGTH.saturating_sub(WORKSPACE_CLEANUP_JOB_SUFFIX.len());
        assert!(
            job_name.len() > available,
            "Job name should exceed cleanup available: len={} available={} name={}",
            job_name.len(),
            available,
            job_name
        );

        let cleanup_name = ResourceNaming::cleanup_job_name(&code_run);
        assert!(cleanup_name.ends_with(WORKSPACE_CLEANUP_JOB_SUFFIX));
        assert!(cleanup_name.len() <= MAX_K8S_NAME_LENGTH);
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
                task_id: Some(42),
                service: "sample-service".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                model: "sonnet".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                env,
                ..Default::default()
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
    fn service_name_stays_within_dns_limit_when_hashed() {
        let long_job_name = "x".repeat(80);

        let service_name = ResourceNaming::headless_service_name(&long_job_name);
        assert!(service_name.starts_with("bridge-"));
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
                service: "review-service".to_string(),
                repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                model: "claude-opus-4-5-20251101".to_string(),
                github_app: Some("5DLabs-Stitch".to_string()),
                ..Default::default()
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
                service: "remediate-service".to_string(),
                repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                ..Default::default()
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
            job_name.contains("claude-opus-4-5"),
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
            job_name.contains("claude-sonnet-4"),
            "Remediate job should contain shortened model name: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn shorten_model_name_handles_opus() {
        // Generic: strips date, preserves dashes for readability
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-opus-4-5-20251101"),
            "claude-opus-4-5"
        );
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-opus-4.5-20251101"),
            "claude-opus-4-5"
        );
        assert_eq!(ResourceNaming::shorten_model_name("opus"), "opus");
    }

    #[test]
    fn shorten_model_name_handles_sonnet() {
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-sonnet-4-20250514"),
            "claude-sonnet-4"
        );
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-3-5-sonnet-20241022"),
            "claude-3-5-sonnet"
        );
        assert_eq!(ResourceNaming::shorten_model_name("sonnet"), "sonnet");
    }

    #[test]
    fn shorten_model_name_handles_other_models() {
        assert_eq!(ResourceNaming::shorten_model_name("haiku"), "haiku");
        assert_eq!(ResourceNaming::shorten_model_name("gpt-4"), "gpt-4");
        assert_eq!(ResourceNaming::shorten_model_name("gemini-pro"), "gemini-pro");
        assert_eq!(
            ResourceNaming::shorten_model_name("some-custom-model"),
            "some-custom-model"
        );
    }

    fn build_heal_remediation_code_run() -> CodeRun {
        use crate::cli::types::CLIType;
        use crate::crds::coderun::CLIConfig;
        use serde_json::json;

        let mut labels = BTreeMap::new();
        labels.insert(
            "agents.platform/type".to_string(),
            "heal-remediation".to_string(),
        );

        let mut settings = HashMap::new();
        settings.insert("template".to_string(), json!("heal/claude"));

        CodeRun {
            metadata: ObjectMeta {
                name: Some("heal-remediation-task123-a2-abc12345".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("abc12345def67890".to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: Some(CLIConfig {
                    cli_type: CLIType::Claude,
                    model: "claude-opus-4-5-20251101".to_string(),
                    settings,
                    max_tokens: None,
                    temperature: None,
                    model_rotation: None,
                    provider: None,
                    provider_base_url: None,
                }),
                task_id: Some(123),
                service: "heal".to_string(),
                repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                docs_repository_url: "https://github.com/5dlabs/cto.git".to_string(),
                model: "claude-opus-4-5-20251101".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                ..Default::default()
            },
            status: None,
        }
    }

    #[test]
    fn heal_remediation_job_name_has_correct_prefix() {
        let code_run = build_heal_remediation_code_run();
        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.starts_with(HEAL_REMEDIATION_JOB_PREFIX),
            "Heal remediation job should start with 'heal-remediation-': {job_name}"
        );
        assert!(
            job_name.contains("t123"),
            "Heal remediation job should contain task ID: {job_name}"
        );
        assert!(
            job_name.contains("rex"),
            "Heal remediation job should contain agent name: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn heal_service_triggers_heal_remediation_prefix() {
        // Test that service="heal" alone triggers heal-remediation- prefix
        let code_run = CodeRun {
            metadata: ObjectMeta {
                name: Some("test-heal".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("1234567890abcdef".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                task_id: Some(456),
                service: "heal".to_string(),
                repository_url: "https://github.com/example/repo.git".to_string(),
                docs_repository_url: "https://github.com/example/docs.git".to_string(),
                model: "sonnet".to_string(),
                github_app: Some("5DLabs-Rex".to_string()),
                ..Default::default()
            },
            status: None,
        };

        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.starts_with(HEAL_REMEDIATION_JOB_PREFIX),
            "Service 'heal' should trigger heal-remediation- prefix: {job_name}"
        );
    }

    #[test]
    fn intake_job_name_has_correct_prefix() {
        use crate::cli::types::CLIType;
        use crate::crds::coderun::CLIConfig;

        let settings = HashMap::new();

        let code_run = CodeRun {
            metadata: ObjectMeta {
                name: Some("intake-run".to_string()),
                namespace: Some("cto".to_string()),
                uid: Some("20bce7e0cf424515".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "intake".to_string(),
                cli_config: Some(CLIConfig {
                    cli_type: CLIType::Claude,
                    model: "claude-opus-4-5-20251101".to_string(),
                    settings,
                    max_tokens: None,
                    temperature: None,
                    model_rotation: None,
                    provider: None,
                    provider_base_url: None,
                }),
                task_id: Some(0),
                service: "prd-alerthub-e2e-test".to_string(),
                repository_url: "https://github.com/5dlabs/prd-alerthub-e2e-test".to_string(),
                docs_repository_url: "https://github.com/5dlabs/prd-alerthub-e2e-test".to_string(),
                model: "claude-opus-4-5-20251101".to_string(),
                github_app: Some("5DLabs-Morgan".to_string()),
                ..Default::default()
            },
            status: None,
        };

        let job_name = ResourceNaming::job_name(&code_run);

        assert!(
            job_name.starts_with(INTAKE_JOB_PREFIX),
            "Intake job should start with 'intake-': {job_name}"
        );
        assert!(
            job_name.contains("t0"),
            "Intake job should contain task ID: {job_name}"
        );
        assert!(
            job_name.contains("morgan"),
            "Intake job should contain agent name: {job_name}"
        );
        assert!(
            job_name.contains("claude"),
            "Intake job should contain CLI type: {job_name}"
        );
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }

    #[test]
    fn shorten_cli_name_works() {
        assert_eq!(ResourceNaming::shorten_cli_name("Claude Code"), "claude-code");
        // "claude" stays "claude" — generic, no special-casing
        assert_eq!(ResourceNaming::shorten_cli_name("claude"), "claude");
        assert_eq!(ResourceNaming::shorten_cli_name("Codex"), "codex");
        assert_eq!(ResourceNaming::shorten_cli_name("Factory"), "factory");
        assert_eq!(
            ResourceNaming::shorten_cli_name("Some New CLI"),
            "some-new-cli"
        );
    }

    #[test]
    fn shorten_provider_name_works() {
        assert_eq!(
            ResourceNaming::shorten_provider_name("Anthropic"),
            "anthropic"
        );
        assert_eq!(ResourceNaming::shorten_provider_name("OpenAI"), "openai");
        assert_eq!(
            ResourceNaming::shorten_provider_name("OpenRouter"),
            "open-router"
        );
        assert_eq!(
            ResourceNaming::shorten_provider_name("Fireworks"),
            "fireworks"
        );
        assert_eq!(
            ResourceNaming::shorten_provider_name("Some Provider"),
            "some-provider"
        );
    }

    #[test]
    fn shorten_model_name_handles_new_models() {
        // Generic algorithm: strip date, dots to dashes, preserve dashes
        assert_eq!(ResourceNaming::shorten_model_name("gpt-5.2-codex"), "gpt-5-2-codex");
        assert_eq!(ResourceNaming::shorten_model_name("o4-mini"), "o4-mini");
        assert_eq!(ResourceNaming::shorten_model_name("gpt-4.1"), "gpt-4-1");
        assert_eq!(
            ResourceNaming::shorten_model_name("claude-opus-4-6-20260205"),
            "claude-opus-4-6"
        );
        // Fireworks path-prefixed models get the base name only
        assert_eq!(
            ResourceNaming::shorten_model_name("accounts/fireworks/routers/kimi-k2p5-turbo"),
            "kimi-k2p5-turbo"
        );
    }

    #[test]
    fn job_name_uses_implementation_agent_when_set() {
        let mut code_run = build_code_run();
        code_run.spec.implementation_agent = Some("blaze".to_string());
        let job_name = ResourceNaming::job_name(&code_run);
        // New format: t{task_id}-{cli}-{model}-{provider}-{uid}-v{version}
        assert!(job_name.contains("t42"));
        assert!(job_name.len() <= MAX_K8S_NAME_LENGTH);
    }
}
