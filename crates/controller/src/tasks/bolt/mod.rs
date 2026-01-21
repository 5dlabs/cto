//! BoltRun Controller Module
//!
//! BOLT-002: Controller Reconciliation
//! BOLT-003: Credential Injection
//! BOLT-004: Progress Reporting
//!
//! Watches BoltRun CRDs and spawns Bolt agent pods for admin provisioning tasks.

mod controller;
mod resources;
mod status;

pub use controller::reconcile_bolt_run;
pub use resources::create_bolt_job;
pub use status::update_bolt_status;
