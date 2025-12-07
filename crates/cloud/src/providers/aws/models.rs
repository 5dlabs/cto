//! AWS API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// EKS (Kubernetes) types
// ============================================================================

/// EKS cluster information.
#[derive(Debug, Clone, Deserialize)]
pub struct EksCluster {
    /// Cluster name.
    pub name: String,
    /// Cluster ARN.
    pub arn: String,
    /// Kubernetes version.
    pub version: String,
    /// Cluster status.
    pub status: String,
    /// API server endpoint.
    pub endpoint: Option<String>,
    /// Role ARN.
    pub role_arn: Option<String>,
    /// VPC configuration.
    pub resources_vpc_config: Option<VpcConfig>,
    /// Created at timestamp.
    pub created_at: Option<String>,
    /// Tags.
    #[serde(default)]
    pub tags: std::collections::HashMap<String, String>,
}

/// VPC configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct VpcConfig {
    /// Subnet IDs.
    pub subnet_ids: Option<Vec<String>>,
    /// Security group IDs.
    pub security_group_ids: Option<Vec<String>>,
    /// VPC ID.
    pub vpc_id: Option<String>,
    /// Public access enabled.
    pub endpoint_public_access: Option<bool>,
    /// Private access enabled.
    pub endpoint_private_access: Option<bool>,
}

/// EKS node group.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeGroup {
    /// Node group name.
    pub node_group_name: String,
    /// Cluster name.
    pub cluster_name: String,
    /// Node group ARN.
    pub node_group_arn: String,
    /// Status.
    pub status: String,
    /// Instance types.
    pub instance_types: Option<Vec<String>>,
    /// Scaling configuration.
    pub scaling_config: Option<ScalingConfig>,
    /// Subnet IDs.
    pub subnets: Option<Vec<String>>,
    /// AMI type.
    pub ami_type: Option<String>,
    /// Disk size in GB.
    pub disk_size: Option<i32>,
}

/// Scaling configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ScalingConfig {
    /// Minimum size.
    pub min_size: i32,
    /// Maximum size.
    pub max_size: i32,
    /// Desired size.
    pub desired_size: i32,
}

/// Create cluster request.
#[derive(Debug, Serialize)]
pub struct CreateClusterRequest {
    /// Cluster name.
    pub name: String,
    /// Kubernetes version.
    pub version: Option<String>,
    /// Role ARN.
    #[serde(rename = "roleArn")]
    pub role_arn: String,
    /// VPC configuration.
    #[serde(rename = "resourcesVpcConfig")]
    pub resources_vpc_config: CreateVpcConfig,
    /// Tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<std::collections::HashMap<String, String>>,
}

/// Create VPC configuration.
#[derive(Debug, Serialize)]
pub struct CreateVpcConfig {
    /// Subnet IDs.
    #[serde(rename = "subnetIds")]
    pub subnet_ids: Vec<String>,
    /// Security group IDs.
    #[serde(rename = "securityGroupIds", skip_serializing_if = "Option::is_none")]
    pub security_group_ids: Option<Vec<String>>,
    /// Enable public access.
    #[serde(
        rename = "endpointPublicAccess",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint_public_access: Option<bool>,
    /// Enable private access.
    #[serde(
        rename = "endpointPrivateAccess",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint_private_access: Option<bool>,
}

/// Create node group request.
#[derive(Debug, Serialize)]
pub struct CreateNodeGroupRequest {
    /// Node group name.
    #[serde(rename = "nodegroupName")]
    pub nodegroup_name: String,
    /// Scaling configuration.
    #[serde(rename = "scalingConfig")]
    pub scaling_config: CreateScalingConfig,
    /// Subnet IDs.
    pub subnets: Vec<String>,
    /// Instance types.
    #[serde(rename = "instanceTypes", skip_serializing_if = "Option::is_none")]
    pub instance_types: Option<Vec<String>>,
    /// AMI type.
    #[serde(rename = "amiType", skip_serializing_if = "Option::is_none")]
    pub ami_type: Option<String>,
    /// Node role ARN.
    #[serde(rename = "nodeRole")]
    pub node_role: String,
    /// Disk size in GB.
    #[serde(rename = "diskSize", skip_serializing_if = "Option::is_none")]
    pub disk_size: Option<i32>,
}

/// Create scaling configuration.
#[derive(Debug, Serialize)]
pub struct CreateScalingConfig {
    /// Minimum size.
    #[serde(rename = "minSize")]
    pub min_size: i32,
    /// Maximum size.
    #[serde(rename = "maxSize")]
    pub max_size: i32,
    /// Desired size.
    #[serde(rename = "desiredSize")]
    pub desired_size: i32,
}

// ============================================================================
// EC2 (Instance) types
// ============================================================================

