//! Shared types for play orchestration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::stage::Stage;

/// Status of a batch of parallel tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch is currently executing
    InProgress {
        /// Number of completed tasks
        completed: usize,
        /// Total number of tasks
        total: usize,
    },
    /// All tasks completed successfully
    Completed,
    /// One or more tasks failed
    Failed {
        /// Task IDs that failed
        failed_tasks: Vec<String>,
    },
}

impl BatchStatus {
    /// Check if the batch is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::InProgress { .. })
    }

    /// Check if the batch completed successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Status of an individual task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to start
    Pending,
    /// Task is currently executing
    InProgress {
        /// Current stage
        stage: Stage,
        /// When this stage started
        stage_started: DateTime<Utc>,
    },
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed {
        /// Stage where failure occurred
        stage: Stage,
        /// Reason for failure
        reason: String,
        /// Active remediation, if any
        remediation: Option<RemediationState>,
    },
}

impl TaskStatus {
    /// Check if the task is still running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::InProgress { .. })
    }

    /// Get the current stage if in progress.
    #[must_use]
    pub fn current_stage(&self) -> Option<Stage> {
        match self {
            Self::InProgress { stage, .. } | Self::Failed { stage, .. } => Some(*stage),
            _ => None,
        }
    }

    /// Get the stage start time if in progress.
    #[must_use]
    pub fn stage_started(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::InProgress { stage_started, .. } => Some(*stage_started),
            _ => None,
        }
    }
}

/// State of an active remediation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationState {
    /// Name of the Healer `CodeRun`
    pub coderun_name: String,
    /// Summary of the diagnosis
    pub diagnosis: String,
    /// When remediation started
    pub started_at: DateTime<Utc>,
}

/// An issue detected during play execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Issue {
    /// Task stuck in stage for >30 minutes
    StageTimeout {
        /// Task identifier
        task_id: String,
        /// Stage where stuck
        stage: Stage,
        /// Time elapsed in this stage
        elapsed: Duration,
    },
    /// Task failed and needs code fix
    NeedsRemediation {
        /// Task identifier
        task_id: String,
        /// Stage where failure occurred
        stage: Stage,
        /// Reason for failure
        failure_reason: String,
    },
    /// Agent behaving suboptimally (optimization opportunity)
    OptimizationOpportunity {
        /// Task identifier
        task_id: String,
        /// Agent name
        agent: String,
        /// What was observed
        observation: String,
        /// Suggested prompt improvement
        suggested_prompt_change: String,
    },
}

impl Issue {
    /// Get the task ID associated with this issue.
    #[must_use]
    pub fn task_id(&self) -> &str {
        match self {
            Self::StageTimeout { task_id, .. }
            | Self::NeedsRemediation { task_id, .. }
            | Self::OptimizationOpportunity { task_id, .. } => task_id,
        }
    }

    /// Get a human-readable description of the issue.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::StageTimeout {
                task_id,
                stage,
                elapsed,
            } => {
                format!(
                    "Task {} stuck in {:?} for {}m (>30m threshold)",
                    task_id,
                    stage,
                    elapsed.as_secs() / 60
                )
            }
            Self::NeedsRemediation {
                task_id,
                stage,
                failure_reason,
            } => {
                format!("Task {task_id} failed in {stage:?}: {failure_reason}")
            }
            Self::OptimizationOpportunity {
                task_id,
                agent,
                observation,
                ..
            } => {
                format!("Task {task_id} ({agent}) optimization: {observation}")
            }
        }
    }
}

/// Context gathered for diagnosing an issue.
#[derive(Debug, Clone, Default)]
pub struct DiagnosisContext {
    /// Logs from the failed pod
    pub logs: String,
    /// Agent's last actions/output
    pub agent_output: String,
    /// Relevant source code snippets
    pub code_snippets: Vec<String>,
    /// PR state (if applicable)
    pub pr_state: Option<PrContext>,
    /// CI results (if applicable)
    pub ci_results: Option<String>,
    /// Artifact trail from agent session (context engineering)
    pub artifact_trail: Option<ArtifactTrail>,
    /// Evaluation probes to run for acceptance verification
    pub evaluation_probes: Vec<EvaluationProbe>,
}

/// PR context for diagnosis.
#[derive(Debug, Clone)]
pub struct PrContext {
    /// PR number
    pub number: u32,
    /// PR state (open, closed, merged)
    pub state: String,
    /// Whether PR is mergeable
    pub mergeable: bool,
    /// Check status
    pub checks_status: String,
}

/// Diagnosis result from analyzing an issue.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    /// Summary of what went wrong
    pub summary: String,
    /// Root cause category
    pub category: DiagnosisCategory,
    /// Suggested fix approach
    pub suggested_fix: String,
    /// Relevant files to examine/modify
    pub relevant_files: Vec<String>,
}

/// Category of diagnosed issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosisCategory {
    /// Issue in the source code being modified
    CodeIssue,
    /// Issue in agent prompts/templates
    PromptIssue,
    /// Infrastructure/environment issue
    InfraIssue,
    /// Git/version control issue
    GitIssue,
    /// Unknown/needs investigation
    Unknown,
}

// =============================================================================
// Probe-Based Evaluation (Context Engineering)
// =============================================================================

/// Type of evaluation probe for acceptance criteria verification.
///
/// Based on context engineering research showing that traditional metrics (ROUGE,
/// embedding similarity) fail to capture functional compression quality.
/// Probe-based evaluation directly measures whether the agent retained critical
/// information by asking targeted questions.
///
/// Reference: Agent-Skills-for-Context-Engineering/skills/context-compression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeType {
    /// Factual retention - "What was the original error message?"
    Recall,
    /// File tracking - "Which files have we modified?"
    Artifact,
    /// Task planning - "What should we do next?"
    Continuation,
    /// Reasoning chain - "What did we decide about the Redis issue?"
    Decision,
    /// Technical accuracy - "What is the function signature?"
    Technical,
    /// Acceptance criteria - "Did we meet requirement X?"
    Acceptance,
}

