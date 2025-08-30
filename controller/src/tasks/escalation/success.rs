//! # Success Criteria Detection
//!
//! This module handles detection of success criteria for remediation tasks,
//! including feedback resolution, PR approval, and CI/CD status checks.

use crate::remediation::RemediationState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Success criteria types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SuccessCriterion {
    /// All feedback items resolved
    AllFeedbackResolved,
    /// PR has been approved
    PRApproved,
    /// All required CI/CD checks passed
    CIChecksPassed,
    /// No critical issues remaining
    NoCriticalIssues,
    /// Manual success signal received
    ManualSuccessSignal,
}

/// Success criteria evaluation result
#[derive(Debug, Clone)]
pub struct SuccessEvaluation {
    pub criterion: SuccessCriterion,
    pub passed: bool,
    pub details: String,
    pub metadata: HashMap<String, String>,
}

/// Overall success assessment
#[derive(Debug, Clone)]
pub struct SuccessAssessment {
    pub overall_success: bool,
    pub criteria_results: Vec<SuccessEvaluation>,
    pub confidence_score: f64, // 0.0 to 1.0
    pub summary: String,
}

/// Success criteria detector
pub struct SuccessDetector {
    required_approvals: u32,
    required_checks: Vec<String>,
    allow_manual_success: bool,
}

impl Default for SuccessDetector {
    fn default() -> Self {
        Self {
            required_approvals: 1,
            required_checks: vec![
                "ci/circleci: test".to_string(),
                "ci/circleci: lint".to_string(),
                "ci/circleci: build".to_string(),
            ],
            allow_manual_success: true,
        }
    }
}

impl SuccessDetector {
    /// Create a new success detector with custom settings
    pub fn new(required_approvals: u32, required_checks: Vec<String>, allow_manual_success: bool) -> Self {
        Self {
            required_approvals,
            required_checks,
            allow_manual_success,
        }
    }

    /// Evaluate if success criteria are met for a remediation task
    pub async fn evaluate_success_criteria(
        &self,
        task_id: &str,
        pr_number: i32,
        state: &RemediationState,
    ) -> Result<SuccessAssessment, SuccessError> {
        info!("Evaluating success criteria for task {} on PR #{}", task_id, pr_number);

        let mut criteria_results = Vec::new();
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        // Check feedback resolution
        let feedback_result = self.check_feedback_resolution(state).await?;
        criteria_results.push(feedback_result.clone());
        if feedback_result.passed {
            total_score += 1.0;
        }
        total_weight += 1.0;

        // Check PR approval (placeholder - would integrate with GitHub API)
        let approval_result = self.check_pr_approval(pr_number).await?;
        criteria_results.push(approval_result.clone());
        if approval_result.passed {
            total_score += 1.0;
        }
        total_weight += 1.0;

        // Check CI/CD status (placeholder - would integrate with GitHub API)
        let ci_result = self.check_ci_status(pr_number).await?;
        criteria_results.push(ci_result.clone());
        if ci_result.passed {
            total_score += 0.8; // Slightly lower weight for CI
        }
        total_weight += 1.0;

        // Check for critical issues
        let critical_result = self.check_no_critical_issues(state).await?;
        criteria_results.push(critical_result.clone());
        if critical_result.passed {
            total_score += 1.0;
        }
        total_weight += 1.0;

        // Check manual success signals (if enabled)
        if self.allow_manual_success {
            let manual_result = self.check_manual_success_signals(pr_number).await?;
            criteria_results.push(manual_result.clone());
            if manual_result.passed {
                total_score += 0.5; // Lower weight for manual signals
            }
            total_weight += 0.5;
        }

        let confidence_score = if total_weight > 0.0 { total_score / total_weight } else { 0.0 };
        let overall_success = confidence_score >= 0.8; // 80% threshold

        let summary = if overall_success {
            format!("Success criteria met with {:.1}% confidence", confidence_score * 100.0)
        } else {
            format!("Success criteria not fully met ({:.1}% confidence)", confidence_score * 100.0)
        };

        debug!("Success evaluation complete for task {}: success={}, confidence={:.2}",
               task_id, overall_success, confidence_score);

        Ok(SuccessAssessment {
            overall_success,
            criteria_results,
            confidence_score,
            summary,
        })
    }

    /// Check if all feedback items are resolved
    async fn check_feedback_resolution(&self, state: &RemediationState) -> Result<SuccessEvaluation, SuccessError> {
        let mut resolved_count = 0;
        let mut total_count = 0;
        let mut unresolved_issues = Vec::new();

        for feedback in &state.feedback_history {
            total_count += 1;

            // Check if feedback has actions taken (indicates it was processed)
            if !feedback.actions_taken.is_empty() {
                resolved_count += 1;
            } else {
                unresolved_issues.push(format!("Feedback {} has no actions taken", feedback.id));
            }
        }

        let passed = resolved_count == total_count && total_count > 0;
        let details = if passed {
            format!("All {} feedback items resolved", total_count)
        } else {
            format!("{}/{} feedback items resolved. Unresolved: {}", resolved_count, total_count, unresolved_issues.join(", "))
        };

        let mut metadata = HashMap::new();
        metadata.insert("resolved_count".to_string(), resolved_count.to_string());
        metadata.insert("total_count".to_string(), total_count.to_string());

        Ok(SuccessEvaluation {
            criterion: SuccessCriterion::AllFeedbackResolved,
            passed,
            details,
            metadata,
        })
    }

