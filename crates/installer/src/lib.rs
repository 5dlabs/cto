//! CTO Platform Installer Library.
//!
//! This library provides programmatic access to the CTO Platform installer,
//! enabling bare metal Kubernetes cluster provisioning from other crates
//! (e.g., the MCP server).
//!
//! # Example
//!
//! ```ignore
//! use cto_cli::{InstallConfig, Installer};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = InstallConfig::with_defaults("my-cluster".into());
//!     let mut installer = Installer::new_or_resume(config).await?;
//!     installer.run_to_completion().await?;
//!     Ok(())
//! }
//! ```

// Allow product names without backticks in doc comments
#![allow(clippy::doc_markdown)]
// Allow async functions that don't use await (may need await in future)
#![allow(clippy::unused_async)]
// Allow imports after statements in functions
#![allow(clippy::items_after_statements)]

pub mod apps_repo;
pub mod bare_metal;
pub mod bootstrap;
pub mod config;
pub mod gitops;
pub mod kubeconfig;
pub mod openbao;
pub mod orchestrator;
pub mod state;
pub mod ui;
pub mod validation;
pub mod validator;

// Re-export commonly used types at the crate root
pub use config::{BareMetalProvider, InstallConfig, InstallProfile};
pub use orchestrator::Installer;
pub use state::{InstallState, InstallStep, ServerState};
