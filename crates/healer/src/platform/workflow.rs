//! Argo Workflow remediation handler.
//!
//! Handles workflow-specific alerts:
//! - Failed workflow steps
//! - Stuck workflows
//! - High failure rates

use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::loki::LokiClient;

use super::types::{
    AlertmanagerAlert, PlatformAlert, PlatformIssue, PlatformIssueType, RemediationStatus,
    RemediationTarget, TrackedRemediation,
};

/// Handler for Argo Workflow alerts.
pub struct WorkflowRemediator {
    /// Loki client for fetching workflow logs
    loki: LokiClient,
    /// Namespace for spawning CodeRuns
    namespace: String,
    /// Repository for platform remediations
    #[allow(dead_code)]
    repository: String,
    /// Active remediations
    active: Arc<RwLock<HashMap<String, TrackedRemediation>>>,
    /// Maximum concurrent workflow remediations
    max_concurrent: usize,
    /// Dedup window in minutes
    dedup_window_mins: u64,
}

impl WorkflowRemediator {
    /// Create a new workflow remediator.
    #[must_use]
    pub fn new(namespace: &str, repository: &str) -> Self {
        Self {
            loki: LokiClient::with_defaults(),
            namespace: namespace.to_string(),
            repository: repository.to_string(),
            active: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent: 3,
            dedup_window_mins: 30,
        }
    }

    /// Process a workflow alert.
    ///
    /// # Errors
    /// Returns an error if processing fails.
    pub async fn process_alert(&self, alert: AlertmanagerAlert) -> Result<Option<String>> {
        // Only process firing alerts
        if !alert.is_firing() {
            info!(
                "Workflow alert {} resolved, fingerprint={}",
                alert.name(),
                alert.fingerprint
            );
            self.mark_resolved(&alert.fingerprint).await;
            return Ok(None);
        }

        info!(
            "Processing workflow alert: {} (severity={}, pod={:?})",
            alert.name(),
            alert.severity(),
            alert.pod()
        );

        // Check for duplicate
        if self.is_duplicate(&alert.fingerprint).await {
            debug!("Skipping duplicate workflow alert: {}", alert.fingerprint);
            return Ok(None);
        }

        // Check concurrent limit
        if self.active_count().await >= self.max_concurrent {
            warn!(
                "Max concurrent workflow remediations ({}) reached",
                self.max_concurrent
            );
            return Ok(None);
        }

        // Determine issue type
        let issue_type = PlatformIssueType::from_alert_name(alert.name());

        // For workflow failures, we need to understand what went wrong
        // This might be a task code issue or an infrastructure issue
        let (logs, diagnosis) = self.analyze_workflow_failure(&alert).await?;

        // Convert to platform alert
        let platform_alert: PlatformAlert = alert.clone().into();

        // Determine remediation target based on diagnosis
        let target = self.determine_target(&diagnosis);

        // Create issue
        let issue = PlatformIssue {
            issue_type,
            alert: platform_alert,
            logs,
            diagnosis: Some(diagnosis),
            target,
        };

        // Track remediation
        let tracked = TrackedRemediation::new(issue.clone());
        self.track_remediation(tracked).await;

        // Spawn CodeRun
        let coderun_name = self.spawn_workflow_remediation(&issue).await?;

        // Update tracking
        self.update_coderun_name(&alert.fingerprint, &coderun_name)
            .await;

        info!(
            "Spawned workflow remediation CodeRun: {} for alert {}",
            coderun_name,
            alert.name()
        );

        Ok(Some(coderun_name))
    }

    /// Analyze a workflow failure to determine root cause.
    async fn analyze_workflow_failure(
        &self,
        alert: &AlertmanagerAlert,
    ) -> Result<(String, String)> {
        let namespace = alert.namespace().unwrap_or("cto");
        let pod = alert.pod();

        // Fetch logs
        let logs = if let Some(pod_name) = pod {
            self.fetch_pod_logs(namespace, pod_name).await?
        } else {
            // Try to find workflow pods from the alert
            let workflow_pattern = self.extract_workflow_pattern(alert);
            self.fetch_workflow_logs(namespace, &workflow_pattern).await?
        };

        // Analyze logs to determine diagnosis
        let diagnosis = self.diagnose_from_logs(&logs, alert);

        Ok((logs, diagnosis))
    }

    /// Extract workflow name pattern from alert.
    #[allow(clippy::unused_self)]
    fn extract_workflow_pattern(&self, alert: &AlertmanagerAlert) -> String {
        // Try to extract from labels
        if let Some(pod) = alert.pod() {
            // Pod names follow pattern: workflow-name-step-random
            // Extract the base workflow name
            let parts: Vec<&str> = pod.split('-').collect();
            if parts.len() >= 2 {
                return format!("{}.*", parts[0..2].join("-"));
            }
        }

        // Fallback to generic pattern
        "play-.*".to_string()
    }

