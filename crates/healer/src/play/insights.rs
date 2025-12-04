//! Intelligence gathering for agent optimization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::stage::Stage;

/// Collects observations about agent behavior for prompt optimization.
#[derive(Debug, Default)]
pub struct InsightCollector {
    /// Recorded observations
    observations: Vec<AgentObservation>,
}

impl InsightCollector {
    /// Create a new insight collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an observation.
    pub fn record(&mut self, observation: AgentObservation) {
        self.observations.push(observation);
    }

    /// Record an inefficiency.
    pub fn record_inefficiency(
        &mut self,
        agent: impl Into<String>,
        task_id: impl Into<String>,
        details: impl Into<String>,
    ) {
        self.record(AgentObservation {
            agent: agent.into(),
            task_id: task_id.into(),
            timestamp: Utc::now(),
            observation_type: ObservationType::InefficiencyDetected,
            details: details.into(),
            stage: None,
        });
    }

    /// Record a repeated mistake.
    pub fn record_mistake(
        &mut self,
        agent: impl Into<String>,
        task_id: impl Into<String>,
        details: impl Into<String>,
    ) {
        self.record(AgentObservation {
            agent: agent.into(),
            task_id: task_id.into(),
            timestamp: Utc::now(),
            observation_type: ObservationType::RepeatedMistake,
            details: details.into(),
            stage: None,
        });
    }

    /// Record a success pattern.
    pub fn record_success(
        &mut self,
        agent: impl Into<String>,
        task_id: impl Into<String>,
        details: impl Into<String>,
    ) {
        self.record(AgentObservation {
            agent: agent.into(),
            task_id: task_id.into(),
            timestamp: Utc::now(),
            observation_type: ObservationType::SuccessPattern,
            details: details.into(),
            stage: None,
        });
    }

    /// Record excessive retries.
    pub fn record_retries(
        &mut self,
        agent: impl Into<String>,
        task_id: impl Into<String>,
        retry_count: usize,
        details: impl Into<String>,
    ) {
        self.record(AgentObservation {
            agent: agent.into(),
            task_id: task_id.into(),
            timestamp: Utc::now(),
            observation_type: ObservationType::ExcessiveRetries { count: retry_count },
            details: details.into(),
            stage: None,
        });
    }

    /// Record fast completion.
    pub fn record_fast_completion(
        &mut self,
        agent: impl Into<String>,
        task_id: impl Into<String>,
        duration_mins: i64,
    ) {
        self.record(AgentObservation {
            agent: agent.into(),
            task_id: task_id.into(),
            timestamp: Utc::now(),
            observation_type: ObservationType::FastCompletion { duration_mins },
            details: format!("Completed in {duration_mins} minutes"),
            stage: None,
        });
    }

    /// Get all observations.
    #[must_use]
    pub fn observations(&self) -> &[AgentObservation] {
        &self.observations
    }

    /// Get observations for a specific agent.
    #[must_use]
    pub fn for_agent(&self, agent: &str) -> Vec<&AgentObservation> {
        self.observations
            .iter()
            .filter(|o| o.agent.eq_ignore_ascii_case(agent))
            .collect()
    }

    /// Get common failure patterns.
    #[must_use]
    pub fn failure_patterns(&self) -> Vec<FailurePattern> {
        let mut patterns: std::collections::HashMap<String, FailurePattern> =
            std::collections::HashMap::new();

        for obs in &self.observations {
            if matches!(
                obs.observation_type,
                ObservationType::RepeatedMistake | ObservationType::InefficiencyDetected
            ) {
                let key = format!("{}:{}", obs.agent, obs.details);
                patterns
                    .entry(key)
                    .or_insert_with(|| FailurePattern {
                        agent: obs.agent.clone(),
                        description: obs.details.clone(),
                        occurrences: 0,
                        first_seen: obs.timestamp,
                        last_seen: obs.timestamp,
                    })
                    .occurrences += 1;

                if let Some(pattern) = patterns.get_mut(&format!("{}:{}", obs.agent, obs.details)) {
                    if obs.timestamp > pattern.last_seen {
                        pattern.last_seen = obs.timestamp;
                    }
                }
            }
        }

        let mut result: Vec<_> = patterns.into_values().collect();
        result.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        result
    }

    /// Analyze observations and suggest prompt improvements.
    #[must_use]
    pub fn suggest_optimizations(&self) -> Vec<PromptSuggestion> {
        let mut suggestions = vec![];

        // Group observations by agent and pattern
        let patterns = self.failure_patterns();

        for pattern in patterns {
            if pattern.occurrences >= 2 {
                suggestions.push(PromptSuggestion {
                    agent: pattern.agent.clone(),
                    observation: pattern.description.clone(),
                    suggested_change: suggest_fix_for_pattern(&pattern),
                    confidence: if pattern.occurrences >= 5 {
                        Confidence::High
                    } else if pattern.occurrences >= 3 {
                        Confidence::Medium
                    } else {
                        Confidence::Low
                    },
                    occurrences: pattern.occurrences,
                });
            }
        }

        suggestions
    }

