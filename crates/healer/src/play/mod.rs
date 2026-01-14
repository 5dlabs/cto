//! Play orchestration module for tracking parallel task execution.
//!
//! This module provides the core functionality for Healer to:
//! - Track batch execution of parallel tasks
//! - Detect stuck/failed tasks (>30 minute threshold)
//! - Spawn code-based remediations
//! - Gather optimization insights
//! - Monitor running plays with real-time log analysis
//! - Detect anomalies based on expected agent behaviors
//! - Probe-based evaluation for context engineering quality (LLM-powered)
//! - HTTP API for MCP server integration (session start notification)

pub mod api;
pub mod batch;
pub mod behavior;
pub mod cleanup;
pub mod evaluation_spawner;
pub mod evaluator;
pub mod feedback;
pub mod insights;
pub mod monitor;
pub mod orchestrator;
pub mod remediate;
pub mod remediation_spawner;
pub mod session;
pub mod stage;
pub mod task;
pub mod tracker;
pub mod types;

// Re-export primary types
pub use api::{build_play_api_router, run_play_api_server, PlayApiState};
pub use batch::PlayBatch;
pub use behavior::{AgentType, BehaviorAnalyzer, DetectionType, LogAnalysis};
pub use evaluation_spawner::{EvaluationSpawnResult, EvaluationSpawner, EvaluationSpawnerConfig};
pub use evaluator::{EvaluatorConfig, ProbeEvaluator};
pub use feedback::{FeedbackConfig, FeedbackEngine, FeedbackResult, PromptSuggestion};
pub use monitor::{MonitorConfig, MonitorEvent, MonitorStatus, PlayMonitor};
pub use orchestrator::{
    verify_language_match, FeedbackLoopResult, HealerOrchestrator, ImplementationLanguage,
    LanguageMatchResult, OrchestratorConfig,
};
pub use remediation_spawner::{
    RemediationSpawnResult, RemediationSpawner, RemediationSpawnerConfig, RemediationStrategy,
};
pub use session::{
    AgentConfig, AgentTools, CtoConfig, IssueSeverity, IssueType, PlaySession, SessionIssue,
    SessionStatus, SessionStore, SessionStoreHandle, StartSessionRequest, TaskInfo,
};
pub use stage::Stage;
pub use task::TaskState;
pub use tracker::PlayTracker;
pub use types::{BatchStatus, EvaluationConfig, Issue, RemediationState, TaskStatus};
