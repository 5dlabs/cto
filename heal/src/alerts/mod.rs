//! Alert detection system for the CTO Monitor.
//!
//! This module implements reactive alerts that detect anomalies during workflow execution.
//! Each alert handler evaluates the current state and optionally triggers Factory for analysis.
//!
//! # Alert Types
//! - A1: Agent comment order mismatch (GitHub vs K8s state)
//! - A2: Silent agent failure (container died, pod still "Running")
//! - A3: Stale progress (no commits for threshold duration)
//! - A4: Repeated approval loop (same agent approving multiple times)
//! - A5: Post-Tess CI/Merge failure (CI failing or merge conflict after Tess approval)
//! - A7: Pod failure (any CTO pod in Failed/Error state)
//! - A8: Workflow step timeout (step running longer than threshold)

pub mod a1_comment_order;
pub mod a2_silent_failure;
pub mod a3_stale_progress;
pub mod a4_approval_loop;
pub mod a5_post_tess_ci;
pub mod a7_pod_failure;
pub mod a8_step_timeout;
pub mod types;

#[allow(unused_imports)] // Public API re-exports
pub use types::{Alert, AlertConfig, AlertContext, AlertHandler, AlertId};

use crate::github::GitHubState;
use crate::k8s::K8sEvent;

/// Registry of all alert handlers
pub struct AlertRegistry {
    handlers: Vec<Box<dyn AlertHandler>>,
}

impl AlertRegistry {
    /// Create a new alert registry with all handlers enabled
    pub fn new() -> Self {
        Self {
            handlers: vec![
                Box::new(a1_comment_order::Handler::new()),
                Box::new(a2_silent_failure::Handler::new()),
                Box::new(a3_stale_progress::Handler::new()),
                Box::new(a4_approval_loop::Handler::new()),
                Box::new(a5_post_tess_ci::Handler::new()),
                Box::new(a7_pod_failure::Handler::new()),
                Box::new(a8_step_timeout::Handler::new()),
            ],
        }
    }

    /// Evaluate all handlers against current state
    pub fn evaluate(
        &self,
        event: &K8sEvent,
        github: &GitHubState,
        ctx: &AlertContext,
    ) -> Vec<Alert> {
        self.handlers
            .iter()
            .filter_map(|h| h.evaluate(event, github, ctx))
            .collect()
    }
}

impl Default for AlertRegistry {
    fn default() -> Self {
        Self::new()
    }
}
