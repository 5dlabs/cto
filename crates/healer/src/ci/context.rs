//! Context gathering for CI remediation.
//!
//! Collects diagnostic information from multiple sources:
//! - GitHub: workflow logs, PR state, changed files
//! - `ArgoCD`: application status, sync state
//! - Loki: recent error logs
//! - Kubernetes: pod state, events

use anyhow::{Context as _, Result};
use std::process::Command;
use tracing::{debug, warn};

use super::types::{
    ArgoCdStatus, ChangedFile, CiFailure, PodState, PullRequest, RemediationContext,
};

/// Context gatherer for CI remediation.
pub struct ContextGatherer {
    /// Repository for GitHub operations
    repository: String,
    /// Namespace for Kubernetes operations
    namespace: String,
    /// `ArgoCD` server URL (optional)
    argocd_url: Option<String>,
    /// Loki URL (optional)
    loki_url: Option<String>,
}

impl ContextGatherer {
    /// Create a new context gatherer.
    #[must_use]
    pub fn new(repository: &str, namespace: &str) -> Self {
        Self {
            repository: repository.to_string(),
            namespace: namespace.to_string(),
            argocd_url: None,
            loki_url: None,
        }
    }

    /// Set `ArgoCD` URL.
    #[must_use]
    pub fn with_argocd(mut self, url: &str) -> Self {
        self.argocd_url = Some(url.to_string());
        self
    }

    /// Set Loki URL.
    #[must_use]
    pub fn with_loki(mut self, url: &str) -> Self {
        self.loki_url = Some(url.to_string());
        self
    }

    /// Gather full context for a CI failure.
    ///
    /// # Errors
    ///
    /// Returns an error if critical context gathering fails. Most individual
    /// context sources are fault-tolerant and will log warnings on failure.
    pub fn gather(&self, failure: &CiFailure) -> Result<RemediationContext> {
        let mut ctx = RemediationContext {
            failure: Some(failure.clone()),
            ..Default::default()
        };

        // Gather workflow logs (most important for diagnosis)
        match self.fetch_workflow_logs(failure.workflow_run_id) {
            Ok(logs) => ctx.workflow_logs = logs,
            Err(e) => warn!("Failed to fetch workflow logs: {e}"),
        }

        // Gather PR information if this is a PR-triggered build
        if let Ok(Some(pr)) = self.fetch_pr_for_branch(&failure.branch) {
            ctx.pr = Some(pr);
        }

        // Gather changed files
        match self.fetch_changed_files(&failure.head_sha) {
            Ok(files) => ctx.changed_files = files,
            Err(e) => warn!("Failed to fetch changed files: {e}"),
        }

        // Gather ArgoCD status if URL is configured
        if self.argocd_url.is_some() {
            match self.fetch_argocd_status("cto-controller") {
                Ok(status) => ctx.argocd_status = Some(status),
                Err(e) => debug!("Failed to fetch ArgoCD status: {e}"),
            }
        }

        // Gather recent Loki logs if URL is configured
        if self.loki_url.is_some() {
            match self.fetch_loki_errors(&failure.branch) {
                Ok(logs) => ctx.recent_logs = logs,
                Err(e) => debug!("Failed to fetch Loki logs: {e}"),
            }
        }

        // Gather pod state from Kubernetes
        match self.fetch_pod_state(&failure.workflow_name) {
            Ok(state) => ctx.pod_state = Some(state),
            Err(e) => debug!("Failed to fetch pod state: {e}"),
        }

        Ok(ctx)
    }

