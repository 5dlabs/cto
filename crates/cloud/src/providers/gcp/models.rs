//! GCP API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// GKE (Kubernetes) types
// ============================================================================

/// GKE cluster information.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GkeCluster {
    /// Cluster name.
    pub name: String,
    /// Cluster self link.
    pub self_link: Option<String>,
    /// Zone/location.
    pub location: String,
    /// Cluster status.
    pub status: String,
    /// Kubernetes master version.
    pub current_master_version: Option<String>,
    /// API server endpoint.
    pub endpoint: Option<String>,
    /// Node pools.
    #[serde(default)]
    pub node_pools: Vec<NodePool>,
    /// Create time.
    pub create_time: Option<String>,
    /// Current node count (total across all pools).
    pub current_node_count: Option<i32>,
}

/// GKE node pool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePool {
    /// Node pool name.
    pub name: String,
    /// Node pool status.
    pub status: Option<String>,
    /// Node config.
    pub config: Option<NodeConfig>,
    /// Initial node count.
    pub initial_node_count: Option<i32>,
    /// Autoscaling config.
    pub autoscaling: Option<NodePoolAutoscaling>,
}

/// Node configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    /// Machine type.
    pub machine_type: String,
    /// Disk size in GB.
    pub disk_size_gb: Option<i32>,
    /// Disk type.
    pub disk_type: Option<String>,
    /// Image type.
    pub image_type: Option<String>,
    /// Service account.
    pub service_account: Option<String>,
}

/// Node pool autoscaling.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePoolAutoscaling {
    /// Enabled.
    pub enabled: bool,
    /// Min node count.
    pub min_node_count: Option<i32>,
    /// Max node count.
    pub max_node_count: Option<i32>,
}

/// Create cluster request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClusterRequest {
    /// Cluster definition.
    pub cluster: ClusterDefinition,
}

/// Cluster definition for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterDefinition {
    /// Cluster name.
    pub name: String,
    /// Initial Kubernetes version.
    pub initial_cluster_version: Option<String>,
    /// Network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Subnetwork.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnetwork: Option<String>,
    /// Node pools.
    pub node_pools: Vec<NodePoolDefinition>,
}

/// Node pool definition for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePoolDefinition {
    /// Node pool name.
    pub name: String,
    /// Initial node count.
    pub initial_node_count: i32,
    /// Node config.
    pub config: NodeConfigDefinition,
    /// Autoscaling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoscaling: Option<NodePoolAutoscalingDefinition>,
}

/// Node config for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfigDefinition {
    /// Machine type.
    pub machine_type: String,
    /// Disk size in GB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_size_gb: Option<i32>,
    /// Disk type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<String>,
    /// Image type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_type: Option<String>,
}

/// Autoscaling definition for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePoolAutoscalingDefinition {
    /// Enabled.
    pub enabled: bool,
    /// Min node count.
    pub min_node_count: i32,
    /// Max node count.
    pub max_node_count: i32,
}

/// Cluster list response.
#[derive(Debug, Deserialize)]
pub struct ClusterListResponse {
    /// List of clusters.
    #[serde(default)]
    pub clusters: Vec<GkeCluster>,
}

// ============================================================================
// Compute Engine (Instance) types
// ============================================================================

/// Compute Engine instance.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GceInstance {
    /// Instance ID.
    pub id: String,
    /// Instance name.
    pub name: String,
    /// Zone.
    pub zone: String,
    /// Machine type (URL).
    pub machine_type: String,
    /// Status.
    pub status: String,
    /// Network interfaces.
    #[serde(default)]
    pub network_interfaces: Vec<NetworkInterface>,
    /// Disks.
    #[serde(default)]
    pub disks: Vec<AttachedDisk>,
    /// Creation timestamp.
    pub creation_timestamp: Option<String>,
    /// Labels.
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}

/// Network interface.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    /// Network (URL).
    pub network: Option<String>,
    /// Subnetwork (URL).
    pub subnetwork: Option<String>,
    /// Network IP (internal).
    pub network_i_p: Option<String>,
    /// Access configs (for external IP).
    #[serde(default)]
    pub access_configs: Vec<AccessConfig>,
}

/// Access configuration (external IP).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    /// Access type.
    #[serde(rename = "type")]
    pub access_type: Option<String>,
    /// External NAT IP.
    pub nat_i_p: Option<String>,
    /// Name.
    pub name: Option<String>,
}

