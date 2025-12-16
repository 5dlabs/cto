//! Platform alert handling and remediation spawning.

use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::loki::LokiClient;
use crate::prometheus::PrometheusClient;

use super::types::{
    AlertmanagerAlert, PlatformAlert, PlatformIssue, PlatformIssueType, RemediationStatus,
    RemediationTarget, TrackedRemediation,
};

/// Handler for platform alerts.
pub struct PlatformAlertHandler {
    /// Loki client for fetching logs
    loki: LokiClient,
    /// Prometheus client for metrics
    prometheus: PrometheusClient,
    /// Namespace for spawning CodeRuns
    namespace: String,
    /// Repository for platform remediations
    #[allow(dead_code)]
    repository: String,
    /// Active remediations (fingerprint -> remediation)
    active: Arc<RwLock<HashMap<String, TrackedRemediation>>>,
    /// Maximum concurrent remediations
    max_concurrent: usize,
    /// Deduplication window in minutes
    dedup_window_mins: u64,
}

impl PlatformAlertHandler {
    /// Create a new platform alert handler.
    #[must_use]
    pub fn new(namespace: &str, repository: &str) -> Self {
        Self {
            loki: LokiClient::with_defaults(),
            prometheus: PrometheusClient::with_defaults(),
            namespace: namespace.to_string(),
            repository: repository.to_string(),
            active: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent: 5,
            dedup_window_mins: 30,
        }
    }

