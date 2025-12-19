//! Latitude.sh GPU VM provider.
//!
//! Implements the [`GpuProvider`] trait for Latitude.sh GPU VMs.
//!
//! ## Available GPU Plans
//!
//! - **H100** - NVIDIA H100 80GB, 16 vCPU, 128GB RAM (~$1.66/hr)
//! - **L40S** - NVIDIA L40S 48GB, 16 vCPU, 128GB RAM (~$0.74/hr)
//! - **RTX 6000 Pro** - NVIDIA RTX 6000 Ada, 16 vCPU, 128GB RAM (~$2.44/hr)
//!
//! ## Example
//!
//! ```ignore
//! use gpu::providers::latitude::Latitude;
//! use gpu::providers::traits::{GpuProvider, CreateGpuVmRequest};
//!
//! let provider = Latitude::new(api_key, project_id)?;
//!
//! // List plans
//! let plans = provider.list_gpu_plans().await?;
//!
//! // Create a VM
//! let vm = provider.create_gpu_vm(CreateGpuVmRequest {
//!     name: "inference-1".to_string(),
//!     plan_id: "plan_BDXM5EKz0rpkw".to_string(),
//!     ssh_keys: vec![],
//! }).await?;
//! ```

mod client;
mod models;

pub use client::Latitude;
pub use models::*;


