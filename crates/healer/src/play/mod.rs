//! Play orchestration module for tracking parallel task execution.
//!
//! This module provides the core functionality for Healer to:
//! - Track batch execution of parallel tasks
//! - Detect stuck/failed tasks (>30 minute threshold)
//! - Spawn code-based remediations
//! - Gather optimization insights
//! - Monitor running plays with real-time log analysis
//! - Detect anomalies based on expected agent behaviors

pub mod batch;
pub mod behavior;
pub mod cleanup;
pub mod insights;
pub mod monitor;
pub mod remediate;
pub mod stage;
pub mod task;
pub mod tracker;
pub mod types;

// Re-export primary types
pub use batch::PlayBatch;
pub use behavior::{AgentType, BehaviorAnalyzer, DetectionType, LogAnalysis};
pub use monitor::{MonitorConfig, MonitorEvent, MonitorStatus, PlayMonitor};
pub use stage::Stage;
pub use task::TaskState;
pub use tracker::PlayTracker;
pub use types::{BatchStatus, Issue, RemediationState, TaskStatus};
