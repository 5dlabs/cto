//! GCP (Google Cloud Platform) provider.
//!
//! Implements the [`CloudProvider`] trait for Google Cloud.
//!
//! ## Services
//!
//! - **GKE** (Google Kubernetes Engine) - Managed Kubernetes
//! - **Compute Engine** - Virtual machines
//!
//! ## Recommended Instance Types
//!
//! ### For Kubernetes Nodes (GKE or self-managed):
//! - **e2-standard-4**: 4 vCPU, 16GB RAM - ~$97/mo
//! - **e2-standard-8**: 8 vCPU, 32GB RAM - ~$195/mo
//! - **n2-standard-4**: 4 vCPU, 16GB RAM (Intel Ice Lake) - ~$117/mo
//! - **n2-highmem-4**: 4 vCPU, 32GB RAM (memory-optimized) - ~$154/mo
//!
//! ### For Sole-tenant Nodes (dedicated):
//! - **n2-node-80-640**: 80 vCPU, 640GB RAM - ~$4.20/hr (~$3,066/mo)

mod client;
mod models;

pub use client::Gcp;
pub use models::*;