    /// Fetch logs for a specific pod.
    #[allow(clippy::format_push_string)]
    async fn fetch_pod_logs(&self, namespace: &str, pod_name: &str) -> Result<String> {
        let start = Utc::now() - chrono::Duration::minutes(30);
        let end = Utc::now();

        let entries = self
            .loki
            .query_pod_logs(namespace, pod_name, start, end, 1000)
            .await
            .context("Failed to query pod logs")?;

        let mut output = String::new();
        for entry in entries {
            let time = entry.timestamp.format("%H:%M:%S%.3f");
            output.push_str(&format!("[{time}] {}\n", entry.line));
        }

        Ok(output)
    }

    /// Fetch logs for workflow pods.
    #[allow(clippy::format_push_string)]
    async fn fetch_workflow_logs(&self, namespace: &str, pattern: &str) -> Result<String> {
        let start = Utc::now() - chrono::Duration::minutes(30);
        let end = Utc::now();

        let query = format!(r#"{{namespace="{namespace}", pod=~"{pattern}"}}"#);
        let entries = self
            .loki
            .query_logs(&query, start, end, 1000)
            .await
            .context("Failed to query workflow logs")?;

        let mut output = String::new();
        for entry in entries {
            let time = entry.timestamp.format("%H:%M:%S%.3f");
            let pod = entry.labels.get("pod").map_or("unknown", String::as_str);
            output.push_str(&format!("[{time}] [{pod}] {}\n", entry.line));
        }

        Ok(output)
    }

    /// Diagnose the issue from logs.
    #[allow(clippy::unused_self)]
    fn diagnose_from_logs(&self, logs: &str, alert: &AlertmanagerAlert) -> String {
        let logs_lower = logs.to_lowercase();

        // Check for common patterns
        if logs_lower.contains("error[e") || logs_lower.contains("cargo build") {
            return "Rust compilation error in agent code. Check for missing imports, type errors, or borrow checker issues.".to_string();
        }

        if logs_lower.contains("clippy") {
            return "Clippy lint errors. Agent needs to fix code style issues.".to_string();
        }

        if logs_lower.contains("test result: failed") || logs_lower.contains("test failed") {
            return "Test failures in agent code. Check test output for specific failures.".to_string();
        }

        if logs_lower.contains("git") && (logs_lower.contains("conflict") || logs_lower.contains("merge")) {
            return "Git merge conflict. Agent needs to resolve conflicting changes.".to_string();
        }

        if logs_lower.contains("timeout") || logs_lower.contains("deadline exceeded") {
            return "Operation timed out. This may indicate a stuck agent or slow external service.".to_string();
        }

        if logs_lower.contains("oom") || logs_lower.contains("out of memory") {
            return "Out of memory error. Consider increasing resource limits for the workflow.".to_string();
        }

        if logs_lower.contains("permission denied") || logs_lower.contains("unauthorized") {
            return "Permission/authentication error. Check GitHub App credentials and RBAC.".to_string();
        }

        if logs_lower.contains("docker") && logs_lower.contains("error") {
            return "Docker build error. Check Dockerfile and build context.".to_string();
        }

        // Check alert-specific patterns
        match alert.name() {
            "ArgoWorkflowStepStuck" => {
                return "Workflow step is stuck. The agent may be unresponsive or waiting for external input.".to_string();
            }
            "ArgoWorkflowPendingTooLong" => {
                return "Workflow pod cannot be scheduled. Check resource availability and image pull status.".to_string();
            }
            "ArgoWorkflowHighFailureRate" => {
                return "High workflow failure rate detected. This indicates a systemic issue requiring investigation.".to_string();
            }
            _ => {}
        }

        // Default diagnosis
        "Unknown failure. Review logs for specific error messages.".to_string()
    }

    /// Determine remediation target based on diagnosis.
    #[allow(clippy::unused_self)]
    fn determine_target(&self, diagnosis: &str) -> RemediationTarget {
        let diagnosis_lower = diagnosis.to_lowercase();

        // Rust code issues -> Rex
        if diagnosis_lower.contains("rust")
            || diagnosis_lower.contains("clippy")
            || diagnosis_lower.contains("cargo")
            || diagnosis_lower.contains("compilation")
        {
            return RemediationTarget::for_agent("rex");
        }

        // Git issues -> Atlas
        if diagnosis_lower.contains("git") || diagnosis_lower.contains("merge") {
            return RemediationTarget::for_agent("atlas");
        }

        // Infrastructure issues -> Bolt
        if diagnosis_lower.contains("docker")
            || diagnosis_lower.contains("kubernetes")
            || diagnosis_lower.contains("resource")
            || diagnosis_lower.contains("oom")
            || diagnosis_lower.contains("permission")
        {
            return RemediationTarget::for_agent("bolt");
        }

        // Default to Bolt for workflow issues
        RemediationTarget::for_agent("bolt")
    }

    /// Spawn a workflow remediation CodeRun.
    async fn spawn_workflow_remediation(&self, issue: &PlatformIssue) -> Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let prompt = self.build_workflow_prompt(issue);
        let yaml = self.build_coderun_yaml(issue, &prompt);

        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-", "-o", "jsonpath={.metadata.name}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(yaml.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl apply failed: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Build workflow remediation prompt.
    #[allow(clippy::unused_self, clippy::format_push_string)]
    fn build_workflow_prompt(&self, issue: &PlatformIssue) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!(
            "# Argo Workflow Remediation: {}\n\n",
            issue.alert.name
        ));

        prompt.push_str(&format!(
            "You are {}, a specialist agent. An Argo Workflow has failed.\n\n",
            issue.target.agent.to_uppercase()
        ));

        prompt.push_str("## Alert Details\n\n");
        prompt.push_str(&format!("- **Alert**: {}\n", issue.alert.name));
        prompt.push_str(&format!("- **Severity**: {}\n", issue.alert.severity));
        prompt.push_str(&format!("- **Namespace**: {}\n", issue.alert.namespace));
        if let Some(pod) = &issue.alert.pod {
            prompt.push_str(&format!("- **Pod**: {pod}\n"));
        }

        if let Some(diagnosis) = &issue.diagnosis {
            prompt.push_str(&format!("\n## Diagnosis\n\n{diagnosis}\n"));
        }

        prompt.push_str("\n## Workflow Logs\n\n```\n");
        let max_len = 10000;
        if issue.logs.len() > max_len {
            prompt.push_str(&issue.logs[..max_len]);
            prompt.push_str("\n... (truncated)\n");
        } else {
            prompt.push_str(&issue.logs);
        }
        prompt.push_str("```\n\n");

        prompt.push_str("## Your Task\n\n");
        prompt.push_str("1. Analyze the workflow failure and diagnosis\n");
        prompt.push_str("2. Identify the root cause in the CTO codebase\n");
        prompt.push_str("3. Implement a fix to prevent this failure\n");
        prompt.push_str("4. Ensure tests pass\n");
        prompt.push_str("5. Create a PR with your fix\n\n");

        prompt.push_str("## Relevant Paths\n\n");
        prompt.push_str("- Workflow templates: `infra/gitops/manifests/argo-workflows/`\n");
        prompt.push_str("- Controller code: `crates/controller/`\n");
        prompt.push_str("- Agent templates: `scripts/templates/`\n");
        prompt.push_str("- Helm charts: `infra/charts/cto/`\n");

        prompt
    }

