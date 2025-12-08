//! AWS (Amazon Web Services) cloud provider.
//!
//! Implements the [`CloudProvider`] trait for AWS.
//!
//! ## Services
//!
//! - **EKS** (Elastic Kubernetes Service) - Managed Kubernetes
//! - **EC2** (Elastic Compute Cloud) - Virtual machines
//!
//! ## Recommended Instance Types
//!
//! ### For Kubernetes Nodes (EKS or self-managed):
//! - **m6i.large**: 2 vCPU, 8GB RAM - $0.096/hr (~$70/mo)
//! - **m6i.xlarge**: 4 vCPU, 16GB RAM - $0.192/hr (~$140/mo)
//! - **m6i.2xlarge**: 8 vCPU, 32GB RAM - $0.384/hr (~$280/mo)
//! - **r6i.xlarge**: 4 vCPU, 32GB RAM (memory-optimized) - $0.252/hr (~$184/mo)
//!
//! ### For Dedicated Hosts (bare-metal like):
//! - **c6i.metal**: 128 vCPU, 256GB RAM - $5.44/hr (~$3,970/mo)
//! - **m6i.metal**: 128 vCPU, 512GB RAM - $6.12/hr (~$4,470/mo)

mod client;
mod models;

pub use client::Aws;
pub use models::*;

