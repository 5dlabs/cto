//! Azure API request and response models.

use serde::{Deserialize, Serialize};

// ============================================================================
// AKS (Kubernetes) types
// ============================================================================

/// AKS managed cluster.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AksCluster {
    /// Resource ID.
    pub id: String,
    /// Cluster name.
    pub name: String,
    /// Location.
    pub location: String,
    /// Resource tags.
    #[serde(default)]
    pub tags: std::collections::HashMap<String, String>,
    /// Cluster properties.
    pub properties: AksClusterProperties,
}

/// AKS cluster properties.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AksClusterProperties {
    /// Provisioning state.
    pub provisioning_state: String,
    /// Power state.
    pub power_state: Option<PowerState>,
    /// Kubernetes version.
    pub kubernetes_version: String,
    /// DNS prefix.
    pub dns_prefix: Option<String>,
    /// FQDN.
    pub fqdn: Option<String>,
    /// Agent pool profiles.
    #[serde(default)]
    pub agent_pool_profiles: Vec<AgentPoolProfile>,
}

/// Power state.
#[derive(Debug, Clone, Deserialize)]
pub struct PowerState {
    /// Code.
    pub code: String,
}

/// Agent pool profile.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPoolProfile {
    /// Pool name.
    pub name: String,
    /// Node count.
    pub count: i32,
    /// VM size.
    pub vm_size: String,
    /// OS type.
    pub os_type: Option<String>,
    /// Provisioning state.
    pub provisioning_state: Option<String>,
}

/// Create AKS cluster request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAksClusterRequest {
    /// Location.
    pub location: String,
    /// Tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<std::collections::HashMap<String, String>>,
    /// Properties.
    pub properties: CreateAksClusterProperties,
}

/// Create AKS cluster properties.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAksClusterProperties {
    /// Kubernetes version.
    pub kubernetes_version: String,
    /// DNS prefix.
    pub dns_prefix: String,
    /// Agent pool profiles.
    pub agent_pool_profiles: Vec<CreateAgentPoolProfile>,
    /// Service principal profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_principal_profile: Option<ServicePrincipalProfile>,
    /// Identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<ManagedIdentity>,
}

/// Create agent pool profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentPoolProfile {
    /// Pool name.
    pub name: String,
    /// Node count.
    pub count: i32,
    /// VM size.
    pub vm_size: String,
    /// OS type.
    pub os_type: String,
    /// Mode (System or User).
    pub mode: String,
}

/// Service principal profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePrincipalProfile {
    /// Client ID.
    pub client_id: String,
    /// Secret.
    pub secret: String,
}

/// Managed identity.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedIdentity {
    /// Identity type.
    #[serde(rename = "type")]
    pub identity_type: String,
}

/// AKS cluster list response.
#[derive(Debug, Deserialize)]
pub struct AksClusterListResponse {
    /// List of clusters.
    #[serde(default)]
    pub value: Vec<AksCluster>,
    /// Next link for pagination.
    pub next_link: Option<String>,
}

/// AKS credentials response.
#[derive(Debug, Deserialize)]
pub struct AksCredentials {
    /// Kubeconfigs.
    #[serde(default)]
    pub kubeconfigs: Vec<KubeconfigEntry>,
}

/// Kubeconfig entry.
#[derive(Debug, Deserialize)]
pub struct KubeconfigEntry {
    /// Kubeconfig name.
    pub name: String,
    /// Kubeconfig value (base64 encoded).
    pub value: String,
}

// ============================================================================
// Azure VM types
// ============================================================================

/// Azure virtual machine.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureVm {
    /// Resource ID.
    pub id: String,
    /// VM name.
    pub name: String,
    /// Location.
    pub location: String,
    /// Tags.
    #[serde(default)]
    pub tags: std::collections::HashMap<String, String>,
    /// VM properties.
    pub properties: AzureVmProperties,
}

/// Azure VM properties.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureVmProperties {
    /// Provisioning state.
    pub provisioning_state: String,
    /// VM ID.
    pub vm_id: Option<String>,
    /// Hardware profile.
    pub hardware_profile: Option<HardwareProfile>,
    /// Storage profile.
    pub storage_profile: Option<StorageProfile>,
    /// OS profile.
    pub os_profile: Option<OsProfile>,
    /// Network profile.
    pub network_profile: Option<NetworkProfile>,
    /// Instance view (for power state).
    pub instance_view: Option<InstanceView>,
}

/// Hardware profile.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareProfile {
    /// VM size.
    pub vm_size: String,
}

/// Storage profile.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageProfile {
    /// Image reference.
    pub image_reference: Option<ImageReference>,
    /// OS disk.
    pub os_disk: Option<OsDisk>,
}

/// Image reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageReference {
    /// Publisher.
    pub publisher: Option<String>,
    /// Offer.
    pub offer: Option<String>,
    /// SKU.
    pub sku: Option<String>,
    /// Version.
    pub version: Option<String>,
    /// Image ID (for custom images).
    pub id: Option<String>,
}

/// OS disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsDisk {
    /// OS type.
    pub os_type: Option<String>,
    /// Create option.
    pub create_option: String,
    /// Disk size in GB.
    pub disk_size_g_b: Option<i32>,
    /// Managed disk.
    pub managed_disk: Option<ManagedDisk>,
}

