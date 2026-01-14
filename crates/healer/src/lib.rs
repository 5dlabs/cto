//! Healer library - self-healing platform monitor.
//!
//! This crate provides:
//! - Play session monitoring and evaluation
//! - CI remediation
//! - Platform health monitoring
//! - Loki log scanning

// Internal modules needed by other modules
mod github;
mod templates;

// Re-export modules for integration tests and library usage
pub mod ci;
pub mod loki;
pub mod play;
pub mod scanner;
pub mod sensors;

// Re-export common types
pub use play::{
    build_play_api_router, run_play_api_server, EvaluationSpawnResult, EvaluationSpawner,
    EvaluationSpawnerConfig, PlayApiState, PlaySession, SessionStore, SessionStoreHandle,
    StartSessionRequest,
};
