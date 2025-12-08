//! Cloud provider abstractions.
//!
//! This module defines the common traits and types for cloud providers.

pub mod aws;
pub mod azure;
pub mod gcp;
mod traits;

pub use traits::{
    CloudProvider, CloudProviderError, CreateClusterRequest, CreateInstanceRequest, Instance,
    InstanceStatus, KubernetesCluster, KubernetesClusterStatus,
};

// Re-export provider clients
pub use aws::Aws;
pub use azure::Azure;
pub use gcp::Gcp;