    /// Process an alert from Alertmanager.
    ///
    /// # Errors
    /// Returns an error if processing fails.
    pub async fn process_alert(&self, alert: AlertmanagerAlert) -> Result<Option<String>> {
        // Only process firing alerts
        if !alert.is_firing() {
            info!(
                "Alert {} resolved, fingerprint={}",
                alert.name(),
                alert.fingerprint
            );
            self.mark_resolved(&alert.fingerprint).await;
            return Ok(None);
        }

        info!(
            "Processing platform alert: {} (severity={}, component={:?})",
            alert.name(),
            alert.severity(),
            alert.component()
        );

        // Check for duplicate
        if self.is_duplicate(&alert.fingerprint).await {
            debug!("Skipping duplicate alert: {}", alert.fingerprint);
            return Ok(None);
        }

        // Check concurrent limit
        if self.active_count().await >= self.max_concurrent {
            warn!(
                "Max concurrent remediations ({}) reached, skipping alert: {}",
                self.max_concurrent,
                alert.name()
            );
            return Ok(None);
        }

        // Convert to platform alert
        let platform_alert: PlatformAlert = alert.clone().into();

        // Determine issue type
        let issue_type = PlatformIssueType::from_alert_name(&platform_alert.name);

        // Gather context (logs)
        let logs = self
            .gather_logs(&platform_alert)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to gather logs: {e}");
                String::new()
            });

        // Create remediation target
        let agent = issue_type.remediation_agent();
        let target = RemediationTarget::for_agent(agent);

        // Create issue
        let issue = PlatformIssue {
            issue_type,
            alert: platform_alert,
            logs,
            diagnosis: None,
            target,
        };

        // Track the remediation
        let tracked = TrackedRemediation::new(issue.clone());
        self.track_remediation(tracked).await;

        // Spawn the CodeRun
        let coderun_name = self.spawn_remediation(&issue).await?;

        // Update tracking with CodeRun name
        self.update_coderun_name(&alert.fingerprint, &coderun_name)
            .await;

        info!(
            "Spawned platform remediation CodeRun: {} for alert {}",
            coderun_name,
            alert.name()
        );

        Ok(Some(coderun_name))
    }

    /// Gather logs for an alert.
    #[allow(clippy::format_push_string)]
    async fn gather_logs(&self, alert: &PlatformAlert) -> Result<String> {
        let namespace = &alert.namespace;

        // Determine what to query based on alert type
        let query = if let Some(pod) = &alert.pod {
            // Query specific pod
            format!(r#"{{namespace="{namespace}", pod="{pod}"}}"#)
        } else {
            // Query by component
            let component = &alert.component;
            format!(r#"{{namespace="{namespace}", pod=~"cto-{component}.*"}}"#)
        };

        let start = alert.started_at - chrono::Duration::minutes(10);
        let end = Utc::now();

        let entries = self
            .loki
            .query_logs(&query, start, end, 500)
            .await
            .context("Failed to query Loki")?;

        // Format logs
        let mut output = String::new();
        for entry in entries {
            let time = entry.timestamp.format("%H:%M:%S%.3f");
            output.push_str(&format!("[{time}] {}\n", entry.line));
        }

        Ok(output)
    }

    /// Spawn a remediation CodeRun.
    async fn spawn_remediation(&self, issue: &PlatformIssue) -> Result<String> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Build the prompt
        let prompt = self.build_remediation_prompt(issue);

        // Build CodeRun YAML
        let yaml = self.build_coderun_yaml(issue, &prompt);

        // Apply via kubectl
        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-", "-o", "jsonpath={.metadata.name}"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(yaml.as_bytes())
                .context("Failed to write YAML")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for kubectl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl apply failed: {stderr}");
        }

        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(name)
    }

    /// Build remediation prompt for an issue.
    #[allow(clippy::unused_self, clippy::format_push_string)]
    fn build_remediation_prompt(&self, issue: &PlatformIssue) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!(
            "# CTO Platform Remediation: {}\n\n",
            issue.alert.name
        ));

        prompt.push_str(&format!(
            "You are {}, a platform specialist. ",
            issue.target.agent.to_uppercase()
        ));
        prompt.push_str("A critical platform alert has been triggered.\n\n");

        prompt.push_str("## Alert Details\n\n");
        prompt.push_str(&format!("- **Alert**: {}\n", issue.alert.name));
        prompt.push_str(&format!("- **Severity**: {}\n", issue.alert.severity));
        prompt.push_str(&format!("- **Component**: {}\n", issue.alert.component));
        prompt.push_str(&format!("- **Namespace**: {}\n", issue.alert.namespace));
        if let Some(pod) = &issue.alert.pod {
            prompt.push_str(&format!("- **Pod**: {pod}\n"));
        }
        prompt.push_str(&format!("- **Started**: {}\n", issue.alert.started_at));
        prompt.push_str(&format!("\n**Summary**: {}\n", issue.alert.summary));
        prompt.push_str(&format!("\n**Description**:\n{}\n", issue.alert.description));

        prompt.push_str("\n## Recent Logs\n\n```\n");
        if issue.logs.is_empty() {
            prompt.push_str("No logs available\n");
        } else {
            // Truncate if too long
            let max_len = 8000;
            if issue.logs.len() > max_len {
                prompt.push_str(&issue.logs[..max_len]);
                prompt.push_str("\n... (truncated)\n");
            } else {
                prompt.push_str(&issue.logs);
            }
        }
        prompt.push_str("```\n\n");

        prompt.push_str("## Your Task\n\n");
        prompt.push_str("1. Analyze the alert and logs to identify the root cause\n");
        prompt.push_str("2. Check the relevant code in the `crates/` and `infra/` directories\n");
        prompt.push_str("3. Implement a fix that resolves the underlying issue\n");
        prompt.push_str("4. Ensure all tests pass and Clippy is happy\n");
        prompt.push_str("5. Create a PR with your fix\n\n");

        prompt.push_str("## Guidelines\n\n");
        prompt.push_str("- Focus on the root cause, not just symptoms\n");
        prompt.push_str("- Make minimal, targeted changes\n");
        prompt.push_str("- If this is an infrastructure issue, check `infra/charts/cto/`\n");
        prompt.push_str("- If this is a code issue, check the relevant crate in `crates/`\n");
        prompt.push_str("- Run `cargo test` and `cargo clippy -- -D warnings` before committing\n");

        prompt
    }

    /// Build CodeRun YAML.
    fn build_coderun_yaml(&self, issue: &PlatformIssue, prompt: &str) -> String {
        let escaped_prompt = prompt
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");

        format!(
            r#"apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  generateName: healer-platform-{agent}-
  namespace: {namespace}
  labels:
    app.kubernetes.io/name: healer
    healer/type: platform
    healer/alert: "{alert_name}"
    healer/component: "{component}"
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
    - name: HEALER_COMPONENT
      value: "{component}"
    - name: HEALER_FINGERPRINT
      value: "{fingerprint}"
"#,
            namespace = self.namespace,
            agent = issue.target.agent,
            alert_name = issue.alert.name,
            component = issue.alert.component,
            fingerprint = issue.alert.fingerprint,
            github_app = issue.target.github_app,
            cli = issue.target.cli,
            model = issue.target.model,
            repository = issue.target.repository,
            escaped_prompt = escaped_prompt,
        )
    }

    /// Check if an alert is a duplicate (already being remediated).
    async fn is_duplicate(&self, fingerprint: &str) -> bool {
        let active = self.active.read().await;
        if let Some(tracked) = active.get(fingerprint) {
            // Check if within dedup window
            let elapsed = Utc::now() - tracked.started_at;
            #[allow(clippy::cast_possible_wrap)]
            if elapsed.num_minutes() < self.dedup_window_mins as i64 {
                return true;
            }
        }
        false
    }

    /// Track a new remediation.
    async fn track_remediation(&self, remediation: TrackedRemediation) {
        let mut active = self.active.write().await;
        active.insert(remediation.fingerprint.clone(), remediation);
    }

    /// Update CodeRun name for a tracked remediation.
    async fn update_coderun_name(&self, fingerprint: &str, coderun_name: &str) {
        let mut active = self.active.write().await;
        if let Some(tracked) = active.get_mut(fingerprint) {
            tracked.coderun_name = Some(coderun_name.to_string());
            tracked.status = RemediationStatus::InProgress;
        }
    }

    /// Mark a remediation as resolved.
    async fn mark_resolved(&self, fingerprint: &str) {
        let mut active = self.active.write().await;
        if let Some(tracked) = active.get_mut(fingerprint) {
            tracked.status = RemediationStatus::Succeeded;
            tracked.completed_at = Some(Utc::now());
        }
    }

    /// Get count of active remediations.
    async fn active_count(&self) -> usize {
        let active = self.active.read().await;
        active
            .values()
            .filter(|t| matches!(t.status, RemediationStatus::Pending | RemediationStatus::InProgress))
            .count()
    }

    /// Get all tracked remediations.
    pub async fn get_remediations(&self) -> Vec<TrackedRemediation> {
        let active = self.active.read().await;
        active.values().cloned().collect()
    }

    /// Clean up old remediations.
    pub async fn cleanup_old(&self, max_age_hours: u64) {
        let mut active = self.active.write().await;
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        active.retain(|_, v| v.started_at > cutoff);
    }

    /// Get Prometheus client reference.
    #[must_use]
    pub fn prometheus(&self) -> &PrometheusClient {
        &self.prometheus
    }

    /// Get Loki client reference.
    #[must_use]
    pub fn loki(&self) -> &LokiClient {
        &self.loki
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let handler = PlatformAlertHandler::new("cto", "5dlabs/cto");
        assert_eq!(handler.namespace, "cto");
        assert_eq!(handler.repository, "5dlabs/cto");
    }
}
