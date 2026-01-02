//! State management for the PM service.
//!
//! This module provides shared state for tracking sessions, workflows,
//! and agent mappings.

pub mod session_tracker;

pub use session_tracker::{SessionInfo, SessionTracker};