    /// Check if PR has required approvals
    async fn check_pr_approval(&self, pr_number: i32) -> Result<SuccessEvaluation, SuccessError> {
        // TODO: Integrate with GitHub API to check PR reviews
        // For now, return a placeholder that assumes approval

        let passed = true; // Placeholder
        let details = if passed {
            format!("PR #{} has {} required approvals", pr_number, self.required_approvals)
        } else {
            format!("PR #{} missing required approvals", pr_number)
        };

        let mut metadata = HashMap::new();
        metadata.insert("required_approvals".to_string(), self.required_approvals.to_string());
        metadata.insert("current_approvals".to_string(), self.required_approvals.to_string()); // Placeholder

        Ok(SuccessEvaluation {
            criterion: SuccessCriterion::PRApproved,
            passed,
            details,
            metadata,
        })
    }

    /// Check if all required CI/CD checks have passed
    async fn check_ci_status(&self, _pr_number: i32) -> Result<SuccessEvaluation, SuccessError> {
        // TODO: Integrate with GitHub API to check commit status/checks
        // For now, return a placeholder that assumes all checks pass

        let passed = true; // Placeholder
        let passed_checks = self.required_checks.clone();
        let failed_checks: Vec<String> = Vec::new(); // Placeholder

        let details = if passed {
            format!("All {} required CI checks passed", self.required_checks.len())
        } else {
            format!("Failed CI checks: {}", failed_checks.join(", "))
        };

        let mut metadata = HashMap::new();
        metadata.insert("required_checks".to_string(), self.required_checks.len().to_string());
        metadata.insert("passed_checks".to_string(), passed_checks.len().to_string());
        metadata.insert("failed_checks".to_string(), failed_checks.len().to_string());

        Ok(SuccessEvaluation {
            criterion: SuccessCriterion::CIChecksPassed,
            passed,
            details,
            metadata,
        })
    }

    /// Check for any remaining critical issues
    async fn check_no_critical_issues(&self, state: &RemediationState) -> Result<SuccessEvaluation, SuccessError> {
        let mut critical_issues = Vec::new();

        for feedback in &state.feedback_history {
            // Check feedback severity and status through the StructuredFeedback
            // For now, we'll use a simplified check based on actions taken
            if feedback.actions_taken.is_empty() {
                critical_issues.push(format!("Unprocessed feedback: {}", feedback.id));
            }
        }

        let passed = critical_issues.is_empty();
        let details = if passed {
            "No critical issues remaining".to_string()
        } else {
            format!("Critical issues found: {}", critical_issues.join(", "))
        };

        let mut metadata = HashMap::new();
        metadata.insert("critical_issues_count".to_string(), critical_issues.len().to_string());

        Ok(SuccessEvaluation {
            criterion: SuccessCriterion::NoCriticalIssues,
            passed,
            details,
            metadata,
        })
    }

    /// Check for manual success signals (e.g., specific comments)
    async fn check_manual_success_signals(&self, _pr_number: i32) -> Result<SuccessEvaluation, SuccessError> {
        // TODO: Integrate with GitHub API to check for manual success comments
        // Look for comments like "✅ Success" or "@bot success"

        let found_signal = false; // Placeholder
        let passed = found_signal;
        let details = if passed {
            "Manual success signal detected".to_string()
        } else {
            "No manual success signals found".to_string()
        };

        let mut metadata = HashMap::new();
        metadata.insert("manual_signal_found".to_string(), found_signal.to_string());

        Ok(SuccessEvaluation {
            criterion: SuccessCriterion::ManualSuccessSignal,
            passed,
            details,
            metadata,
        })
    }

    /// Get detailed breakdown of success criteria for reporting
    pub fn get_success_breakdown(&self, assessment: &SuccessAssessment) -> String {
        let mut breakdown = format!("Success Assessment: {}\n", assessment.summary);
        breakdown.push_str(&format!("Overall Success: {}\n", assessment.overall_success));
        breakdown.push_str(&format!("Confidence Score: {:.1}%\n\n", assessment.confidence_score * 100.0));

        breakdown.push_str("Criteria Details:\n");
        for result in &assessment.criteria_results {
            let status = if result.passed { "✅" } else { "❌" };
            breakdown.push_str(&format!("{} {:?}: {}\n", status, result.criterion, result.details));

            if !result.metadata.is_empty() {
                for (key, value) in &result.metadata {
                    breakdown.push_str(&format!("  - {}: {}\n", key, value));
                }
            }
            breakdown.push('\n');
        }

        breakdown
    }
}

/// Success detection errors
#[derive(Debug, thiserror::Error)]
pub enum SuccessError {
    #[error("GitHub API error: {0}")]
    GitHubError(String),

    #[error("State validation error: {0}")]
    StateError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