    /// Build CodeRun YAML.
    fn build_coderun_yaml(&self, issue: &PlatformIssue, prompt: &str) -> String {
        let escaped_prompt = prompt
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");

        format!(
            r#"apiVersion: cto.5dlabs.io/v1
kind: CodeRun
metadata:
  generateName: healer-workflow-{agent}-
  namespace: {namespace}
  labels:
    app.kubernetes.io/name: healer
    healer/type: workflow
    healer/alert: "{alert_name}"
    healer/fingerprint: "{fingerprint}"
spec:
  githubApp: {github_app}
  cli: {cli}
  model: {model}
  repositoryUrl: https://github.com/{repository}
  prompt: "{escaped_prompt}"
  env:
    - name: HEALER_ALERT_NAME
      value: "{alert_name}"
    - name: HEALER_FINGERPRINT
      value: "{fingerprint}"
"#,
            namespace = self.namespace,
            agent = issue.target.agent,
            alert_name = issue.alert.name,
            fingerprint = issue.alert.fingerprint,
            github_app = issue.target.github_app,
            cli = issue.target.cli,
            model = issue.target.model,
            repository = issue.target.repository,
            escaped_prompt = escaped_prompt,
        )
    }

    /// Check for duplicate.
    async fn is_duplicate(&self, fingerprint: &str) -> bool {
        let active = self.active.read().await;
        if let Some(tracked) = active.get(fingerprint) {
            let elapsed = Utc::now() - tracked.started_at;
            #[allow(clippy::cast_possible_wrap)]
            if elapsed.num_minutes() < self.dedup_window_mins as i64 {
                return true;
            }
        }
        false
    }

    /// Track a remediation.
    async fn track_remediation(&self, remediation: TrackedRemediation) {
        let mut active = self.active.write().await;
        active.insert(remediation.fingerprint.clone(), remediation);
    }

    /// Update CodeRun name.
    async fn update_coderun_name(&self, fingerprint: &str, name: &str) {
        let mut active = self.active.write().await;
        if let Some(tracked) = active.get_mut(fingerprint) {
            tracked.coderun_name = Some(name.to_string());
            tracked.status = RemediationStatus::InProgress;
        }
    }

    /// Mark resolved.
    async fn mark_resolved(&self, fingerprint: &str) {
        let mut active = self.active.write().await;
        if let Some(tracked) = active.get_mut(fingerprint) {
            tracked.status = RemediationStatus::Succeeded;
            tracked.completed_at = Some(Utc::now());
        }
    }

    /// Count active.
    async fn active_count(&self) -> usize {
        let active = self.active.read().await;
        active
            .values()
            .filter(|t| {
                matches!(
                    t.status,
                    RemediationStatus::Pending | RemediationStatus::InProgress
                )
            })
            .count()
    }
}
