//! Bare metal provisioning for CTO Platform.
//!
//! This crate provides abstractions for provisioning bare metal servers
//! from various cloud providers (Latitude.sh, etc.) and bootstrapping
//! them with Talos Linux for Kubernetes clusters.
//!
//! # Example
//!
//! ```rust,ignore
//! use cto_metal::providers::latitude::Latitude;
//! use cto_metal::providers::Provider;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let provider = Latitude::new("api_key", "project_id");
//!     
//!     // Provision a server
//!     let server = provider.create_server(CreateServerRequest {
//!         hostname: "talos-node-1".into(),
//!         plan: "c2-small-x86".into(),
//!         region: "MIA2".into(),
//!     }).await?;
//!     
//!     // Wait for it to be ready
//!     provider.wait_ready(&server.id).await?;
//!     
//!     // Boot Talos via iPXE
//!     provider.reinstall_ipxe(&server.id, &talos_ipxe_url).await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod cilium;
pub mod clustermesh;
pub mod providers;
pub mod stack;
pub mod state;
pub mod talos;

pub use providers::latitude::Latitude;
pub use providers::{CreateServerRequest, Provider, ReinstallIpxeRequest, Server, ServerStatus};

// Re-export Cilium types for convenience
pub use cilium::CiliumConfig;
pub use clustermesh::ClusterMeshStatus;
