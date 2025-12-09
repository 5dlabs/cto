//! Integrations for alerting, messaging, and incident management.
//!
//! This crate provides integrations with third-party services for
//! notifications, alerting, and incident management:
//!
//! - **Alerts**: Discord, Slack, PagerDuty, and other notification/incident systems
//!
//! Note: Linear and PM integrations have been moved to the `pm` crate.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod alerts;

pub use alerts::{ChannelError, NotifyChannel, NotifyEvent, Notifier, Severity};
