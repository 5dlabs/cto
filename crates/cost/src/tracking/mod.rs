//! Cost tracking and analytics for task/agent efficiency.
//!
//! This module provides tools for tracking API calls with custom context
//! (task, agent, project, session) and computing efficiency metrics.
//!
//! ## Quick Start
//!
//! ```rust
//! use cto_cost::tracking::{CostTracker, TrackingContext, TrackingFilter};
//!
//! // Create a tracker
//! let tracker = CostTracker::new();
//!
//! // Record API calls with context
//! tracker.builder()
//!     .project("my-project")
//!     .task("implement-feature")
//!     .agent("claude-opus")
//!     .iteration(1)
//!     .provider("anthropic")
//!     .model("claude-3-opus")
//!     .input_tokens(1000)
//!     .output_tokens(500)
//!     .cost(0.05)
//!     .record();
//!
//! // Query by task
//! let filter = TrackingFilter::new().with_task("implement-feature");
//! let calls = tracker.query(&filter);
//!
//! // Get task metrics
//! let metrics = tracker.task_metrics("implement-feature");
//! println!("Iterations: {}", metrics.iterations);
//! println!("Cost per iteration: ${:.4}", metrics.cost_per_iteration);
//!
//! // Compare agent efficiency
//! let comparison = tracker.compare_agents();
//! if let Some(best) = comparison.most_efficient_agent {
//!     println!("Most efficient: {}", best);
//! }
//! ```

mod metrics;
mod models;
mod tracker;

pub use metrics::{AgentComparison, AgentMetrics, AggregatedMetrics, ProjectMetrics, TaskMetrics};
pub use models::{
    AgentId, ProjectId, SessionId, TaskId, TrackedCall, TrackingContext, TrackingFilter,
};
pub use tracker::{CostTracker, TrackedCallBuilder};