impl ProbeType {
    /// Get example questions for this probe type.
    #[must_use]
    pub fn example_questions(&self) -> &[&str] {
        match self {
            Self::Recall => &[
                "What was the original error message?",
                "What error triggered this task?",
                "What was the initial problem description?",
            ],
            Self::Artifact => &[
                "Which files have we modified?",
                "What new files were created?",
                "Which configuration files were changed?",
            ],
            Self::Continuation => &[
                "What should we do next?",
                "What's the next step in the plan?",
                "What remains to be done?",
            ],
            Self::Decision => &[
                "What did we decide about the architecture?",
                "Why did we choose this approach?",
                "What alternatives were considered?",
            ],
            Self::Technical => &[
                "What is the function signature?",
                "What parameters does the API accept?",
                "What type does this function return?",
            ],
            Self::Acceptance => &[
                "Did we meet the acceptance criteria?",
                "Does the implementation satisfy requirement X?",
                "Have all tests passed?",
            ],
        }
    }

    /// Get the weight for this probe type (higher = more important).
    #[must_use]
    pub fn default_weight(&self) -> f32 {
        match self {
            Self::Acceptance => 1.0, // Most important for Play workflow
            Self::Artifact => 0.9,   // Critical for context compression
            Self::Technical => 0.8,
            Self::Recall => 0.7,
            Self::Continuation => 0.6,
            Self::Decision => 0.5,
        }
    }
}

/// An evaluation probe for testing agent knowledge retention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationProbe {
    /// Type of probe
    pub probe_type: ProbeType,
    /// The question to ask
    pub question: String,
    /// Keywords expected in a correct answer
    pub expected_keywords: Vec<String>,
    /// Weight of this probe (0.0-1.0)
    pub weight: f32,
    /// Optional ground truth answer
    pub ground_truth: Option<String>,
}

impl EvaluationProbe {
    /// Create a new probe with default weight.
    #[must_use]
    pub fn new(probe_type: ProbeType, question: impl Into<String>) -> Self {
        Self {
            weight: probe_type.default_weight(),
            probe_type,
            question: question.into(),
            expected_keywords: Vec::new(),
            ground_truth: None,
        }
    }

    /// Add expected keywords.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.expected_keywords = keywords;
        self
    }

    /// Set ground truth answer.
    #[must_use]
    pub fn with_ground_truth(mut self, truth: impl Into<String>) -> Self {
        self.ground_truth = Some(truth.into());
        self
    }

    /// Set custom weight.
    #[must_use]
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Score a response against this probe (0.0-1.0).
    #[must_use]
    pub fn score_response(&self, response: &str) -> f32 {
        if self.expected_keywords.is_empty() {
            // No keywords to check - can't score automatically
            return 0.5;
        }

        let response_lower = response.to_lowercase();
        let matches = self
            .expected_keywords
            .iter()
            .filter(|kw| response_lower.contains(&kw.to_lowercase()))
            .count();

        #[allow(clippy::cast_precision_loss)]
        let score = matches as f32 / self.expected_keywords.len() as f32;
        score
    }
}

/// Result of running a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// The probe that was run
    pub probe: EvaluationProbe,
    /// The response from the agent
    pub response: String,
    /// Score (0.0-1.0)
    pub score: f32,
    /// Whether the probe passed (score >= threshold)
    pub passed: bool,
    /// Any notes about the evaluation
    pub notes: Option<String>,
}

/// Artifact trail for tracking file operations (mirrors sidecar struct).
///
/// This addresses the "artifact trail problem" where file tracking scores
/// 2.2-2.5/5.0 across all compression methods.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactTrail {
    /// Files created during this session
    pub files_created: Vec<String>,
    /// Files modified with change summaries
    pub files_modified: std::collections::HashMap<String, String>,
    /// Files read but not modified
    pub files_read: Vec<String>,
    /// Key decisions made during the session
    pub decisions_made: Vec<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Evaluation results for a task or stage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvaluationResults {
    /// Individual probe results
    pub probes: Vec<ProbeResult>,
    /// Overall score (weighted average)
    pub overall_score: f32,
    /// Whether evaluation passed (overall_score >= threshold)
    pub passed: bool,
    /// Passing threshold used
    pub threshold: f32,
    /// Artifact trail if available
    pub artifact_trail: Option<ArtifactTrail>,
    /// Timestamp of evaluation
    pub evaluated_at: Option<String>,
}

impl EvaluationResults {
    /// Create from probe results with default threshold.
    #[must_use]
    pub fn from_probes(probes: Vec<ProbeResult>) -> Self {
        Self::from_probes_with_threshold(probes, 0.7)
    }

    /// Create from probe results with custom threshold.
    #[must_use]
    pub fn from_probes_with_threshold(probes: Vec<ProbeResult>, threshold: f32) -> Self {
        let total_weight: f32 = probes.iter().map(|p| p.probe.weight).sum();
        let weighted_score: f32 = probes
            .iter()
            .map(|p| p.score * p.probe.weight)
            .sum();

        let overall_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        Self {
            probes,
            overall_score,
            passed: overall_score >= threshold,
            threshold,
            artifact_trail: None,
            evaluated_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Attach artifact trail to results.
    #[must_use]
    pub fn with_artifact_trail(mut self, trail: ArtifactTrail) -> Self {
        self.artifact_trail = Some(trail);
        self
    }
}
