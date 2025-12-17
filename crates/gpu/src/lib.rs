//! GPU VM provisioning for AI/ML workloads.
//!
//! This crate provides abstractions for provisioning GPU-enabled virtual machines
//! from various cloud providers, specifically optimized for AI/ML inference workloads.
//!
//! ## Architecture
//!
//! GPU VMs are designed to join existing Kubernetes clusters as worker nodes:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     CTO Platform Cluster                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────┐     ┌─────────────────────────┐   │
//! │  │   Control Plane     │     │    GPU Worker Pool      │   │
//! │  │   (Bare Metal)      │◄────│   (GPU VMs)             │   │
//! │  │   • Always on       │     │   • H100, L40S, etc.    │   │
//! │  │   • Stable, cheap   │     │   • Scale on demand     │   │
//! │  └─────────────────────┘     └─────────────────────────┘   │
//! │                                       │                     │
//! │                                       ▼                     │
//! │                        ┌───────────────────────┐            │
//! │                        │   AI Workloads        │            │
//! │                        │   • vLLM, TGI         │            │
//! │                        │   • nvidia.com/gpu    │            │
//! │                        └───────────────────────┘            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Supported Providers
//!
//! - **Latitude.sh** - H100, L40S, RTX 6000 Pro GPU VMs
//!
//! ## Example
//!
//! ```ignore
//! use gpu::providers::latitude::Latitude;
//!
//! let provider = Latitude::new(api_key, project_id)?;
//!
//! // List available GPU plans
//! let plans = provider.list_gpu_plans().await?;
//!
//! // Create a GPU VM
//! let vm = provider.create_gpu_vm("inference-node-1", "plan_BDXM5EKz0rpkw", vec![]).await?;
//!
//! // Delete when done
//! provider.delete_gpu_vm(&vm.id).await?;
//! ```

pub mod providers;

pub use providers::latitude;
pub use providers::traits::{
    CreateGpuVmRequest, GpuPlan, GpuProvider, GpuProviderError, GpuSpecs, GpuVm, GpuVmStatus,
};





