//! GPU provider implementations.
//!
//! This module contains the [`GpuProvider`] trait and implementations
//! for various cloud providers offering GPU VMs.

pub mod latitude;
pub mod traits;

pub use traits::{
    CreateGpuVmRequest, GpuPlan, GpuProvider, GpuProviderError, GpuSpecs, GpuVm, GpuVmStatus,
};

