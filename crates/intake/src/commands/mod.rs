//! Command implementations for the intake CLI.
//!
//! This module contains the business logic for CLI commands,
//! separated from the argument parsing in the CLI binary.

pub mod update;

pub use update::{execute as update, UpdateConfig, UpdateDomain, UpdateResult};
