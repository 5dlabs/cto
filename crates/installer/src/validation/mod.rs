//! AI-powered cluster validation and remediation.
//!
//! This module provides automated cluster health checking and remediation
//! using Claude as the AI agent with access to kubectl and cluster tools.

pub mod agent;
pub mod checks;
pub mod remediation;
pub mod report;
pub mod workloads;

use std::path::Path;

use anyhow::Result;
use tracing::info;

pub use agent::ValidationAgent;
pub use report::ValidationReport;

/// Run the full AI-powered validation suite.
///
/// This spawns Claude with access to kubectl and cluster tools to:
/// 1. Run health checks on all cluster components
/// 2. Deploy test workloads to validate functionality
/// 3. Attempt automatic remediation of issues
/// 4. Generate a comprehensive validation report
///
/// # Errors
///
/// Returns an error if the validation agent fails to run or times out.
pub async fn run_validation(kubeconfig: &Path, remediate: bool) -> Result<ValidationReport> {
    info!("🤖 Starting AI-powered cluster validation...");

    let agent = ValidationAgent::new(kubeconfig)?;
    let report = agent.run(remediate).await?;

    if report.all_passed() {
        info!(
            "✅ Cluster validation passed! All {} checks green.",
            report.total_checks()
        );
    } else {
        info!(
            "⚠️  Cluster validation complete: {}/{} checks passed, {} issues found",
            report.passed_count(),
            report.total_checks(),
            report.failed_count()
        );
    }

    Ok(report)
}
