//! `CodeRun` spawner for CI remediation.
//!
//! Creates Kubernetes `CodeRun` resources with:
//! - Deduplication to prevent duplicate remediation attempts
//! - Enriched prompts from templates
//! - Proper labels for tracking

use anyhow::{bail, Context as _, Result};
use chrono::Utc;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::process::Command;
use tracing::{debug, info, warn};

use super::types::{Agent, CiFailureType, RemediationConfig, RemediationContext};

/// `CodeRun` spawner for CI remediation.
pub struct CodeRunSpawner {
    /// Configuration
    config: RemediationConfig,
    /// Template engine
    templates: Handlebars<'static>,
    /// Kubernetes namespace
    namespace: String,
    /// Repository
    repository: String,
}

impl CodeRunSpawner {
    /// Create a new spawner.
    ///
    /// # Errors
    ///
    /// Returns an error if the template engine cannot be initialized.
    pub fn new(config: RemediationConfig, namespace: &str, repository: &str) -> Result<Self> {
        let mut templates = Handlebars::new();
        templates.set_strict_mode(true);

        // Register built-in helpers
        templates.register_helper("concat", Box::new(crate::templates::concat_helper));
        templates.register_helper("lowercase", Box::new(crate::templates::lowercase_helper));

        Ok(Self {
            config,
            templates,
            namespace: namespace.to_string(),
            repository: repository.to_string(),
        })
    }