/// Attached disk.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDisk {
    /// Disk source (URL).
    pub source: Option<String>,
    /// Boot disk.
    pub boot: Option<bool>,
    /// Auto delete.
    pub auto_delete: Option<bool>,
    /// Disk size in GB.
    pub disk_size_gb: Option<String>,
}

/// Create instance request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceRequest {
    /// Instance name.
    pub name: String,
    /// Machine type (URL).
    pub machine_type: String,
    /// Disks.
    pub disks: Vec<AttachedDiskDefinition>,
    /// Network interfaces.
    pub network_interfaces: Vec<NetworkInterfaceDefinition>,
    /// Labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<String, String>>,
    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Attached disk definition for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDiskDefinition {
    /// Boot disk.
    pub boot: bool,
    /// Auto delete.
    pub auto_delete: bool,
    /// Initialize params.
    pub initialize_params: InitializeParams,
}

/// Disk initialization parameters.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// Source image (URL).
    pub source_image: String,
    /// Disk size in GB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_size_gb: Option<String>,
    /// Disk type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<String>,
}

/// Network interface definition for creation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceDefinition {
    /// Network (URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Subnetwork (URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnetwork: Option<String>,
    /// Access configs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_configs: Option<Vec<AccessConfigDefinition>>,
}

/// Access config definition.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfigDefinition {
    /// Access type.
    #[serde(rename = "type")]
    pub access_type: String,
    /// Name.
    pub name: String,
}

/// Metadata for instance.
#[derive(Debug, Serialize)]
pub struct Metadata {
    /// Metadata items.
    pub items: Vec<MetadataItem>,
}

/// Metadata item.
#[derive(Debug, Serialize)]
pub struct MetadataItem {
    /// Key.
    pub key: String,
    /// Value.
    pub value: String,
}

/// Instance list response.
#[derive(Debug, Deserialize)]
pub struct InstanceListResponse {
    /// List of instances.
    #[serde(default)]
    pub items: Vec<GceInstance>,
}

// ============================================================================
// Operation types
// ============================================================================

/// GCP operation (async task).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    /// Operation name.
    pub name: String,
    /// Status.
    pub status: String,
    /// Target link.
    pub target_link: Option<String>,
    /// Operation type.
    pub operation_type: Option<String>,
    /// Error.
    pub error: Option<OperationError>,
}

/// Operation error.
#[derive(Debug, Clone, Deserialize)]
pub struct OperationError {
    /// Errors.
    #[serde(default)]
    pub errors: Vec<OperationErrorDetail>,
}

/// Operation error detail.
#[derive(Debug, Clone, Deserialize)]
pub struct OperationErrorDetail {
    /// Error code.
    pub code: Option<String>,
    /// Error message.
    pub message: Option<String>,
}

// ============================================================================
// Common GCP types
// ============================================================================

/// Common GCP regions.
pub mod regions {
    /// US Central (Iowa).
    pub const US_CENTRAL1: &str = "us-central1";
    /// US East (South Carolina).
    pub const US_EAST1: &str = "us-east1";
    /// US East (N. Virginia).
    pub const US_EAST4: &str = "us-east4";
    /// US West (Oregon).
    pub const US_WEST1: &str = "us-west1";
    /// Europe West (Belgium).
    pub const EUROPE_WEST1: &str = "europe-west1";
    /// Europe West (Frankfurt).
    pub const EUROPE_WEST3: &str = "europe-west3";
    /// Asia East (Taiwan).
    pub const ASIA_EAST1: &str = "asia-east1";
    /// Asia Northeast (Tokyo).
    pub const ASIA_NORTHEAST1: &str = "asia-northeast1";
}

/// Common GCP images.
pub mod images {
    /// Ubuntu 24.04 LTS.
    pub const UBUNTU_24_04: &str =
        "projects/ubuntu-os-cloud/global/images/family/ubuntu-2404-lts-amd64";
    /// Ubuntu 22.04 LTS.
    pub const UBUNTU_22_04: &str = "projects/ubuntu-os-cloud/global/images/family/ubuntu-2204-lts";
    /// Debian 12.
    pub const DEBIAN_12: &str = "projects/debian-cloud/global/images/family/debian-12";
    /// Container-Optimized OS.
    pub const COS: &str = "projects/cos-cloud/global/images/family/cos-stable";
}



