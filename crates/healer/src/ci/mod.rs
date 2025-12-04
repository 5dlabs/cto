//! CI remediation module for intelligent failure routing and fixing.
//!
//! This module provides the central CI remediation hub functionality:
//! - Receives CI failure events from webhooks/sensors
//! - Classifies failure types using workflow logs and changed files
//! - Routes to specialist agents (Rex, Blaze, Bolt, Cipher, Atlas)
//! - Tracks remediation attempts and implements retry logic
//! - Escalates to humans after max attempts
//! - Stores outcomes to OpenMemory for learning

pub mod context;
pub mod escalate;
pub mod memory;
pub mod router;
pub mod server;
pub mod spawner;
pub mod tracker;
pub mod types;

// Re-export primary types
pub use context::ContextGatherer;
pub use escalate::Escalator;
pub use memory::{MemoryClient, MemoryConfig};
pub use router::CiRouter;
pub use server::{build_router, run_server, ServerState};
pub use spawner::CodeRunSpawner;
pub use tracker::{CompletionAction, RemediationTracker, TrackedRemediation};
pub use types::{
    Agent, AttemptOutcome, ChangedFile, CiFailure, CiFailureType, HistoricalContext, MemoryEntry,
    PullRequest, RemediationAttempt, RemediationConfig, RemediationContext, RemediationState,
    RemediationStatus, SecurityAlert,
};

