//! Validate command - AI-powered cluster health check.

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use tracing::info;

use crate::validation;

/// Run AI-powered cluster validation.
#[derive(Args)]
pub struct ValidateCommand {
    /// Path to kubeconfig file.
    #[arg(long, env = "KUBECONFIG")]
    kubeconfig: PathBuf,

    /// Attempt automatic remediation of issues.
    #[arg(long, default_value = "false")]
    remediate: bool,

    /// Output report as JSON.
    #[arg(long, default_value = "false")]
    json: bool,
}

impl ValidateCommand {
    /// Run the validation command.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails to run.
    pub async fn run(&self) -> Result<()> {
        info!("🤖 Starting AI-powered cluster validation...");
        info!("   Kubeconfig: {}", self.kubeconfig.display());
        info!(
            "   Remediation: {}",
            if self.remediate {
                "enabled"
            } else {
                "disabled"
            }
        );

        let report = validation::run_validation(&self.kubeconfig, self.remediate).await?;

        if self.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            report.print_summary();
        }

        if report.all_passed() {
            Ok(())
        } else {
            anyhow::bail!("Validation found {} issues", report.failed_count());
        }
    }
}
