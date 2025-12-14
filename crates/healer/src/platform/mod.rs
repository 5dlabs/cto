//! Platform self-healing module for CTO infrastructure.
//!
//! This module monitors and remediates issues in the CTO platform itself:
//! - Controller pod failures
//! - Healer pod failures  
//! - PM pod failures
//! - BuildKit issues
//! - Stuck CodeRuns
//! - Argo Workflow failures
//!
//! It receives alerts from Alertmanager and spawns remediation CodeRuns
//! targeting the `5dlabs/cto` repository.

pub mod alerts;
pub mod server;
pub mod types;
pub mod workflow;

pub use alerts::PlatformAlertHandler;
pub use server::{build_platform_router, PlatformServerState};
pub use types::{AlertmanagerPayload, PlatformAlert, PlatformIssue, RemediationTarget};
pub use workflow::WorkflowRemediator;
