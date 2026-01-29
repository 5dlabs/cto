//! Linear Agent Activity Sink for CTO Platform
//!
//! This crate provides CLI-agnostic Linear Agent Activity emission,
//! enabling any agent pod to stream its activity to Linear's agent dialog.
//!
//! # Architecture
//!
//! - **client**: GraphQL client for Linear API
//! - **activities**: Activity content types and plan steps
//! - **emitter**: Trait-based abstraction for activity emission
//! - **parsers**: CLI-specific stream parsers (Claude, Factory, Codex, etc.)
//! - **sidecar**: Modular sidecar components for Kubernetes pods
//!
//! # Supported CLIs
//!
//! - Claude (stream-json format with System event)
//! - Factory (stream-json format)
//! - Codex (JSONL with commands array)
//! - Code/Every-Code (same as Codex)
//! - Gemini (JSONL format)
//! - `OpenCode` (JSONL format)
//! - Dexter (single JSON format)
//! - Cursor (JSON format)
//!
//! # Example
//!
//! ```rust,ignore
//! use linear_sink::{LinearClient, LinearAgentEmitter, AgentActivityEmitter, PlanStep};
//!
//! // Create emitter from environment
//! let emitter = LinearAgentEmitter::from_env()?;
//!
//! // Emit activities
//! emitter.emit_thought("Analyzing...", false).await?;
//! emitter.emit_action("Reading", "file.rs").await?;
//! emitter.emit_response("Task completed").await?;
//!
//! // Update plan
//! emitter.update_plan(&[
//!     PlanStep::completed("Parse input"),
//!     PlanStep::in_progress("Process data"),
//!     PlanStep::pending("Generate output"),
//! ]).await?;
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::ref_option)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unused_self)]

pub mod activities;
pub mod client;
pub mod emitter;
pub mod parsers;
pub mod sidecar;
pub mod testing;

// Re-export activity types
pub use activities::{
    ActivityContent, ActivitySignal, AgentActivityCreateInput, AgentActivityCreateResponse,
    AgentSessionUpdateInput, AgentSessionUpdateResponse, AuthSignalMetadata, CreatedAgentActivity,
    PlanStep, PlanStepStatus, SelectOption, SelectSignalMetadata, SignalMetadata,
    AGENT_ACTIVITY_CREATE_MUTATION, AGENT_SESSION_UPDATE_MUTATION,
};

// Re-export client
pub use client::LinearClient;

// Re-export emitter
pub use emitter::{AgentActivityEmitter, LinearAgentEmitter};

// Re-export parser types
pub use parsers::{
    ArtifactOperation, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats,
};