    /// Get agent performance stats.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    pub fn agent_stats(&self, agent: &str) -> AgentStats {
        let obs = self.for_agent(agent);

        let successes = obs
            .iter()
            .filter(|o| matches!(o.observation_type, ObservationType::SuccessPattern))
            .count();

        let failures = obs
            .iter()
            .filter(|o| {
                matches!(
                    o.observation_type,
                    ObservationType::RepeatedMistake | ObservationType::InefficiencyDetected
                )
            })
            .count();

        let total = successes + failures;
        let success_rate = if total > 0 {
            (successes as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let avg_duration = obs
            .iter()
            .filter_map(|o| {
                if let ObservationType::FastCompletion { duration_mins } = o.observation_type {
                    Some(duration_mins)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let avg_duration_mins = if avg_duration.is_empty() {
            0
        } else {
            avg_duration.iter().sum::<i64>() / avg_duration.len() as i64
        };

        AgentStats {
            agent: agent.to_string(),
            runs_analyzed: total,
            success_rate,
            avg_duration_mins,
            top_issues: self.failure_patterns().into_iter().take(3).collect(),
        }
    }

    /// Clear all observations.
    pub fn clear(&mut self) {
        self.observations.clear();
    }
}

/// Suggest a fix based on a failure pattern.
fn suggest_fix_for_pattern(pattern: &FailurePattern) -> String {
    let details = pattern.description.to_lowercase();

    if details.contains("import") {
        "Add: \"Before committing, verify all imports resolve\"".to_string()
    } else if details.contains("conflict") || details.contains("git") {
        "Add: \"Pull latest changes before pushing to avoid conflicts\"".to_string()
    } else if details.contains("test") {
        "Add: \"Always run tests before committing\"".to_string()
    } else if details.contains("timeout") {
        "Add: \"Make atomic commits - one logical change per commit\"".to_string()
    } else if details.contains("retry") {
        "Add: \"If an operation fails, analyze the error before retrying\"".to_string()
    } else {
        format!("Investigate recurring issue: {}", pattern.description)
    }
}

/// An observation about agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservation {
    /// Agent name
    pub agent: String,
    /// Task ID
    pub task_id: String,
    /// When observed
    pub timestamp: DateTime<Utc>,
    /// Type of observation
    pub observation_type: ObservationType,
    /// Details
    pub details: String,
    /// Stage where observed (if applicable)
    pub stage: Option<Stage>,
}

/// Type of observation about agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationType {
    /// Agent took an inefficient path
    InefficiencyDetected,
    /// Agent made a common mistake
    RepeatedMistake,
    /// Agent succeeded with a good pattern
    SuccessPattern,
    /// Agent used excessive retries
    ExcessiveRetries {
        /// Number of retries
        count: usize,
    },
    /// Agent completed faster than expected
    FastCompletion {
        /// Duration in minutes
        duration_mins: i64,
    },
}

/// A recurring failure pattern.
#[derive(Debug, Clone)]
pub struct FailurePattern {
    /// Agent name
    pub agent: String,
    /// Description of the pattern
    pub description: String,
    /// Number of occurrences
    pub occurrences: usize,
    /// First time seen
    pub first_seen: DateTime<Utc>,
    /// Last time seen
    pub last_seen: DateTime<Utc>,
}

/// A suggested prompt improvement.
#[derive(Debug, Clone)]
pub struct PromptSuggestion {
    /// Agent name
    pub agent: String,
    /// What was observed
    pub observation: String,
    /// Suggested prompt change
    pub suggested_change: String,
    /// Confidence level
    pub confidence: Confidence,
    /// Number of occurrences
    pub occurrences: usize,
}

/// Confidence level for a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// High confidence (5+ occurrences)
    High,
    /// Medium confidence (3-4 occurrences)
    Medium,
    /// Low confidence (2 occurrences)
    Low,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
        }
    }
}

/// Stats for an agent.
#[derive(Debug, Clone)]
pub struct AgentStats {
    /// Agent name
    pub agent: String,
    /// Number of runs analyzed
    pub runs_analyzed: usize,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average duration in minutes
    pub avg_duration_mins: i64,
    /// Top issues
    pub top_issues: Vec<FailurePattern>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_collector() {
        let mut collector = InsightCollector::new();

        collector.record_mistake("Rex", "1", "Missing import error");
        collector.record_mistake("Rex", "2", "Missing import error");
        collector.record_success("Rex", "3", "Clean implementation");

        let patterns = collector.failure_patterns();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].occurrences, 2);

        let suggestions = collector.suggest_optimizations();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].suggested_change.contains("import"));
    }

    #[test]
    fn test_agent_stats() {
        let mut collector = InsightCollector::new();

        collector.record_success("Cleo", "1", "Good review");
        collector.record_success("Cleo", "2", "Good review");
        collector.record_mistake("Cleo", "3", "Missed issue");

        let stats = collector.agent_stats("Cleo");
        assert_eq!(stats.runs_analyzed, 3);
        assert!((stats.success_rate - 66.67).abs() < 1.0);
    }
}

