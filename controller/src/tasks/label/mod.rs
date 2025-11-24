//! # Label-Based Workflow Orchestration Module
//!
//! This module provides comprehensive PR label management for tracking remediation state
//! and iteration counts in the Agent Remediation Loop system.
//!
//! ## Label Schema Design
//!
//! The system uses a hierarchical label structure:
//! - **Task Association**: `task-{id}` - Persistent task identification
//! - **Iteration Tracking**: `iteration-{n}` - Current remediation cycle
//! - **Status Labels**: Workflow state indicators
//! - **Override Controls**: Human intervention capabilities
//!
//! ## Architecture
//!
//! The label orchestration system consists of several key components:
//!
//! - **`LabelSchema`**: Defines label types, patterns, and lifecycle rules
//! - **`GitHubLabelClient`**: GitHub API client with rate limiting and atomic operations
//! - **`LabelOrchestrator`**: State machine for workflow transitions
//! - **`OverrideDetector`**: Human override label detection and handling
//! - **`LabelCleanupManager`**: Automated cleanup of obsolete labels
//! - **`ConcurrentLabelManager`**: Race condition prevention and conflict resolution

pub mod cleanup;
pub mod client;
pub mod concurrent;
pub mod orchestrator;
pub mod override_detector;
pub mod schema;

pub use cleanup::LabelCleanupManager;
pub use client::GitHubLabelClient;
pub use concurrent::ConcurrentLabelManager;
pub use orchestrator::LabelOrchestrator;
pub use override_detector::OverrideDetector;
pub use schema::{LabelOperation, LabelSchema, LabelType, StateTransition, WorkflowState};