    /// Fetch workflow logs using gh CLI.
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI command fails.
    pub fn fetch_workflow_logs(&self, run_id: u64) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "run",
                "view",
                &run_id.to_string(),
                "--repo",
                &self.repository,
                "--log-failed",
            ])
            .output()
            .context("Failed to execute gh run view")?;

        if !output.status.success() {
            // Try getting any logs if --log-failed doesn't work
            let output = Command::new("gh")
                .args([
                    "run",
                    "view",
                    &run_id.to_string(),
                    "--repo",
                    &self.repository,
                    "--log",
                ])
                .output()
                .context("Failed to execute gh run view --log")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("gh run view failed: {stderr}");
            }

            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Fetch PR for a branch using gh CLI.
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI command fails to execute.
    #[allow(clippy::cast_possible_truncation)]
    pub fn fetch_pr_for_branch(&self, branch: &str) -> Result<Option<PullRequest>> {
        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--repo",
                &self.repository,
                "--head",
                branch,
                "--json",
                "number,title,state,headRefName,baseRefName,mergeable,statusCheckRollup,url",
                "--limit",
                "1",
            ])
            .output()
            .context("Failed to execute gh pr list")?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let prs: Vec<serde_json::Value> =
            serde_json::from_str(&stdout).unwrap_or_default();

        if prs.is_empty() {
            return Ok(None);
        }

        let pr = &prs[0];
        Ok(Some(PullRequest {
            number: pr["number"].as_u64().unwrap_or(0) as u32,
            title: pr["title"].as_str().unwrap_or("").to_string(),
            state: pr["state"].as_str().unwrap_or("").to_string(),
            head_ref: pr["headRefName"].as_str().unwrap_or("").to_string(),
            base_ref: pr["baseRefName"].as_str().unwrap_or("").to_string(),
            mergeable: pr["mergeable"].as_str().map(|s| s == "MERGEABLE"),
            checks_status: pr["statusCheckRollup"]
                .as_array()
                .map_or_else(
                    || "unknown".to_string(),
                    |checks| {
                        let failed = checks
                            .iter()
                            .filter(|c| c["conclusion"].as_str() == Some("FAILURE"))
                            .count();
                        if failed > 0 {
                            format!("{failed} checks failing")
                        } else {
                            "all passing".to_string()
                        }
                    },
                ),
            html_url: pr["url"].as_str().unwrap_or("").to_string(),
        }))
    }

    /// Fetch changed files for a commit using gh CLI.
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI command fails.
    pub fn fetch_changed_files(&self, sha: &str) -> Result<Vec<ChangedFile>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!("/repos/{}/commits/{}", self.repository, sha),
                "--jq",
                ".files[] | {filename: .filename, status: .status, additions: .additions, deletions: .deletions}",
            ])
            .output()
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh api failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();

        // Parse JSONL output (one JSON object per line)
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(file) = serde_json::from_str::<ChangedFile>(line) {
                files.push(file);
            }
        }

        Ok(files)
    }

    /// Fetch `ArgoCD` application status using argocd CLI or API.
    ///
    /// # Errors
    ///
    /// Returns an error if the kubectl command fails.
    pub fn fetch_argocd_status(&self, app_name: &str) -> Result<ArgoCdStatus> {
        // Try using kubectl to get ArgoCD app status
        let output = Command::new("kubectl")
            .args([
                "get",
                "application",
                app_name,
                "-n",
                "argocd",
                "-o",
                "jsonpath={.status.health.status},{.status.sync.status}",
            ])
            .output()
            .context("Failed to execute kubectl get application")?;

        if !output.status.success() {
            anyhow::bail!("kubectl get application failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.split(',').collect();

        Ok(ArgoCdStatus {
            health: (*parts.first().unwrap_or(&"Unknown")).to_string(),
            sync: (*parts.get(1).unwrap_or(&"Unknown")).to_string(),
            unhealthy_resources: Vec::new(), // Would need additional query
        })
    }

    /// Fetch recent error logs from Loki.
    ///
    /// # Errors
    ///
    /// Returns an error if log fetching fails completely.
    pub fn fetch_loki_errors(&self, branch: &str) -> Result<String> {
        // This would typically use HTTP to query Loki
        // For now, use kubectl logs as a fallback
        let output = Command::new("kubectl")
            .args([
                "logs",
                "-n",
                &self.namespace,
                "-l",
                &format!("branch={branch}"),
                "--tail=100",
                "--since=30m",
            ])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            }
            _ => Ok(String::new()), // Empty string if we can't get logs
        }
    }

    /// Fetch pod state from Kubernetes.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl commands fail.
    pub fn fetch_pod_state(&self, workflow_name: &str) -> Result<PodState> {
        // Get pod names
        let output = Command::new("kubectl")
            .args([
                "get",
                "pods",
                "-n",
                &self.namespace,
                "-l",
                &format!("workflows.argoproj.io/workflow={workflow_name}"),
                "-o",
                "jsonpath={.items[*].metadata.name}",
            ])
            .output()
            .context("Failed to get pods")?;

        let names: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(String::from)
            .collect();

        // Get recent events
        let output = Command::new("kubectl")
            .args([
                "get",
                "events",
                "-n",
                &self.namespace,
                "--field-selector",
                &format!("involvedObject.name={workflow_name}"),
                "-o",
                "jsonpath={.items[*].message}",
            ])
            .output()
            .context("Failed to get events")?;

        let events: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .split('\n')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Ok(PodState { names, events })
    }

    /// Fetch commits since a given SHA (for retry context).
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI command fails.
    pub fn fetch_commits_since(&self, original_sha: &str, branch: &str) -> Result<Vec<serde_json::Value>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "/repos/{}/compare/{}...{}",
                    self.repository, original_sha, branch
                ),
                "--jq",
                ".commits[] | {sha: .sha, message: .commit.message, author: .commit.author.name}",
            ])
            .output()
            .context("Failed to fetch commits")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            if let Ok(commit) = serde_json::from_str(line) {
                commits.push(commit);
            }
        }

        Ok(commits)
    }

    /// Fetch `CodeRun` logs for retry context.
    ///
    /// # Errors
    ///
    /// Returns an error if kubectl logs command fails.
    pub fn fetch_coderun_logs(&self, coderun_name: &str) -> Result<String> {
        let output = Command::new("kubectl")
            .args([
                "logs",
                "-n",
                &self.namespace,
                "-l",
                &format!("coderun.cto.5dlabs.io/name={coderun_name}"),
                "--tail=200",
            ])
            .output()
            .context("Failed to fetch CodeRun logs")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("kubectl logs failed: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Post a comment on a PR.
    ///
    /// # Errors
    ///
    /// Returns an error if the `gh` CLI command fails.
    pub fn post_pr_comment(&self, pr_number: u32, comment: &str) -> Result<()> {
        let output = Command::new("gh")
            .args([
                "pr",
                "comment",
                &pr_number.to_string(),
                "--repo",
                &self.repository,
                "--body",
                comment,
            ])
            .output()
            .context("Failed to post PR comment")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr comment failed: {stderr}");
        }

        Ok(())
    }
}