/// EC2 instance information.
#[derive(Debug, Clone, Deserialize)]
pub struct Ec2Instance {
    /// Instance ID.
    #[serde(rename = "InstanceId")]
    pub instance_id: String,
    /// Instance type.
    #[serde(rename = "InstanceType")]
    pub instance_type: String,
    /// Instance state.
    #[serde(rename = "State")]
    pub state: InstanceState,
    /// Public IP address.
    #[serde(rename = "PublicIpAddress")]
    pub public_ip_address: Option<String>,
    /// Private IP address.
    #[serde(rename = "PrivateIpAddress")]
    pub private_ip_address: Option<String>,
    /// Image ID (AMI).
    #[serde(rename = "ImageId")]
    pub image_id: String,
    /// Availability zone.
    #[serde(rename = "Placement")]
    pub placement: Option<Placement>,
    /// Launch time.
    #[serde(rename = "LaunchTime")]
    pub launch_time: Option<String>,
    /// Tags.
    #[serde(rename = "Tags", default)]
    pub tags: Vec<Tag>,
    /// Key name.
    #[serde(rename = "KeyName")]
    pub key_name: Option<String>,
    /// VPC ID.
    #[serde(rename = "VpcId")]
    pub vpc_id: Option<String>,
    /// Subnet ID.
    #[serde(rename = "SubnetId")]
    pub subnet_id: Option<String>,
}

/// Instance state.
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceState {
    /// State code.
    #[serde(rename = "Code")]
    pub code: i32,
    /// State name.
    #[serde(rename = "Name")]
    pub name: String,
}

/// Placement information.
#[derive(Debug, Clone, Deserialize)]
pub struct Placement {
    /// Availability zone.
    #[serde(rename = "AvailabilityZone")]
    pub availability_zone: String,
    /// Tenancy.
    #[serde(rename = "Tenancy")]
    pub tenancy: Option<String>,
}

/// Tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag key.
    #[serde(rename = "Key")]
    pub key: String,
    /// Tag value.
    #[serde(rename = "Value")]
    pub value: String,
}

/// Run instances request.
#[derive(Debug, Serialize)]
pub struct RunInstancesRequest {
    /// Image ID (AMI).
    #[serde(rename = "ImageId")]
    pub image_id: String,
    /// Instance type.
    #[serde(rename = "InstanceType")]
    pub instance_type: String,
    /// Minimum count.
    #[serde(rename = "MinCount")]
    pub min_count: i32,
    /// Maximum count.
    #[serde(rename = "MaxCount")]
    pub max_count: i32,
    /// Key name.
    #[serde(rename = "KeyName", skip_serializing_if = "Option::is_none")]
    pub key_name: Option<String>,
    /// Security group IDs.
    #[serde(rename = "SecurityGroupIds", skip_serializing_if = "Option::is_none")]
    pub security_group_ids: Option<Vec<String>>,
    /// Subnet ID.
    #[serde(rename = "SubnetId", skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
    /// User data (base64 encoded).
    #[serde(rename = "UserData", skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
    /// Tags.
    #[serde(rename = "TagSpecifications", skip_serializing_if = "Option::is_none")]
    pub tag_specifications: Option<Vec<TagSpecification>>,
}

/// Tag specification for instance creation.
#[derive(Debug, Serialize)]
pub struct TagSpecification {
    /// Resource type.
    #[serde(rename = "ResourceType")]
    pub resource_type: String,
    /// Tags.
    #[serde(rename = "Tags")]
    pub tags: Vec<Tag>,
}

/// Describe instances response.
#[derive(Debug, Deserialize)]
pub struct DescribeInstancesResponse {
    /// Reservations.
    #[serde(rename = "Reservations")]
    pub reservations: Vec<Reservation>,
}

/// Reservation.
#[derive(Debug, Deserialize)]
pub struct Reservation {
    /// Instances.
    #[serde(rename = "Instances")]
    pub instances: Vec<Ec2Instance>,
}

// ============================================================================
// Common AWS types
// ============================================================================

/// AWS region information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// Region name (e.g., "us-east-1").
    pub region_name: String,
    /// Region endpoint.
    pub endpoint: String,
}

/// Common AWS regions.
pub mod regions {
    /// US East (N. Virginia).
    pub const US_EAST_1: &str = "us-east-1";
    /// US East (Ohio).
    pub const US_EAST_2: &str = "us-east-2";
    /// US West (N. California).
    pub const US_WEST_1: &str = "us-west-1";
    /// US West (Oregon).
    pub const US_WEST_2: &str = "us-west-2";
    /// EU (Ireland).
    pub const EU_WEST_1: &str = "eu-west-1";
    /// EU (Frankfurt).
    pub const EU_CENTRAL_1: &str = "eu-central-1";
    /// Asia Pacific (Tokyo).
    pub const AP_NORTHEAST_1: &str = "ap-northeast-1";
    /// Asia Pacific (Singapore).
    pub const AP_SOUTHEAST_1: &str = "ap-southeast-1";
}

/// Common AMI IDs for Ubuntu.
pub mod amis {
    /// Ubuntu 24.04 LTS (us-east-1).
    pub const UBUNTU_24_04_US_EAST_1: &str = "ami-0e001c9271cf7f3b9";
    /// Ubuntu 22.04 LTS (us-east-1).
    pub const UBUNTU_22_04_US_EAST_1: &str = "ami-0557a15b87f6559cf";
}

