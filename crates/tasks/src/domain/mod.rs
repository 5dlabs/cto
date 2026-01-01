//! Domain facades for task management.
//!
//! These facades provide high-level operations that combine
//! storage operations with business logic.

mod ai;
mod config;
pub mod cto_config;
mod deps;
pub mod docs;
pub mod intake;
pub mod platform_config;
pub mod routing;
mod tags;
mod tasks;

pub use ai::AIDomain;
pub use config::ConfigDomain;
pub use cto_config::{generate_cto_config, save_cto_config, CtoConfig};
pub use deps::DependencyDomain;
pub use intake::{IntakeConfig, IntakeDomain, IntakeResult};
pub use platform_config::{IntakeMode, PlatformConfig};
pub use routing::{
    infer_agent_hint, infer_agent_hint_str, infer_agent_hint_with_deps,
    infer_agent_hint_with_deps_str, is_implementation_agent, parse_agent, Agent,
};
pub use tags::TagsDomain;
pub use tasks::TasksDomain;
