//! Domain facades for task management.
//!
//! These facades provide high-level operations that combine
//! storage operations with business logic.

mod ai;
mod config;
mod deps;
pub mod docs;
pub mod intake;
pub mod routing;
mod tags;
mod tasks;

pub use ai::AIDomain;
pub use config::ConfigDomain;
pub use deps::DependencyDomain;
pub use intake::{IntakeConfig, IntakeDomain, IntakeResult};
pub use routing::{infer_agent_hint, infer_agent_hint_str, Agent};
pub use tags::TagsDomain;
pub use tasks::TasksDomain;
