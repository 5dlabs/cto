//! Authentication module for Twitter/X access.
//!
//! Provides browser-based authentication and session management.

mod browser;
mod session;

pub use browser::BrowserAuth;
pub use session::Session;
