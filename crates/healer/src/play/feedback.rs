//! Feedback loop for improving agent prompts based on evaluation results.
//!
//! When probe-based evaluations consistently fail, this module generates
//! suggested improvements to agent prompts and creates tracking issues.
//!
//! Reference: Agent-Skills-for-Context-Engineering/skills/evaluation

// Allow raw string hashes - some strings contain quotes that need escaping
#![allow(clippy::needless_raw_string_hashes)]
// Allow format! with push_str - clearer than write! for building issue bodies
#![allow(clippy::format_push_string)]
// Allow Result wrapper for consistency with other functions
#![allow(clippy::unnecessary_wraps)]

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::github::GitHubClient;

use super::types::{EvaluationResults, ProbeResult, ProbeType};

/// Configuration for the feedback engine.
#[derive(Debug, Clone)]
pub struct FeedbackConfig {
    /// Number of consecutive failures before creating an issue
    pub failure_threshold: u32,
    /// Whether to automatically create issues
    pub auto_create_issues: bool,
    /// Repository for issue creation (owner/repo format)
    pub repository: Option<String>,
    /// Labels to add to feedback issues
    pub issue_labels: Vec<String>,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            auto_create_issues: true,
            repository: None,
            issue_labels: vec![
                "feedback".to_string(),
                "context-engineering".to_string(),
                "auto-generated".to_string(),
            ],
        }
    }
}

/// A suggested prompt improvement based on failed probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSuggestion {
    /// Category of the suggestion
    pub category: SuggestionCategory,
    /// Human-readable title
    pub title: String,
    /// Detailed description of what to change
    pub description: String,
    /// The probe type that triggered this suggestion
    pub triggered_by: ProbeType,
    /// Example text to add to the prompt
    pub example_addition: Option<String>,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// Category of prompt improvement suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Add explicit file tracking instructions
    FileTracking,
    /// Improve context placement (lost-in-middle fix)
    ContextPlacement,
    /// Add decision logging requirements
    DecisionLogging,
    /// Add continuation/planning section
    TaskPlanning,
    /// Add acceptance criteria verification
    AcceptanceVerification,
    /// General improvement
    General,
}

impl std::fmt::Display for SuggestionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTracking => write!(f, "File Tracking"),
            Self::ContextPlacement => write!(f, "Context Placement"),
            Self::DecisionLogging => write!(f, "Decision Logging"),
            Self::TaskPlanning => write!(f, "Task Planning"),
            Self::AcceptanceVerification => write!(f, "Acceptance Verification"),
            Self::General => write!(f, "General"),
        }
    }
}

/// Tracks failure history for feedback decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailureHistory {
    /// Consecutive failures by probe type
    pub consecutive_failures: HashMap<String, u32>,
    /// Total failures by probe type
    pub total_failures: HashMap<String, u32>,
    /// Issues already created for specific patterns
    pub created_issues: Vec<String>,
}

/// Feedback generated from evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackResult {
    /// Play ID this feedback is for
    pub play_id: String,
    /// Suggested improvements
    pub suggestions: Vec<PromptSuggestion>,
    /// Whether an issue was created
    pub issue_created: Option<String>,
    /// When feedback was generated
    pub generated_at: String,
}

/// Engine for generating feedback from evaluation results.
pub struct FeedbackEngine {
    config: FeedbackConfig,
    github: Option<GitHubClient>,
    /// Failure history by play_id
    history: HashMap<String, FailureHistory>,
}

impl FeedbackEngine {
    /// Create a new feedback engine.
    #[must_use]
    pub fn new(config: FeedbackConfig) -> Self {
        let github = config.repository.as_ref().map(|repo| {
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() == 2 {
                GitHubClient::new(parts[0], parts[1])
            } else {
                GitHubClient::new("5dlabs", repo)
            }
        });

        Self {
            config,
            github,
            history: HashMap::new(),
        }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(FeedbackConfig::default())
    }

    /// Process evaluation results and generate feedback if needed.
    pub fn process_results(
        &mut self,
        play_id: &str,
        results: &EvaluationResults,
    ) -> Result<Option<FeedbackResult>> {
        // Find failed probes
        let failed_probes: Vec<_> = results
            .probes
            .iter()
            .filter(|p| !p.passed)
            .collect();

        if failed_probes.is_empty() {
            // All probes passed - reset consecutive failures
            if let Some(history) = self.history.get_mut(play_id) {
                history.consecutive_failures.clear();
            }
            return Ok(None);
        }

        // Get or create history for this play and update failure counts
        let history = self
            .history
            .entry(play_id.to_string())
            .or_default();

        for probe in &failed_probes {
            let key = format!("{:?}", probe.probe.probe_type);
            *history.consecutive_failures.entry(key.clone()).or_default() += 1;
            *history.total_failures.entry(key).or_default() += 1;
        }

        // Generate suggestions for failed probes
        let suggestions = self.generate_suggestions(&failed_probes);

        debug!(
            play_id = %play_id,
            failed_count = %failed_probes.len(),
            suggestion_count = %suggestions.len(),
            "Generated feedback suggestions"
        );

        // Check if we should create an issue (need to clone history data to avoid borrow)
        let history_snapshot = self.history.get(play_id).cloned().unwrap_or_default();
        let should_create_issue = self.config.auto_create_issues
            && self.check_should_create_issue(&history_snapshot, &failed_probes);

        let issue_url = if should_create_issue {
            self.create_feedback_issue(play_id, results, &suggestions)?
        } else {
            None
        };

        Ok(Some(FeedbackResult {
            play_id: play_id.to_string(),
            suggestions,
            issue_created: issue_url,
            generated_at: Utc::now().to_rfc3339(),
        }))
    }

