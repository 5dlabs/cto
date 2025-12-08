//! Cloud provider trait and common types.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during cloud provider operations.
#[derive(Error, Debug)]
pub enum CloudProviderError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Operation timed out.
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Quota exceeded.
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}

// ============================================================================
// Kubernetes Cluster types
// ============================================================================

/// Kubernetes cluster status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KubernetesClusterStatus {
    /// Cluster is being created.
    Creating,
    /// Cluster is active and running.
    Running,
    /// Cluster is being updated.
    Updating,
    /// Cluster is being deleted.
    Deleting,
    /// Cluster is in an error state.
    Error,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for KubernetesClusterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creating => write!(f, "creating"),
            Self::Running => write!(f, "running"),
            Self::Updating => write!(f, "updating"),
            Self::Deleting => write!(f, "deleting"),
            Self::Error => write!(f, "error"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A managed Kubernetes cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesCluster {
    /// Unique cluster identifier.
    pub id: String,
    /// Cluster name.
    pub name: String,
    /// Current status.
    pub status: KubernetesClusterStatus,
    /// Kubernetes version.
    pub version: String,
    /// Region/location.
    pub region: String,
    /// API server endpoint.
    pub endpoint: Option<String>,
    /// Number of nodes.
    pub node_count: i32,
    /// Node instance type/size.
    pub node_type: String,
    /// When the cluster was created.
    pub created_at: Option<DateTime<Utc>>,
}

/// Request to create a new Kubernetes cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClusterRequest {
    /// Cluster name.
    pub name: String,
    /// Kubernetes version.
    pub version: String,
    /// Region/location to deploy in.
    pub region: String,
    /// Number of nodes.
    pub node_count: i32,
    /// Node instance type/size.
    pub node_type: String,
    /// VPC/network to deploy in (optional).
    pub network: Option<String>,
    /// Subnet(s) to deploy in (optional).
    pub subnets: Option<Vec<String>>,
}

// ============================================================================
// Instance (VM) types
// ============================================================================

/// Instance (VM) status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// Instance is pending creation.
    Pending,
    /// Instance is running.
    Running,
    /// Instance is stopped.
    Stopped,
    /// Instance is being terminated.
    Terminating,
    /// Instance is terminated.
    Terminated,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
            Self::Terminating => write!(f, "terminating"),
            Self::Terminated => write!(f, "terminated"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A virtual machine instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// Unique instance identifier.
    pub id: String,
    /// Instance name.
    pub name: String,
    /// Current status.
    pub status: InstanceStatus,
    /// Instance type/size.
    pub instance_type: String,
    /// Region/zone.
    pub region: String,
    /// Public IPv4 address.
    pub public_ip: Option<String>,
    /// Private IPv4 address.
    pub private_ip: Option<String>,
    /// OS image used.
    pub image: String,
    /// When the instance was created.
    pub created_at: Option<DateTime<Utc>>,
}

/// Request to create a new instance (VM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstanceRequest {
    /// Instance name.
    pub name: String,
    /// Instance type/size.
    pub instance_type: String,
    /// Region/zone to deploy in.
    pub region: String,
    /// OS image to use.
    pub image: String,
    /// SSH key names/IDs.
    pub ssh_keys: Vec<String>,
    /// VPC/network (optional).
    pub network: Option<String>,
    /// Subnet (optional).
    pub subnet: Option<String>,
    /// User data/cloud-init script (optional).
    pub user_data: Option<String>,
}

/// Trait for cloud providers.
#[async_trait]
pub trait CloudProvider: Send + Sync {
    // ========================================================================
    // Managed Kubernetes operations
    // ========================================================================

    /// Create a new managed Kubernetes cluster.
    async fn create_cluster(
        &self,
        req: CreateClusterRequest,
    ) -> Result<KubernetesCluster, CloudProviderError>;

    /// Get cluster by ID.
    async fn get_cluster(&self, id: &str) -> Result<KubernetesCluster, CloudProviderError>;

    /// Wait for cluster to be running.
    async fn wait_cluster_ready(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<KubernetesCluster, CloudProviderError>;

    /// Delete a cluster.
    async fn delete_cluster(&self, id: &str) -> Result<(), CloudProviderError>;

    /// List all clusters.
    async fn list_clusters(&self) -> Result<Vec<KubernetesCluster>, CloudProviderError>;

    /// Get kubeconfig for a cluster.
    async fn get_kubeconfig(&self, id: &str) -> Result<String, CloudProviderError>;

    // ========================================================================
    // Instance (VM) operations
    // ========================================================================

    /// Create a new instance (VM).
    async fn create_instance(
        &self,
        req: CreateInstanceRequest,
    ) -> Result<Instance, CloudProviderError>;

    /// Get instance by ID.
    async fn get_instance(&self, id: &str) -> Result<Instance, CloudProviderError>;

    /// Wait for instance to be running.
    async fn wait_instance_ready(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<Instance, CloudProviderError>;

    /// Stop an instance.
    async fn stop_instance(&self, id: &str) -> Result<(), CloudProviderError>;

    /// Start an instance.
    async fn start_instance(&self, id: &str) -> Result<(), CloudProviderError>;

    /// Terminate an instance.
    async fn terminate_instance(&self, id: &str) -> Result<(), CloudProviderError>;

    /// List all instances.
    async fn list_instances(&self) -> Result<Vec<Instance>, CloudProviderError>;
}

