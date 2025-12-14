//! Claude-powered validation agent.
//!
//! This module spawns the Claude CLI with access to kubectl and runs
//! the validation prompt against the cluster.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use super::report::ValidationReport;

/// The validation agent that runs Claude against the cluster.
pub struct ValidationAgent {
    kubeconfig: PathBuf,
    timeout: Duration,
}

impl ValidationAgent {
    /// Create a new validation agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the kubeconfig path doesn't exist.
    pub fn new(kubeconfig: &Path) -> Result<Self> {
        if !kubeconfig.exists() {
            anyhow::bail!("Kubeconfig not found: {}", kubeconfig.display());
        }

        Ok(Self {
            kubeconfig: kubeconfig.to_path_buf(),
            timeout: Duration::from_secs(600), // 10 minutes max
        })
    }

    /// Set the timeout for the validation run.
    #[must_use]
    #[allow(dead_code)] // Will be used when we add CLI flag for custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run the validation agent.
    ///
    /// # Errors
    ///
    /// Returns an error if Claude fails to run or times out.
    pub async fn run(&self, remediate: bool) -> Result<ValidationReport> {
        info!("Running Claude validation agent...");

        let prompt = self.build_prompt(remediate);
        debug!("Validation prompt:\n{}", prompt);

        // Run Claude CLI with the prompt
        let output = self.run_claude(&prompt).await?;

        // Parse the output into a validation report
        let report = ValidationReport::parse_from_output(&output)?;

        Ok(report)
    }

    /// Build the validation prompt for Claude.
    fn build_prompt(&self, remediate: bool) -> String {
        let remediation_note = if remediate {
            "If issues found, attempt fixes and document results."
        } else {
            "Do NOT attempt fixes, just report issues."
        };

        format!(
            r#"Validate this Kubernetes cluster. KUBECONFIG={kubeconfig}

Run these checks:
1. kubectl get nodes (all Ready?)
2. kubectl get pods -A --no-headers | grep -v Running | grep -v Completed (any stuck pods?)
3. kubectl get sc (default storage class?)
4. kubectl get pvc -A (all Bound?)
5. kubectl get applications -n argocd 2>/dev/null | head -10 (ArgoCD apps healthy?)

{remediation_note}

Output EXACTLY this format:
=== VALIDATION REPORT ===
Cluster: [context name]
Timestamp: [now]

CHECKS:
- node_health: [PASS/FAIL] - [brief detail]
- pods_status: [PASS/FAIL] - [brief detail]
- storage: [PASS/FAIL] - [brief detail]
- argocd: [PASS/FAIL] - [brief detail]

ISSUES:
- [list issues or "None"]

REMEDIATIONS ATTEMPTED:
- [list or "None"]

SUMMARY: [X/4 checks passed]
=== END REPORT ==="#,
            kubeconfig = self.kubeconfig.display(),
            remediation_note = remediation_note
        )
    }

    /// Find the Claude CLI binary.
    ///
    /// Checks multiple locations since Claude may be installed as:
    /// - A shell alias (not visible to `which`)
    /// - In ~/.claude/local/claude (Anthropic's default)
    /// - In PATH via npm global install
    async fn find_claude_cli(&self) -> Result<String> {
        // Common locations for Claude CLI
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let candidates = [
            home.join(".claude/local/claude"),
            home.join(".local/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
        ];

        // Check known paths first
        for path in &candidates {
            if path.exists() {
                return Ok(path.to_string_lossy().to_string());
            }
        }

        // Fall back to which
        let which_result = Command::new("which")
            .arg("claude")
            .output()
            .await
            .context("Failed to run 'which claude'")?;

        if which_result.status.success() {
            let path = String::from_utf8_lossy(&which_result.stdout)
                .trim()
                .to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }

        anyhow::bail!(
            "Claude CLI not found. Checked: {candidates:?}. \
             Please install it or ensure it's in your PATH."
        )
    }

    /// Run Claude CLI with the given prompt.
    async fn run_claude(&self, prompt: &str) -> Result<String> {
        info!("Spawning Claude CLI...");

        // Try to find Claude CLI in common locations
        let claude_path = self.find_claude_cli().await?;
        debug!("Found Claude at: {}", claude_path);

        // Spawn Claude with the prompt as argument
        let child = Command::new(&claude_path)
            .arg("--print") // Print output to stdout (non-interactive)
            .arg("--dangerously-skip-permissions") // Skip permission prompts in automation
            .arg(prompt)
            .env("KUBECONFIG", &self.kubeconfig)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn Claude CLI")?;

        // Wait for completion with timeout
        let result = timeout(self.timeout, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    warn!("Claude exited with non-zero status: {}", output.status);
                    warn!("stderr: {}", stderr);
                }

                debug!("Claude output length: {} bytes", stdout.len());
                Ok(stdout.to_string())
            }
            Ok(Err(e)) => Err(e).context("Claude process failed"),
            Err(_) => {
                anyhow::bail!(
                    "Claude validation timed out after {} seconds",
                    self.timeout.as_secs()
                );
            }
        }
    }
}