    /// Load templates from a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or templates are invalid.
    pub fn load_templates(&mut self, dir: &str) -> Result<()> {
        use std::fs;

        let entries = fs::read_dir(dir).context("Failed to read templates directory")?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "hbs") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read template: {}", path.display()))?;
                self.templates
                    .register_template_string(name, &content)
                    .context(format!("Failed to register template: {name}"))?;
            }
        }
        Ok(())
    }

    /// Register a template from string.
    ///
    /// # Errors
    ///
    /// Returns an error if the template is invalid.
    pub fn register_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.templates
            .register_template_string(name, content)
            .context("Failed to register template")
    }

    /// Check for existing remediation `CodeRun`.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl command fails.
    pub fn has_existing_remediation(&self, workflow_run_id: u64) -> Result<bool> {
        let label_selector = format!(
            "app.kubernetes.io/name=healer,healer/workflow-run-id={}",
            workflow_run_id
        );

        let output = Command::new("kubectl")
            .args([
                "get",
                "coderuns",
                "-n",
                &self.namespace,
                "-l",
                &label_selector,
                "-o",
                "jsonpath={.items[*].metadata.name}",
            ])
            .output()
            .context("Failed to check for existing CodeRuns")?;

        if !output.status.success() {
            // If we can't check, assume none exists to avoid blocking
            warn!("Could not verify existing CodeRuns, assuming none exist");
            return Ok(false);
        }

        let names = String::from_utf8_lossy(&output.stdout);
        let exists = !names.trim().is_empty();

        if exists {
            debug!("Found existing CodeRun for workflow run {workflow_run_id}: {names}");
        }

        Ok(exists)
    }

    /// Check for recent remediation within time window.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl command fails.
    pub fn has_recent_remediation(&self, branch: &str) -> Result<bool> {
        let time_window_secs = i64::from(self.config.time_window_mins) * 60;
        let cutoff = Utc::now() - chrono::Duration::seconds(time_window_secs);

        // Get CodeRuns created after cutoff
        let label_selector = format!("app.kubernetes.io/name=healer,healer/branch={branch}");

        let output = Command::new("kubectl")
            .args([
                "get",
                "coderuns",
                "-n",
                &self.namespace,
                "-l",
                &label_selector,
                "-o",
                "jsonpath={range .items[*]}{.metadata.creationTimestamp}:{.metadata.name}{\"\\n\"}{end}",
            ])
            .output()
            .context("Failed to check for recent CodeRuns")?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() < 2 {
                continue;
            }

            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(parts[0]) {
                if timestamp > cutoff {
                    debug!(
                        "Found recent CodeRun {} created at {} (within time window)",
                        parts[1], parts[0]
                    );
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Spawn a `CodeRun` for CI remediation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A remediation already exists for this workflow run
    /// - A recent remediation exists for this branch
    /// - Prompt rendering fails
    /// - kubectl apply fails
    pub fn spawn(&self, agent: Agent, ctx: &RemediationContext) -> Result<String> {
        // Check for existing remediation
        if let Some(failure) = &ctx.failure {
            if self.has_existing_remediation(failure.workflow_run_id)? {
                bail!(
                    "CodeRun already exists for workflow run {}",
                    failure.workflow_run_id
                );
            }

            // Check time window deduplication
            if self.has_recent_remediation(&failure.branch)? {
                bail!(
                    "Recent remediation exists for branch {} (within {} min window)",
                    failure.branch,
                    self.config.time_window_mins
                );
            }
        }

        // Render the prompt
        let prompt = self.render_prompt(agent, ctx)?;

        // Build the CodeRun YAML
        let coderun_yaml = self.build_coderun_yaml(agent, ctx, &prompt);

        // Apply the CodeRun
        let coderun_name = Self::apply_coderun(&coderun_yaml)?;

        info!(
            "Spawned CodeRun {} for {:?} with agent {:?}",
            coderun_name,
            ctx.failure_type,
            agent
        );

        Ok(coderun_name)
    }

    /// Render the prompt for an agent.
    fn render_prompt(&self, agent: Agent, ctx: &RemediationContext) -> Result<String> {
        let is_retry = !ctx.previous_attempts.is_empty();

        // Use retry template for subsequent attempts
        let template_name = if is_retry {
            "ci/retry".to_string()
        } else {
            format!("ci/{}", agent.template_name())
        };

        // Build template data using serde_json::Map to avoid macro recursion limits
        let data = Self::build_template_data(agent, ctx, is_retry, self.config.max_attempts);

        // Try to render the template
        if self.templates.has_template(&template_name) {
            self.templates
                .render(&template_name, &data)
                .context("Failed to render prompt template")
        } else {
            // Fallback to a generic prompt
            Ok(Self::build_generic_prompt(agent, ctx))
        }
    }

    /// Build template data for rendering.
    fn build_template_data(
        agent: Agent,
        ctx: &RemediationContext,
        is_retry: bool,
        max_attempts: u32,
    ) -> serde_json::Value {
        use serde_json::Map;

        let mut data = Map::new();

        // Basic failure info
        if let Some(failure) = &ctx.failure {
            data.insert("repository".into(), json!(&failure.repository));
            data.insert("workflow_name".into(), json!(&failure.workflow_name));
            data.insert("workflow_url".into(), json!(&failure.html_url));
            data.insert("html_url".into(), json!(&failure.html_url));
            data.insert("branch".into(), json!(&failure.branch));
            data.insert("head_sha".into(), json!(&failure.head_sha));
            data.insert("commit_sha".into(), json!(&failure.head_sha));
            data.insert("commit_message".into(), json!(&failure.commit_message));
            if let Some(job_name) = &failure.job_name {
                data.insert("job_name".into(), json!(job_name));
            }
        }

        // Logs
        data.insert("logs".into(), json!(&ctx.workflow_logs));
        data.insert("workflow_logs".into(), json!(&ctx.workflow_logs));
        data.insert("recent_error_logs".into(), json!(&ctx.recent_logs));

        // PR context
        if let Some(pr) = &ctx.pr {
            data.insert("pr_number".into(), json!(pr.number));
            data.insert("pr_title".into(), json!(&pr.title));
            data.insert("pr_url".into(), json!(&pr.html_url));
            data.insert("target_branch".into(), json!(&pr.head_ref));
        } else if let Some(failure) = &ctx.failure {
            data.insert("target_branch".into(), json!(&failure.branch));
        }

        // Changed files
        data.insert("changed_files".into(), json!(&ctx.changed_files));
        data.insert("file_diff_summary".into(), json!(ctx.summarize_diff()));

        // ArgoCD context
        if let Some(argocd) = &ctx.argocd_status {
            data.insert("argocd_status".into(), json!(argocd));
            data.insert("argocd_app_status".into(), json!(&argocd.health));
            data.insert("argocd_sync_status".into(), json!(&argocd.sync));
            data.insert(
                "argocd_resources_unhealthy".into(),
                json!(&argocd.unhealthy_resources),
            );
        }

        // Pod state
        if let Some(pod_state) = &ctx.pod_state {
            data.insert("related_pods".into(), json!(&pod_state.names));
            data.insert("pod_events".into(), json!(&pod_state.events));
        }

        // Classification
        if let Some(failure_type) = &ctx.failure_type {
            data.insert("failure_type".into(), json!(failure_type.short_name()));
            data.insert("failure_category".into(), json!(failure_type.category()));
            data.insert(
                "suggested_fix_approach".into(),
                json!(failure_type.fix_approach()),
            );
        }

        // Security alert info
        if let Some(alert) = &ctx.security_alert {
            data.insert("alert_type".into(), json!(&alert.alert_type));
            data.insert("severity".into(), json!(&alert.severity));
            data.insert("description".into(), json!(&alert.description));
            if let Some(cve_id) = &alert.cve_id {
                data.insert("cve_id".into(), json!(cve_id));
            }
            if let Some(package_name) = &alert.package_name {
                data.insert("package_name".into(), json!(package_name));
            }
        }

        // Historical context
        data.insert("historical_context".into(), json!(&ctx.historical));

        // Retry context
        data.insert(
            "attempt_number".into(),
            json!(ctx.previous_attempts.len() + 1),
        );
        data.insert("max_attempts".into(), json!(max_attempts));
        data.insert("previous_attempts".into(), json!(&ctx.previous_attempts));
        data.insert(
            "agent_failure_output".into(),
            json!(&ctx.agent_failure_output),
        );
        data.insert("changes_made_so_far".into(), json!(&ctx.changes_made_so_far));
        data.insert("is_retry".into(), json!(is_retry));

        // Agent info
        data.insert("agent_name".into(), json!(agent.name()));

        // Agent-specific flags
        data.insert("is_rust".into(), json!(matches!(agent, Agent::Rex)));
        data.insert("is_frontend".into(), json!(matches!(agent, Agent::Blaze)));
        data.insert("is_infra".into(), json!(matches!(agent, Agent::Bolt)));
        data.insert("is_security".into(), json!(matches!(agent, Agent::Cipher)));

        serde_json::Value::Object(data)
    }

    /// Build a generic prompt when no template is available.
    fn build_generic_prompt(agent: Agent, ctx: &RemediationContext) -> String {
        use std::fmt::Write as _;

        let mut prompt = String::new();

        let _ = writeln!(prompt, "# CI Fix - {}\n", agent.name().to_uppercase());
        let _ = writeln!(
            prompt,
            "You are {}, a specialist agent. A CI workflow has failed.\n",
            agent.name()
        );

        prompt.push_str("## Failure Details\n");
        if let Some(failure) = &ctx.failure {
            let _ = writeln!(prompt, "- **Workflow**: {}", failure.workflow_name);
            if let Some(job) = &failure.job_name {
                let _ = writeln!(prompt, "- **Job**: {job}");
            }
            let _ = writeln!(prompt, "- **Branch**: {}", failure.branch);
            let _ = writeln!(prompt, "- **Commit**: {}", failure.head_sha);
        }

        if let Some(failure_type) = &ctx.failure_type {
            let _ = writeln!(prompt, "- **Failure Type**: {}", failure_type.short_name());
            let _ = writeln!(
                prompt,
                "- **Suggested Approach**: {}",
                failure_type.fix_approach()
            );
        }

        prompt.push_str("\n## Failure Logs\n```\n");
        // Truncate logs if too long (UTF-8 safe)
        let max_log_bytes = 10000;
        if ctx.workflow_logs.len() > max_log_bytes {
            // Find a safe UTF-8 boundary to truncate at
            let truncated = truncate_utf8_safe(&ctx.workflow_logs, max_log_bytes);
            let _ = writeln!(
                prompt,
                "{}...\n(truncated, {} total bytes)",
                truncated,
                ctx.workflow_logs.len()
            );
        } else {
            prompt.push_str(&ctx.workflow_logs);
        }
        prompt.push_str("\n```\n\n");

        prompt.push_str("## Instructions\n");
        prompt.push_str("1. Analyze the failure logs above\n");
        prompt.push_str("2. Identify the root cause\n");
        prompt.push_str("3. Apply a minimal, targeted fix\n");
        prompt.push_str("4. Ensure CI passes after your fix\n");
        prompt.push_str("5. Create a PR or push to the existing branch\n\n");
        prompt.push_str("Focus only on fixing the CI failure. Do not refactor unrelated code.\n");

        prompt
    }

    /// Build the `CodeRun` YAML manifest.
    fn build_coderun_yaml(&self, agent: Agent, ctx: &RemediationContext, prompt: &str) -> String {
        let failure = ctx.failure.as_ref();

        let name_prefix = format!("healer-ci-{}-", agent.name());
        let workflow_run_id = failure.map_or(0, |f| f.workflow_run_id);
        let branch = failure.map_or("unknown", |f| f.branch.as_str());
        let failure_type = ctx
            .failure_type
            .as_ref()
            .map_or("general", CiFailureType::short_name);
        let task_id = ctx.task_id();

        // Build labels
        let mut labels: BTreeMap<&str, String> = BTreeMap::new();
        labels.insert("app.kubernetes.io/name", "healer".to_string());
        labels.insert("healer/agent", agent.name().to_string());
        labels.insert("healer/failure-type", failure_type.to_string());
        labels.insert("healer/workflow-run-id", workflow_run_id.to_string());
        labels.insert("healer/branch", sanitize_label(branch));
        labels.insert("healer/task-id", task_id.clone());

        // Escape prompt for YAML
        let escaped_prompt = prompt
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");

        let yaml = format!(
            r#"apiVersion: cto.5dlabs.io/v1
kind: CodeRun
metadata:
  generateName: {name_prefix}
  namespace: {namespace}
  labels:
{labels_yaml}
spec:
  githubApp: {github_app}
  cli: {cli}
  model: {model}
  repositoryUrl: https://github.com/{repository}
  prompt: "{escaped_prompt}"
  env:
    - name: HEALER_TASK_ID
      value: "{task_id}"
    - name: FAILURE_TYPE
      value: "{failure_type}"
    - name: WORKFLOW_RUN_ID
      value: "{workflow_run_id}"
    - name: TARGET_BRANCH
      value: "{branch}"
"#,
            name_prefix = name_prefix,
            namespace = self.namespace,
            labels_yaml = labels
                .iter()
                .map(|(k, v)| format!("    {k}: \"{v}\""))
                .collect::<Vec<_>>()
                .join("\n"),
            github_app = agent.github_app(),
            cli = self.config.cli,
            model = self.config.model,
            repository = self.repository,
            escaped_prompt = escaped_prompt,
            task_id = task_id,
            failure_type = failure_type,
            workflow_run_id = workflow_run_id,
            branch = branch,
        );

        yaml
    }

    /// Apply the `CodeRun` YAML and return the created name.
    fn apply_coderun(yaml: &str) -> Result<String> {
        use std::io::Write;
        use std::process::Stdio;

        // Apply with kubectl using stdin to avoid temp files
        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-", "-o", "jsonpath={.metadata.name}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        // Write YAML to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(yaml.as_bytes())
                .context("Failed to write YAML to kubectl stdin")?;
        }

        let output = child.wait_with_output().context("Failed to wait for kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("kubectl apply failed: {stderr}");
        }

        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(name)
    }
}

/// Truncate a UTF-8 string at a safe byte boundary.
///
/// This avoids panicking when slicing in the middle of a multi-byte character.
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Find the last valid UTF-8 character boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    &s[..end]
}

/// Sanitize a string for use as a Kubernetes label value.
fn sanitize_label(value: &str) -> String {
    // Labels must be <= 63 characters, alphanumeric, dashes, underscores, dots
    let sanitized: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Truncate to 63 chars
    if sanitized.len() > 63 {
        sanitized[..63].to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_label() {
        assert_eq!(sanitize_label("main"), "main");
        assert_eq!(sanitize_label("feat/new-feature"), "feat-new-feature");
        assert_eq!(sanitize_label("fix#123"), "fix-123");

        // Test truncation
        let long = "a".repeat(100);
        assert_eq!(sanitize_label(&long).len(), 63);
    }

    #[test]
    fn test_spawner_creation() {
        let config = RemediationConfig::default();
        let spawner = CodeRunSpawner::new(config, "cto", "5dlabs/cto");
        assert!(spawner.is_ok());
    }
}
