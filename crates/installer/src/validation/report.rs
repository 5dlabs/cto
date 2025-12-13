//! Validation report parsing and formatting.

use std::fmt;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The result of a validation check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Skip,
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "✅ PASS"),
            Self::Fail => write!(f, "❌ FAIL"),
            Self::Skip => write!(f, "⏭️  SKIP"),
        }
    }
}

/// A single validation check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub details: String,
    pub remediation_attempted: Option<String>,
    pub remediation_result: Option<String>,
}

/// An issue found during validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub component: String,
    pub message: String,
    pub severity: IssueSeverity,
}

/// Issue severity level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

/// The complete validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub cluster: String,
    pub timestamp: String,
    pub checks: Vec<CheckResult>,
    pub issues: Vec<Issue>,
    pub remediations: Vec<String>,
    pub raw_output: String,
}

impl ValidationReport {
    /// Parse a validation report from Claude's output.
    ///
    /// # Errors
    ///
    /// Returns an error if the output cannot be parsed.
    pub fn parse_from_output(output: &str) -> Result<Self> {
        // Find the report section
        let report_start = output
            .find("=== VALIDATION REPORT ===")
            .context("Could not find validation report in output")?;

        let report_end = output.find("=== END REPORT ===").unwrap_or(output.len());

        let report_text = &output[report_start..report_end];

        // Parse cluster name
        let cluster =
            Self::extract_field(report_text, "Cluster:").unwrap_or_else(|| "unknown".to_string());

        // Parse timestamp
        let timestamp = Self::extract_field(report_text, "Timestamp:")
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        // Parse checks
        let checks = Self::parse_checks(report_text);

        // Parse issues
        let issues = Self::parse_issues(report_text);

        // Parse remediations
        let remediations = Self::parse_remediations(report_text);

        Ok(Self {
            cluster,
            timestamp,
            checks,
            issues,
            remediations,
            raw_output: output.to_string(),
        })
    }

    /// Extract a field value from the report text.
    fn extract_field(text: &str, field: &str) -> Option<String> {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(field) {
                return Some(rest.trim().to_string());
            }
        }
        None
    }

    /// Parse check results from the report.
    fn parse_checks(text: &str) -> Vec<CheckResult> {
        let mut checks = Vec::new();
        let mut in_checks = false;

        for line in text.lines() {
            if line.starts_with("CHECKS:") {
                in_checks = true;
                continue;
            }
            if in_checks && line.starts_with("ISSUES:") {
                break;
            }
            if in_checks && line.starts_with("- ") {
                if let Some(check) = Self::parse_check_line(line) {
                    checks.push(check);
                }
            }
        }

        checks
    }

    /// Parse a single check line.
    fn parse_check_line(line: &str) -> Option<CheckResult> {
        // Format: "- check_name: [PASS/FAIL] - details"
        let line = line.trim_start_matches("- ");

        let colon_pos = line.find(':')?;
        let name = line[..colon_pos].trim().to_string();

        let rest = line[colon_pos + 1..].trim();

        let status = if rest.contains("PASS") {
            CheckStatus::Pass
        } else if rest.contains("FAIL") {
            CheckStatus::Fail
        } else {
            CheckStatus::Skip
        };

        let details = rest.split(" - ").nth(1).unwrap_or("").trim().to_string();

        Some(CheckResult {
            name,
            status,
            details,
            remediation_attempted: None,
            remediation_result: None,
        })
    }

    /// Parse issues from the report.
    fn parse_issues(text: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        let mut in_issues = false;

        for line in text.lines() {
            if line.starts_with("ISSUES:") {
                in_issues = true;
                continue;
            }
            if in_issues && line.starts_with("REMEDIATIONS") {
                break;
            }
            if in_issues && line.starts_with("- ") {
                let message = line.trim_start_matches("- ").to_string();
                if !message.is_empty() {
                    issues.push(Issue {
                        component: "cluster".to_string(),
                        message,
                        severity: IssueSeverity::Warning,
                    });
                }
            }
        }

        issues
    }

    /// Parse remediations from the report.
    fn parse_remediations(text: &str) -> Vec<String> {
        let mut remediations = Vec::new();
        let mut in_remediations = false;

        for line in text.lines() {
            if line.contains("REMEDIATIONS") {
                in_remediations = true;
                continue;
            }
            if in_remediations && line.starts_with("SUMMARY:") {
                break;
            }
            if in_remediations && line.starts_with("- ") {
                let remediation = line.trim_start_matches("- ").to_string();
                if !remediation.is_empty() {
                    remediations.push(remediation);
                }
            }
        }

        remediations
    }

    /// Check if all validation checks passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.status == CheckStatus::Pass)
    }

    /// Get the total number of checks.
    #[must_use]
    pub fn total_checks(&self) -> usize {
        self.checks.len()
    }

    /// Get the number of passed checks.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == CheckStatus::Pass)
            .count()
    }

    /// Get the number of failed checks.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail)
            .count()
    }

    /// Get all issues.
    #[must_use]
    #[allow(dead_code)] // Will be used for detailed reporting
    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Print a summary of the validation report.
    pub fn print_summary(&self) {
        println!();
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║              CLUSTER VALIDATION REPORT                       ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ Cluster: {:<52} ║", self.cluster);
        println!("║ Time:    {:<52} ║", self.timestamp);
        println!("╠══════════════════════════════════════════════════════════════╣");

        for check in &self.checks {
            let status_icon = match check.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Fail => "❌",
                CheckStatus::Skip => "⏭️ ",
            };
            println!(
                "║ {} {:<20} {:<35} ║",
                status_icon,
                check.name,
                truncate(&check.details, 35)
            );
        }

        println!("╠══════════════════════════════════════════════════════════════╣");
        println!(
            "║ SUMMARY: {}/{} checks passed                                  ║",
            self.passed_count(),
            self.total_checks()
        );

        if self.all_passed() {
            println!("║ ✅ CLUSTER VALIDATION PASSED                                  ║");
        } else {
            println!("║ ⚠️  CLUSTER VALIDATION FOUND ISSUES                           ║");
        }

        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
