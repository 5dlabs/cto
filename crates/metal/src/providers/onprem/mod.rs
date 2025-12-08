//! On-premises / Colocation bare metal provider.
//!
//! Implements the [`Provider`] trait for self-hosted and colocation deployments.
//!
//! ## Overview
//!
//! This provider supports customer-owned hardware, whether in their own data center,
//! colocation facility, or even on-site. It uses a local inventory file to track
//! servers and their state.
//!
//! ## Use Cases
//!
//! - Air-gapped deployments
//! - Regulatory/data sovereignty requirements
//! - Existing hardware investments
//! - Colocation with fixed costs
//! - Home lab / development environments
//!
//! ## Requirements
//!
//! - IPMI/BMC access for power management
//! - Network access to servers for provisioning
//! - PXE boot capability for automated installation

mod client;
mod models;

pub use client::OnPrem;
pub use models::*;


