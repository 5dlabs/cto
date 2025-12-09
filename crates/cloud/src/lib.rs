//! Cloud provider integrations for CTO Platform.
//!
//! This crate provides integrations with major cloud providers:
//!
//! - **AWS** - Amazon Web Services (EKS, EC2)
//! - **GCP** - Google Cloud Platform (GKE, Compute Engine)
//! - **Azure** - Microsoft Azure (AKS, Virtual Machines)
//!
//! ## Deployment Modes
//!
//! Each provider supports two deployment modes:
//!
//! 1. **Managed Kubernetes** - EKS, GKE, AKS
//!    - Fully managed control plane
//!    - Automatic upgrades and patching
//!    - Integrated with cloud provider services
//!
//! 2. **Virtual Machines** - EC2, Compute Engine, Azure VMs
//!    - Self-managed Kubernetes (e.g., Talos Linux)
//!    - Full control over configuration
//!    - Potentially lower cost at scale

pub mod providers;

pub use providers::{
    aws, azure, gcp, CloudProvider, CloudProviderError, CreateClusterRequest,
    CreateInstanceRequest, Instance, InstanceStatus, KubernetesCluster, KubernetesClusterStatus,
};