/// Managed disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDisk {
    /// Storage account type.
    pub storage_account_type: String,
}

/// OS profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsProfile {
    /// Computer name.
    pub computer_name: String,
    /// Admin username.
    pub admin_username: String,
    /// Admin password (for Windows).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_password: Option<String>,
    /// Linux configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux_configuration: Option<LinuxConfiguration>,
    /// Custom data (cloud-init, base64 encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_data: Option<String>,
}

/// Linux configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxConfiguration {
    /// Disable password authentication.
    pub disable_password_authentication: bool,
    /// SSH configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshConfiguration>,
}

/// SSH configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConfiguration {
    /// Public keys.
    pub public_keys: Vec<SshPublicKey>,
}

/// SSH public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshPublicKey {
    /// Path (e.g., `/home/azureuser/.ssh/authorized_keys`).
    pub path: String,
    /// Key data.
    pub key_data: String,
}

/// Network profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProfile {
    /// Network interfaces.
    pub network_interfaces: Vec<NetworkInterfaceReference>,
}

/// Network interface reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceReference {
    /// Network interface ID.
    pub id: String,
    /// Properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<NetworkInterfaceReferenceProperties>,
}

/// Network interface reference properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceReferenceProperties {
    /// Primary.
    pub primary: Option<bool>,
}

/// Instance view.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceView {
    /// Power state.
    pub statuses: Option<Vec<InstanceViewStatus>>,
}

/// Instance view status.
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceViewStatus {
    /// Status code.
    pub code: String,
    /// Display status.
    pub display_status: Option<String>,
}

/// Create VM request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVmRequest {
    /// Location.
    pub location: String,
    /// Tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<std::collections::HashMap<String, String>>,
    /// Properties.
    pub properties: CreateVmProperties,
}

/// Create VM properties.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVmProperties {
    /// Hardware profile.
    pub hardware_profile: CreateHardwareProfile,
    /// Storage profile.
    pub storage_profile: CreateStorageProfile,
    /// OS profile.
    pub os_profile: OsProfile,
    /// Network profile.
    pub network_profile: NetworkProfile,
}

/// Create hardware profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHardwareProfile {
    /// VM size.
    pub vm_size: String,
}

/// Create storage profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStorageProfile {
    /// Image reference.
    pub image_reference: ImageReference,
    /// OS disk.
    pub os_disk: OsDisk,
}

/// VM list response.
#[derive(Debug, Deserialize)]
pub struct VmListResponse {
    /// List of VMs.
    #[serde(default)]
    pub value: Vec<AzureVm>,
    /// Next link for pagination.
    pub next_link: Option<String>,
}

// ============================================================================
// Network types
// ============================================================================

/// Network interface.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    /// Resource ID.
    pub id: String,
    /// Name.
    pub name: String,
    /// Properties.
    pub properties: NetworkInterfaceProperties,
}

/// Network interface properties.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceProperties {
    /// IP configurations.
    #[serde(default)]
    pub ip_configurations: Vec<IpConfiguration>,
}

/// IP configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpConfiguration {
    /// Properties.
    pub properties: Option<IpConfigurationProperties>,
}

/// IP configuration properties.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpConfigurationProperties {
    /// Private IP address.
    pub private_i_p_address: Option<String>,
    /// Public IP address.
    pub public_i_p_address: Option<PublicIpAddressReference>,
}

/// Public IP address reference.
#[derive(Debug, Clone, Deserialize)]
pub struct PublicIpAddressReference {
    /// Public IP resource ID.
    pub id: String,
}

/// Public IP address.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpAddress {
    /// Resource ID.
    pub id: String,
    /// Properties.
    pub properties: Option<PublicIpAddressProperties>,
}

/// Public IP address properties.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIpAddressProperties {
    /// IP address.
    pub ip_address: Option<String>,
}

// ============================================================================
// Common Azure types
// ============================================================================

/// Common Azure regions.
pub mod regions {
    /// East US.
    pub const EAST_US: &str = "eastus";
    /// East US 2.
    pub const EAST_US_2: &str = "eastus2";
    /// West US.
    pub const WEST_US: &str = "westus";
    /// West US 2.
    pub const WEST_US_2: &str = "westus2";
    /// West Europe.
    pub const WEST_EUROPE: &str = "westeurope";
    /// North Europe.
    pub const NORTH_EUROPE: &str = "northeurope";
    /// UK South.
    pub const UK_SOUTH: &str = "uksouth";
    /// Southeast Asia.
    pub const SOUTHEAST_ASIA: &str = "southeastasia";
}

/// Common Azure VM images.
pub mod images {
    /// Ubuntu 24.04 LTS.
    pub const UBUNTU_24_04: (&str, &str, &str, &str) =
        ("Canonical", "ubuntu-24_04-lts", "server", "latest");
    /// Ubuntu 22.04 LTS.
    pub const UBUNTU_22_04: (&str, &str, &str, &str) = (
        "Canonical",
        "0001-com-ubuntu-server-jammy",
        "22_04-lts-gen2",
        "latest",
    );
    /// Debian 12.
    pub const DEBIAN_12: (&str, &str, &str, &str) = ("Debian", "debian-12", "12", "latest");
}
