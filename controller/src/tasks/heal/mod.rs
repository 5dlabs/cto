//! Heal task module for remediation CodeRuns.
//!
//! This module provides naming and template utilities for heal remediation agents.
//! Unlike code/docs tasks, heal remediation CodeRuns are spawned by the heal server
//! (not by the controller), but the controller can use these utilities for
//! consistent naming and template resolution.

pub mod naming;
pub mod templates;

pub use naming::HealNaming;
pub use templates::HealTemplatePaths;
