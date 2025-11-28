//! Domain facades for task management.
//!
//! These facades provide high-level operations that combine
//! storage operations with business logic.

mod ai;
mod config;
mod deps;
mod tags;
mod tasks;

pub use ai::AIDomain;
pub use config::ConfigDomain;
pub use deps::DependencyDomain;
pub use tags::TagsDomain;
pub use tasks::TasksDomain;