/// Helper to determine if changed files are mostly a certain type.
pub trait ChangedFilesAnalysis {
    /// Check if mostly Rust files.
    fn mostly_rust(&self) -> bool;
    /// Check if mostly frontend files.
    fn mostly_frontend(&self) -> bool;
    /// Check if mostly infrastructure files.
    fn mostly_infra(&self) -> bool;
}

impl ChangedFilesAnalysis for Vec<ChangedFile> {
    fn mostly_rust(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let rust_count = self.iter().filter(|f| f.is_rust()).count();
        rust_count > self.len() / 2
    }

    fn mostly_frontend(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let fe_count = self.iter().filter(|f| f.is_frontend()).count();
        fe_count > self.len() / 2
    }

    fn mostly_infra(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let infra_count = self.iter().filter(|f| f.is_infra()).count();
        infra_count > self.len() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changed_files_analysis() {
        let rust_files = vec![
            ChangedFile {
                filename: "src/main.rs".to_string(),
                status: "modified".to_string(),
                additions: 10,
                deletions: 5,
            },
            ChangedFile {
                filename: "Cargo.toml".to_string(),
                status: "modified".to_string(),
                additions: 1,
                deletions: 1,
            },
        ];
        assert!(rust_files.mostly_rust());
        assert!(!rust_files.mostly_frontend());

        let fe_files = vec![
            ChangedFile {
                filename: "src/App.tsx".to_string(),
                status: "modified".to_string(),
                additions: 20,
                deletions: 10,
            },
            ChangedFile {
                filename: "package.json".to_string(),
                status: "modified".to_string(),
                additions: 5,
                deletions: 2,
            },
        ];
        assert!(fe_files.mostly_frontend());
        assert!(!fe_files.mostly_rust());

        let empty: Vec<ChangedFile> = vec![];
        assert!(!empty.mostly_rust());
        assert!(!empty.mostly_frontend());
    }
}
