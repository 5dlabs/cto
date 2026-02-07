pub mod error;
pub mod state;
pub mod scaleway;
pub mod talosctl;
pub mod ssh;
pub mod config;

pub use error::{Error, Result};
pub use state::{InstallationState, Orchestrator};
