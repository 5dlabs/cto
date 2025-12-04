//! Code-based remediation for play issues.

use anyhow::{Context, Result};
use std::process::Command;

use super::batch::PlayBatch;
use super::types::{Diagnosis, DiagnosisCategory, DiagnosisContext, Issue, PrContext};

/// Engine for gathering context and spawning fix `CodeRuns`.
pub struct RemediationEngine {
    /// Namespace for `CodeRuns`
    namespace: String,
}

impl RemediationEngine {
    /// Create a new remediation engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            namespace: "cto".to_string(),
        }
    }

    /// Create with a custom namespace.
    #[must_use]
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    /// Gather context for diagnosing an issue.
    ///
    /// # Errors
    ///
    /// Returns an error if gathering context fails.
    pub fn gather_context(&self, issue: &Issue, batch: &PlayBatch) -> Result<DiagnosisContext> {
        let mut context = DiagnosisContext::default();

        let task_id = issue.task_id();
        let task = batch.get_task(task_id);

        // Get logs from Loki if we have a workflow/coderun name
        if let Some(task) = task {
            if let Some(ref coderun) = task.active_coderun {
                context.logs = self.fetch_pod_logs(coderun).unwrap_or_default();
            }
            if let Some(ref workflow) = task.workflow_name {
                if context.logs.is_empty() {
                    context.logs = self.fetch_workflow_logs(workflow).unwrap_or_default();
                }
            }
        }

        // Get PR context if we have a PR number
        if let Some(task) = task {
            if let Some(pr_number) = task.pr_number {
                context.pr_state = self.fetch_pr_context(&batch.repository, pr_number).ok();
            }
        }

        // Get agent output from the issue description
        context.agent_output = issue.description();

        Ok(context)
    }

    /// Fetch pod logs from Loki.
    fn fetch_pod_logs(&self, pod_name: &str) -> Result<String> {
        // Use kubectl logs as fallback (Loki query would be better)
        let output = Command::new("kubectl")
            .args([
                "logs",
                pod_name,
                "-n",
                &self.namespace,
                "--tail=100",
            ])
            .output()
            .context("Failed to fetch pod logs")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Try with -l selector for jobs/workflows
            let output = Command::new("kubectl")
                .args([
                    "logs",
                    "-l",
                    &format!("app.kubernetes.io/name={pod_name}"),
                    "-n",
                    &self.namespace,
                    "--tail=100",
                ])
                .output()
                .context("Failed to fetch pod logs by label")?;

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }

    /// Fetch workflow logs.
    fn fetch_workflow_logs(&self, workflow_name: &str) -> Result<String> {
        let output = Command::new("kubectl")
            .args([
                "logs",
                "-l",
                &format!("workflows.argoproj.io/workflow={workflow_name}"),
                "-n",
                &self.namespace,
                "--tail=100",
                "--all-containers",
            ])
            .output()
            .context("Failed to fetch workflow logs")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Fetch PR context from GitHub.
    #[allow(clippy::unused_self)]
    fn fetch_pr_context(&self, repository: &str, pr_number: u32) -> Result<PrContext> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                repository,
                "--json",
                "number,state,mergeable,statusCheckRollup",
            ])
            .output()
            .context("Failed to fetch PR context")?;

        if !output.status.success() {
            anyhow::bail!("gh pr view failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse PR JSON")?;

        Ok(PrContext {
            number: pr_number,
            state: json["state"].as_str().unwrap_or("unknown").to_string(),
            mergeable: json["mergeable"].as_str() == Some("MERGEABLE"),
            checks_status: json["statusCheckRollup"]
                .as_array()
                .map_or_else(|| "unknown".to_string(), |arr| {
                    let passed = arr.iter().filter(|c| c["conclusion"] == "SUCCESS").count();
                    let total = arr.len();
                    format!("{passed}/{total} passed")
                }),
        })
    }

    /// Diagnose the root cause of an issue.
    ///
    /// # Errors
    ///
    /// This function currently always succeeds but may fail in the future.
    pub fn diagnose(&self, context: &DiagnosisContext) -> Result<Diagnosis> {
        // Simple pattern-based diagnosis
        let logs = &context.logs;
        let agent_output = &context.agent_output;

        // Check for common patterns
        let (category, summary, suggested_fix) = if logs.contains("merge conflict")
            || logs.contains("CONFLICT")
            || agent_output.contains("conflict")
        {
            (
                DiagnosisCategory::GitIssue,
                "Git merge conflict detected".to_string(),
                "Add pre-commit rebase step or conflict resolution logic".to_string(),
            )
        } else if logs.contains("authentication")
            || logs.contains("401")
            || logs.contains("403")
        {
            (
                DiagnosisCategory::InfraIssue,
                "Authentication/authorization error".to_string(),
                "Check credentials and permissions".to_string(),
            )
        } else if logs.contains("timeout") || logs.contains("timed out") {
            (
                DiagnosisCategory::InfraIssue,
                "Operation timed out".to_string(),
                "Increase timeout or optimize operation".to_string(),
            )
        } else if logs.contains("import") && logs.contains("error") {
            (
                DiagnosisCategory::CodeIssue,
                "Import/dependency error".to_string(),
                "Add missing imports or dependencies".to_string(),
            )
        } else if logs.contains("test") && logs.contains("fail") {
            (
                DiagnosisCategory::CodeIssue,
                "Test failure".to_string(),
                "Fix failing tests or update test expectations".to_string(),
            )
        } else if logs.contains("lint") || logs.contains("clippy") {
            (
                DiagnosisCategory::CodeIssue,
                "Lint/style error".to_string(),
                "Fix lint errors in the code".to_string(),
            )
        } else {
            (
                DiagnosisCategory::Unknown,
                "Unknown issue - needs investigation".to_string(),
                "Investigate logs and agent output for root cause".to_string(),
            )
        };

        Ok(Diagnosis {
            summary,
            category,
            suggested_fix,
            relevant_files: vec![], // Would need code analysis to populate
        })
    }

    /// Spawn a Healer `CodeRun` to fix the diagnosed issue.
    ///
    /// The `task_id` is used to label the `CodeRun` for later cancellation.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl fails to apply the `CodeRun` resource.
    pub fn spawn_fix_coderun(&self, task_id: &str, diagnosis: &Diagnosis) -> Result<String> {
        let coderun_name = format!(
            "healer-fix-{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")
        );

        // Build the prompt for the fix
        let prompt = format!(
            r"You are Healer, fixing an issue in the CTO platform.

## Diagnosis
Summary: {}
Category: {:?}
Suggested Fix: {}

## Instructions
1. Investigate the root cause based on the diagnosis
2. Write a fix for the issue (code, config, or prompt change)
3. Create a PR with the fix
4. Include clear commit message and PR description

Do NOT just restart or retry - fix the underlying issue in code.
",
            diagnosis.summary, diagnosis.category, diagnosis.suggested_fix
        );

        // Create the CodeRun YAML
        // Labels include task-id for cancellation via CancelRemediation command
        let coderun_yaml = format!(
            r"apiVersion: cto.5dlabs.io/v1alpha1
kind: CodeRun
metadata:
  name: {}
  namespace: {}
  labels:
    app.kubernetes.io/name: healer
    app.kubernetes.io/component: remediation
    task-id: '{}'
spec:
  cli: claude
  model: sonnet
  githubApp: cto-healer
  repository: 5dlabs/cto
  workingDir: /workspace
  prompt: |
{}
",
            coderun_name,
            self.namespace,
            task_id,
            prompt.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
        );

        // Apply the CodeRun
        let mut child = Command::new("kubectl")
            .args(["apply", "-f", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn kubectl apply")?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(coderun_yaml.as_bytes())
                .context("Failed to write CodeRun YAML")?;
        }

        let status = child.wait().context("Failed to wait for kubectl apply")?;

        if !status.success() {
            anyhow::bail!("kubectl apply failed for CodeRun");
        }

        Ok(coderun_name)
    }
}

impl Default for RemediationEngine {
    fn default() -> Self {
        Self::new()
    }
}
