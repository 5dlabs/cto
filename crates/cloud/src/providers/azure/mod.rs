//! Azure (Microsoft Azure) cloud provider.
//!
//! Implements the [`CloudProvider`] trait for Microsoft Azure.
//!
//! ## Services
//!
//! - **AKS** (Azure Kubernetes Service) - Managed Kubernetes
//! - **Azure VMs** (Virtual Machines) - Compute instances
//!
//! ## Recommended Instance Types
//!
//! ### For Kubernetes Nodes (AKS or self-managed):
//! - `Standard_D4s_v5`: 4 vCPU, 16GB RAM - ~$140/mo
//! - `Standard_D8s_v5`: 8 vCPU, 32GB RAM - ~$281/mo
//! - `Standard_E4s_v5`: 4 vCPU, 32GB RAM (memory-optimized) - ~$182/mo
//! - `Standard_F8s_v2`: 8 vCPU, 16GB RAM (compute-optimized) - ~$235/mo
//!
//! ### For Dedicated Hosts:
//! - **DSv3-Type1**: 16 vCPU, 64GB RAM (dedicated) - ~$1,450/mo
//! - **Dsv4-Type1**: 64 vCPU, 256GB RAM (dedicated) - ~$3,500/mo

mod client;
mod models;

pub use client::Azure;
pub use models::*;
