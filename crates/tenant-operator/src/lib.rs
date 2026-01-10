//! Tenant Operator for CTO `SaaS` Platform
//!
//! This operator watches Tenant Custom Resources and provisions:
//! - Kubernetes namespaces for tenant isolation
//! - RBAC (`ServiceAccounts`, `RoleBindings`)
//! - `ExternalSecrets` for `OpenBao` integration
//! - `ArgoCD` Applications for tenant agent deployments

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod controller;
pub mod crd;
pub mod error;
pub mod resources;

pub use controller::run_controller;
pub use crd::{Tenant, TenantSpec, TenantStatus};
pub use error::Error;
