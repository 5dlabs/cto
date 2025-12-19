//! Email digest module for research entries.
//!
//! Sends periodic email summaries of new research content with AI analysis.
//!
//! Features:
//! - AI-powered analysis of research entries
//! - Actionable recommendations for the CTO platform
//! - Dark-themed responsive email design
//! - Hybrid triggering: daily scheduled + burst threshold

mod analyzer;
mod config;
mod email;
mod generator;
mod state;

pub use analyzer::{ActionItem, DigestAnalysis, DigestAnalyzer};
pub use config::DigestConfig;
pub use email::EmailSender;
pub use generator::DigestGenerator;
pub use state::DigestState;