    /// Check if an issue should be created based on failure history.
    fn check_should_create_issue(&self, history: &FailureHistory, failed_probes: &[&ProbeResult]) -> bool {
        for probe in failed_probes {
            let key = format!("{:?}", probe.probe.probe_type);
            if let Some(count) = history.consecutive_failures.get(&key) {
                if *count >= self.config.failure_threshold {
                    // Check if we already created an issue for this pattern
                    let fingerprint = format!("{}:{}", key, probe.probe.question);
                    if !history.created_issues.iter().any(|i| i.contains(&fingerprint)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Generate improvement suggestions based on failed probes.
    #[must_use]
    pub fn generate_suggestions(&self, failed_probes: &[&ProbeResult]) -> Vec<PromptSuggestion> {
        let mut suggestions = Vec::new();

        for probe in failed_probes {
            let suggestion = match probe.probe.probe_type {
                ProbeType::Artifact => PromptSuggestion {
                    category: SuggestionCategory::FileTracking,
                    title: "Add explicit file tracking instructions".to_string(),
                    description: "The agent failed to track which files were created or modified. \
                        Add explicit instructions to maintain a running list of file operations."
                        .to_string(),
                    triggered_by: ProbeType::Artifact,
                    example_addition: Some(
                        r#"## File Tracking Requirements

Maintain a running list of all file operations:
- When creating a file: Log "Created: <path>"
- When modifying a file: Log "Modified: <path> - <brief description>"
- When reading a file: Note important contents

At the end of each major step, summarize files changed."#
                            .to_string(),
                    ),
                    priority: 9,
                },

                ProbeType::Recall => PromptSuggestion {
                    category: SuggestionCategory::ContextPlacement,
                    title: "Improve error context placement".to_string(),
                    description: "The agent failed to recall error information. This is often due to \
                        the 'lost-in-middle' effect where information in the middle of context \
                        receives 10-40% lower attention. Place error context at the START or END \
                        of the prompt."
                        .to_string(),
                    triggered_by: ProbeType::Recall,
                    example_addition: Some(
                        r#"## Current Error Context (IMPORTANT - READ FIRST)

[Place error message and stack trace here at the START of context]

---"#
                            .to_string(),
                    ),
                    priority: 8,
                },

                ProbeType::Continuation => PromptSuggestion {
                    category: SuggestionCategory::TaskPlanning,
                    title: "Add clearer next-step planning".to_string(),
                    description: "The agent couldn't articulate the next step. Add a structured \
                        planning section that's updated throughout the task."
                        .to_string(),
                    triggered_by: ProbeType::Continuation,
                    example_addition: Some(
                        r#"## Task Progress

### Completed Steps
- [List what's done]

### Current Step
- [What you're working on now]

### Next Steps
- [What comes after current step]
- [Additional pending work]"#
                            .to_string(),
                    ),
                    priority: 6,
                },

                ProbeType::Decision => PromptSuggestion {
                    category: SuggestionCategory::DecisionLogging,
                    title: "Add decision logging requirements".to_string(),
                    description: "The agent couldn't recall decisions made. Require explicit \
                        decision documentation with rationale."
                        .to_string(),
                    triggered_by: ProbeType::Decision,
                    example_addition: Some(
                        r#"## Decision Log

When making significant decisions, document:
1. What decision was made
2. Why this approach was chosen
3. What alternatives were considered

Example: "Decision: Using Redis for session storage because it provides built-in TTL support and the team already has Redis infrastructure.""#
                            .to_string(),
                    ),
                    priority: 5,
                },

                ProbeType::Technical => PromptSuggestion {
                    category: SuggestionCategory::General,
                    title: "Improve technical detail retention".to_string(),
                    description: "The agent failed to recall technical details like function \
                        signatures or API parameters. Consider adding a technical notes section."
                        .to_string(),
                    triggered_by: ProbeType::Technical,
                    example_addition: Some(
                        r#"## Technical Notes

Maintain key technical details:
- Function signatures
- API endpoints and parameters
- Type definitions
- Configuration values"#
                            .to_string(),
                    ),
                    priority: 7,
                },

                ProbeType::Acceptance => PromptSuggestion {
                    category: SuggestionCategory::AcceptanceVerification,
                    title: "Add acceptance criteria verification".to_string(),
                    description: "The agent couldn't confirm acceptance criteria status. Add \
                        explicit verification checkpoints."
                        .to_string(),
                    triggered_by: ProbeType::Acceptance,
                    example_addition: Some(
                        r#"## Acceptance Criteria Checklist

Before completing, verify each criterion:
- [ ] Criterion 1: [description]
- [ ] Criterion 2: [description]

Run verification: [specific command or check]"#
                            .to_string(),
                    ),
                    priority: 10, // Highest priority
                },
            };

            suggestions.push(suggestion);
        }

        // Sort by priority (highest first)
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions
    }

    /// Create a GitHub issue for feedback.
    fn create_feedback_issue(
        &mut self,
        play_id: &str,
        results: &EvaluationResults,
        suggestions: &[PromptSuggestion],
    ) -> Result<Option<String>> {
        let Some(github) = &self.github else {
            debug!("GitHub client not configured, skipping issue creation");
            return Ok(None);
        };

        let title = format!(
            "[CONTEXT-ENG] Agent prompt improvements needed for {}",
            play_id
        );

        let failed_probes: Vec<_> = results
            .probes
            .iter()
            .filter(|p| !p.passed)
            .collect();

        let mut body = format!(
            r#"## Context Engineering Feedback

**Play ID:** `{play_id}`
**Overall Score:** {score:.2}
**Threshold:** {threshold:.2}
**Status:** {status}
**Evaluated At:** {evaluated_at}

## Failed Probes

| Type | Question | Score | Expected Keywords |
|------|----------|-------|-------------------|
"#,
            play_id = play_id,
            score = results.overall_score,
            threshold = results.threshold,
            status = if results.passed { "PASSED" } else { "FAILED" },
            evaluated_at = results.evaluated_at.as_deref().unwrap_or("N/A"),
        );

        for probe in &failed_probes {
            body.push_str(&format!(
                "| {:?} | {} | {:.2} | {} |\n",
                probe.probe.probe_type,
                probe.probe.question,
                probe.score,
                probe.probe.expected_keywords.join(", "),
            ));
        }

        body.push_str("\n## Suggested Improvements\n\n");

        for suggestion in suggestions {
            body.push_str(&format!(
                "### {} (Priority: {})\n\n",
                suggestion.title, suggestion.priority
            ));
            body.push_str(&format!("**Category:** {}\n\n", suggestion.category));
            body.push_str(&format!("{}\n\n", suggestion.description));

            if let Some(example) = &suggestion.example_addition {
                body.push_str("**Suggested Addition:**\n\n```markdown\n");
                body.push_str(example);
                body.push_str("\n```\n\n");
            }
        }

        body.push_str(
            r#"
---
*This issue was auto-generated by the Context Engineering Feedback Engine based on probe-based evaluation results.*

**Reference:** [Agent-Skills-for-Context-Engineering](https://github.com/muratcankoylan/Agent-Skills-for-Context-Engineering)
"#,
        );

        let labels: Vec<&str> = self.config.issue_labels.iter().map(String::as_str).collect();

        match github.create_issue(&title, &body, &labels) {
            Ok(url) => {
                info!(issue = %url, play_id = %play_id, "Created feedback issue");

                // Record that we created this issue
                if let Some(history) = self.history.get_mut(play_id) {
                    history.created_issues.push(url.clone());
                }

                Ok(Some(url))
            }
            Err(e) => {
                warn!(error = %e, "Failed to create feedback issue");
                Ok(None)
            }
        }
    }

    /// Reset failure history for a play.
    pub fn reset_history(&mut self, play_id: &str) {
        self.history.remove(play_id);
    }

    /// Get current failure history for a play.
    #[must_use]
    pub fn get_history(&self, play_id: &str) -> Option<&FailureHistory> {
        self.history.get(play_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::types::EvaluationProbe;

    fn make_failed_probe(probe_type: ProbeType, score: f32) -> ProbeResult {
        ProbeResult {
            probe: EvaluationProbe::new(probe_type, "Test question"),
            response: "Test response".to_string(),
            score,
            passed: false,
            notes: None,
        }
    }

    #[test]
    fn test_generate_suggestions_artifact() {
        let engine = FeedbackEngine::with_defaults();
        let probe = make_failed_probe(ProbeType::Artifact, 0.3);
        let suggestions = engine.generate_suggestions(&[&probe]);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].category, SuggestionCategory::FileTracking);
        assert!(suggestions[0].example_addition.is_some());
    }

    #[test]
    fn test_generate_suggestions_multiple() {
        let engine = FeedbackEngine::with_defaults();
        let probe1 = make_failed_probe(ProbeType::Artifact, 0.3);
        let probe2 = make_failed_probe(ProbeType::Recall, 0.2);
        let suggestions = engine.generate_suggestions(&[&probe1, &probe2]);

        assert_eq!(suggestions.len(), 2);
        // Should be sorted by priority (Artifact = 9, Recall = 8)
        assert_eq!(suggestions[0].triggered_by, ProbeType::Artifact);
        assert_eq!(suggestions[1].triggered_by, ProbeType::Recall);
    }

    #[test]
    fn test_suggestion_category_display() {
        assert_eq!(
            format!("{}", SuggestionCategory::FileTracking),
            "File Tracking"
        );
        assert_eq!(
            format!("{}", SuggestionCategory::ContextPlacement),
            "Context Placement"
        );
    }
}
