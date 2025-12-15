//! Sensors for actively monitoring external systems.
//!
//! This module contains sensors that poll external systems for issues and trigger
//! remediation when problems are detected.

pub mod github_actions;

pub use github_actions::{GitHubActionsSensor, SensorConfig, WorkflowFailure};
