//! Azure API client implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    AksCluster, AksClusterListResponse, AksCredentials, AzureVm, CreateAgentPoolProfile,
    CreateAksClusterProperties, CreateAksClusterRequest, CreateHardwareProfile,
    CreateStorageProfile, CreateVmProperties, CreateVmRequest, ImageReference, LinuxConfiguration,
    ManagedDisk, ManagedIdentity, NetworkInterfaceReference, NetworkInterfaceReferenceProperties,
    NetworkProfile, OsDisk, OsProfile, SshConfiguration, SshPublicKey, VmListResponse,
};
use crate::providers::traits::{
    CloudProvider, CloudProviderError, CreateClusterRequest, CreateInstanceRequest, Instance,
    InstanceStatus, KubernetesCluster, KubernetesClusterStatus,
};

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for resources.
const POLL_INTERVAL_SECS: u64 = 15;

/// Azure API version for AKS.
const AKS_API_VERSION: &str = "2023-11-01";

/// Azure API version for Compute.
const COMPUTE_API_VERSION: &str = "2023-09-01";

/// Azure cloud provider.
#[derive(Clone)]
pub struct Azure {
    /// HTTP client.
    client: Client,
    /// Subscription ID.
    subscription_id: String,
    /// Resource group.
    resource_group: String,
    /// Access token.
    access_token: String,
    /// Default location.
    location: String,
}

impl Azure {
    /// Create a new Azure provider.
    ///
    /// # Arguments
    /// * `subscription_id` - Azure subscription ID
    /// * `resource_group` - Resource group name
    /// * `access_token` - `OAuth2` access token
    /// * `location` - Default location (e.g., "eastus")
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        subscription_id: impl Into<String>,
        resource_group: impl Into<String>,
        access_token: impl Into<String>,
        location: impl Into<String>,
    ) -> Result<Self, CloudProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(CloudProviderError::Http)?;

        Ok(Self {
            client,
            subscription_id: subscription_id.into(),
            resource_group: resource_group.into(),
            access_token: access_token.into(),
            location: location.into(),
        })
    }

    /// Get Azure Resource Manager base URL.
    fn arm_base_url(&self) -> String {
        format!(
            "https://management.azure.com/subscriptions/{}/resourceGroups/{}",
            self.subscription_id, self.resource_group
        )
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, CloudProviderError> {
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated PUT request.
    async fn put<T, B>(&self, url: &str, body: &B) -> Result<T, CloudProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        debug!(url = %url, "PUT request");

        let response = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request that returns empty body.
    async fn post_empty(&self, url: &str) -> Result<(), CloudProviderError> {
        debug!(url = %url, "POST request (empty)");

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::ACCEPTED {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(CloudProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Make an authenticated DELETE request.
    async fn delete(&self, url: &str) -> Result<(), CloudProviderError> {
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::ACCEPTED || status == StatusCode::NOT_FOUND
        {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(CloudProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Handle API response.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, CloudProviderError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() || status == StatusCode::CREATED {
            serde_json::from_str(&text).map_err(|e| {
                warn!(error = %e, body = %text, "Failed to parse response");
                CloudProviderError::Serialization(e)
            })
        } else if status == StatusCode::NOT_FOUND {
            Err(CloudProviderError::NotFound(text))
        } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            Err(CloudProviderError::Auth(text))
        } else {
            Err(CloudProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Convert AKS cluster to our type.
    fn to_cluster(cluster: &AksCluster) -> KubernetesCluster {
        let status = match cluster.properties.provisioning_state.as_str() {
            "Creating" | "Updating" => KubernetesClusterStatus::Creating,
            "Succeeded" => KubernetesClusterStatus::Running,
            "Scaling" => KubernetesClusterStatus::Updating,
            "Deleting" => KubernetesClusterStatus::Deleting,
            "Failed" => KubernetesClusterStatus::Error,
            _ => KubernetesClusterStatus::Unknown,
        };

        let node_count: i32 = cluster
            .properties
            .agent_pool_profiles
            .iter()
            .map(|p| p.count)
            .sum();

        let node_type = cluster
            .properties
            .agent_pool_profiles
            .first()
            .map(|p| p.vm_size.clone())
            .unwrap_or_default();

        let endpoint = cluster
            .properties
            .fqdn
            .as_ref()
            .map(|fqdn| format!("https://{fqdn}"));

        KubernetesCluster {
            id: cluster.id.clone(),
            name: cluster.name.clone(),
            status,
            version: cluster.properties.kubernetes_version.clone(),
            region: cluster.location.clone(),
            endpoint,
            node_count,
            node_type,
            created_at: None, // Azure doesn't return creation time directly
        }
    }

    /// Convert Azure VM to our type.
    fn to_instance(
        vm: &AzureVm,
        public_ip: Option<String>,
        private_ip: Option<String>,
    ) -> Instance {
        let status = if let Some(ref instance_view) = vm.properties.instance_view {
            instance_view
                .statuses
                .as_ref()
                .and_then(|statuses| {
                    statuses
                        .iter()
                        .find(|s| s.code.starts_with("PowerState/"))
                        .map(|s| match s.code.as_str() {
                            "PowerState/running" => InstanceStatus::Running,
                            "PowerState/stopped" | "PowerState/deallocated" => {
                                InstanceStatus::Stopped
                            }
                            "PowerState/starting" => InstanceStatus::Pending,
                            "PowerState/stopping" | "PowerState/deallocating" => {
                                InstanceStatus::Terminating
                            }
                            _ => InstanceStatus::Unknown,
                        })
                })
                .unwrap_or(InstanceStatus::Unknown)
        } else {
            match vm.properties.provisioning_state.as_str() {
                "Succeeded" => InstanceStatus::Running,
                "Creating" => InstanceStatus::Pending,
                "Deleting" => InstanceStatus::Terminating,
                _ => InstanceStatus::Unknown,
            }
        };

        let instance_type = vm
            .properties
            .hardware_profile
            .as_ref()
            .map(|hp| hp.vm_size.clone())
            .unwrap_or_default();

        let image = vm
            .properties
            .storage_profile
            .as_ref()
            .and_then(|sp| sp.image_reference.as_ref())
            .map(|ir| {
                format!(
                    "{}:{}:{}:{}",
                    ir.publisher.as_deref().unwrap_or(""),
                    ir.offer.as_deref().unwrap_or(""),
                    ir.sku.as_deref().unwrap_or(""),
                    ir.version.as_deref().unwrap_or("")
                )
            })
            .unwrap_or_default();

        Instance {
            id: vm
                .properties
                .vm_id
                .clone()
                .unwrap_or_else(|| vm.name.clone()),
            name: vm.name.clone(),
            status,
            instance_type,
            region: vm.location.clone(),
            public_ip,
            private_ip,
            image,
            created_at: None,
        }
    }
}

#[async_trait]
impl CloudProvider for Azure {
    // ========================================================================
    // AKS (Managed Kubernetes) operations
    // ========================================================================

    async fn create_cluster(
        &self,
        req: CreateClusterRequest,
    ) -> Result<KubernetesCluster, CloudProviderError> {
        info!(
            name = %req.name,
            version = %req.version,
            region = %req.region,
            "Creating AKS cluster"
        );

        let body = CreateAksClusterRequest {
            location: if req.region.is_empty() {
                self.location.clone()
            } else {
                req.region
            },
            tags: None,
            properties: CreateAksClusterProperties {
                kubernetes_version: req.version,
                dns_prefix: req.name.clone(),
                agent_pool_profiles: vec![CreateAgentPoolProfile {
                    name: "nodepool1".to_string(),
                    count: req.node_count,
                    vm_size: req.node_type,
                    os_type: "Linux".to_string(),
                    mode: "System".to_string(),
                }],
                service_principal_profile: None,
                identity: Some(ManagedIdentity {
                    identity_type: "SystemAssigned".to_string(),
                }),
            },
        };

        let url = format!(
            "{}/providers/Microsoft.ContainerService/managedClusters/{}?api-version={}",
            self.arm_base_url(),
            req.name,
            AKS_API_VERSION
        );

        let cluster: AksCluster = self.put(&url, &body).await?;

        info!(
            cluster_name = %cluster.name,
            "AKS cluster creation initiated"
        );

        Ok(Self::to_cluster(&cluster))
    }

    async fn get_cluster(&self, id: &str) -> Result<KubernetesCluster, CloudProviderError> {
        // id can be full resource ID or just the cluster name
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        let url = format!(
            "{}/providers/Microsoft.ContainerService/managedClusters/{}?api-version={}",
            self.arm_base_url(),
            name,
            AKS_API_VERSION
        );

        let cluster: AksCluster = self.get(&url).await?;
        Ok(Self::to_cluster(&cluster))
    }

    async fn wait_cluster_ready(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<KubernetesCluster, CloudProviderError> {
        info!(cluster_id = %id, timeout_secs, "Waiting for cluster to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let cluster = self.get_cluster(id).await?;

            debug!(
                cluster_id = %id,
                status = %cluster.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Polling cluster status"
            );

            if cluster.status == KubernetesClusterStatus::Running {
                info!(cluster_id = %id, "Cluster is ready");
                return Ok(cluster);
            }

            if cluster.status == KubernetesClusterStatus::Error {
                return Err(CloudProviderError::Api {
                    status: 500,
                    message: "Cluster creation failed".to_string(),
                });
            }

            if start.elapsed() > timeout {
                return Err(CloudProviderError::Timeout(timeout_secs));
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn delete_cluster(&self, id: &str) -> Result<(), CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        info!(cluster_name = %name, "Deleting AKS cluster");

        let url = format!(
            "{}/providers/Microsoft.ContainerService/managedClusters/{}?api-version={}",
            self.arm_base_url(),
            name,
            AKS_API_VERSION
        );

        self.delete(&url).await?;

        info!(cluster_name = %name, "AKS cluster deletion initiated");
        Ok(())
    }

    async fn list_clusters(&self) -> Result<Vec<KubernetesCluster>, CloudProviderError> {
        let url = format!(
            "{}/providers/Microsoft.ContainerService/managedClusters?api-version={}",
            self.arm_base_url(),
            AKS_API_VERSION
        );

        let response: AksClusterListResponse = self.get(&url).await?;
        Ok(response.value.iter().map(Self::to_cluster).collect())
    }

    async fn get_kubeconfig(&self, id: &str) -> Result<String, CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        let url = format!(
            "{}/providers/Microsoft.ContainerService/managedClusters/{}/listClusterUserCredential?api-version={}",
            self.arm_base_url(),
            name,
            AKS_API_VERSION
        );

        // Use POST for this endpoint
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        let creds: AksCredentials = self.handle_response(response).await?;

        let kubeconfig = creds
            .kubeconfigs
            .first()
            .ok_or_else(|| CloudProviderError::Config("No kubeconfig available".to_string()))?;

        // Decode base64 kubeconfig
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &kubeconfig.value,
        )
        .map_err(|e| CloudProviderError::Config(format!("Failed to decode kubeconfig: {e}")))?;

        String::from_utf8(decoded)
            .map_err(|e| CloudProviderError::Config(format!("Invalid kubeconfig: {e}")))
    }

    // ========================================================================
    // Azure VM operations
    // ========================================================================

    async fn create_instance(
        &self,
        req: CreateInstanceRequest,
    ) -> Result<Instance, CloudProviderError> {
        info!(
            name = %req.name,
            instance_type = %req.instance_type,
            region = %req.region,
            "Creating Azure VM"
        );

        // Parse image (format: publisher:offer:sku:version)
        let image_parts: Vec<&str> = req.image.split(':').collect();
        let image_reference = if image_parts.len() == 4 {
            ImageReference {
                publisher: Some(image_parts[0].to_string()),
                offer: Some(image_parts[1].to_string()),
                sku: Some(image_parts[2].to_string()),
                version: Some(image_parts[3].to_string()),
                id: None,
            }
        } else {
            // Assume it's a custom image ID
            ImageReference {
                publisher: None,
                offer: None,
                sku: None,
                version: None,
                id: Some(req.image),
            }
        };

        // Build SSH keys
        let ssh_keys: Vec<SshPublicKey> = req
            .ssh_keys
            .iter()
            .map(|key| SshPublicKey {
                path: "/home/azureuser/.ssh/authorized_keys".to_string(),
                key_data: key.clone(),
            })
            .collect();

        // Note: This requires a pre-existing NIC. In production, you'd create the NIC first.
        let nic_id = format!(
            "{}/providers/Microsoft.Network/networkInterfaces/{}-nic",
            self.arm_base_url(),
            req.name
        );

        let body = CreateVmRequest {
            location: if req.region.is_empty() {
                self.location.clone()
            } else {
                req.region
            },
            tags: Some({
                let mut tags = std::collections::HashMap::new();
                tags.insert("Name".to_string(), req.name.clone());
                tags
            }),
            properties: CreateVmProperties {
                hardware_profile: CreateHardwareProfile {
                    vm_size: req.instance_type,
                },
                storage_profile: CreateStorageProfile {
                    image_reference,
                    os_disk: OsDisk {
                        os_type: Some("Linux".to_string()),
                        create_option: "FromImage".to_string(),
                        disk_size_g_b: Some(128),
                        managed_disk: Some(ManagedDisk {
                            storage_account_type: "Premium_LRS".to_string(),
                        }),
                    },
                },
                os_profile: OsProfile {
                    computer_name: req.name.clone(),
                    admin_username: "azureuser".to_string(),
                    admin_password: None,
                    linux_configuration: Some(LinuxConfiguration {
                        disable_password_authentication: true,
                        ssh: if ssh_keys.is_empty() {
                            None
                        } else {
                            Some(SshConfiguration {
                                public_keys: ssh_keys,
                            })
                        },
                    }),
                    custom_data: req.user_data.map(|s| {
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s)
                    }),
                },
                network_profile: NetworkProfile {
                    network_interfaces: vec![NetworkInterfaceReference {
                        id: nic_id,
                        properties: Some(NetworkInterfaceReferenceProperties {
                            primary: Some(true),
                        }),
                    }],
                },
            },
        };

        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}",
            self.arm_base_url(),
            req.name,
            COMPUTE_API_VERSION
        );

        let vm: AzureVm = self.put(&url, &body).await?;

        info!(vm_name = %vm.name, "Azure VM created");

        Ok(Self::to_instance(&vm, None, None))
    }

    async fn get_instance(&self, id: &str) -> Result<Instance, CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}&$expand=instanceView",
            self.arm_base_url(),
            name,
            COMPUTE_API_VERSION
        );

        let vm: AzureVm = self.get(&url).await?;

        // Get IP addresses from network interface
        // Note: This is simplified - in production you'd fetch NIC and public IP details
        Ok(Self::to_instance(&vm, None, None))
    }

    async fn wait_instance_ready(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<Instance, CloudProviderError> {
        info!(instance_id = %id, timeout_secs, "Waiting for instance to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let instance = self.get_instance(id).await?;

            debug!(
                instance_id = %id,
                status = %instance.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Polling instance status"
            );

            if instance.status == InstanceStatus::Running {
                info!(instance_id = %id, "Instance is ready");
                return Ok(instance);
            }

            if instance.status == InstanceStatus::Terminated {
                return Err(CloudProviderError::Api {
                    status: 500,
                    message: "Instance was terminated".to_string(),
                });
            }

            if start.elapsed() > timeout {
                return Err(CloudProviderError::Timeout(timeout_secs));
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn stop_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        info!(instance_id = %name, "Stopping instance");

        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines/{}/deallocate?api-version={}",
            self.arm_base_url(),
            name,
            COMPUTE_API_VERSION
        );

        self.post_empty(&url).await?;

        info!(instance_id = %name, "Instance stop initiated");
        Ok(())
    }

    async fn start_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        info!(instance_id = %name, "Starting instance");

        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines/{}/start?api-version={}",
            self.arm_base_url(),
            name,
            COMPUTE_API_VERSION
        );

        self.post_empty(&url).await?;

        info!(instance_id = %name, "Instance start initiated");
        Ok(())
    }

    async fn terminate_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        let name = if id.contains('/') {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        info!(instance_id = %name, "Terminating instance");

        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines/{}?api-version={}",
            self.arm_base_url(),
            name,
            COMPUTE_API_VERSION
        );

        self.delete(&url).await?;

        info!(instance_id = %name, "Instance termination initiated");
        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<Instance>, CloudProviderError> {
        let url = format!(
            "{}/providers/Microsoft.Compute/virtualMachines?api-version={}",
            self.arm_base_url(),
            COMPUTE_API_VERSION
        );

        let response: VmListResponse = self.get(&url).await?;
        Ok(response
            .value
            .iter()
            .map(|vm| Self::to_instance(vm, None, None))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::azure::models::{AgentPoolProfile, AksClusterProperties};

    #[test]
    fn test_cluster_status_mapping() {
        let cluster = AksCluster {
            id: "/subscriptions/xxx/resourceGroups/rg/providers/Microsoft.ContainerService/managedClusters/test".to_string(),
            name: "test-cluster".to_string(),
            location: "eastus".to_string(),
            tags: std::collections::HashMap::new(),
            properties: AksClusterProperties {
                provisioning_state: "Succeeded".to_string(),
                power_state: None,
                kubernetes_version: "1.28.0".to_string(),
                dns_prefix: Some("test".to_string()),
                fqdn: Some("test-dns.hcp.eastus.azmk8s.io".to_string()),
                agent_pool_profiles: vec![AgentPoolProfile {
                    name: "nodepool1".to_string(),
                    count: 3,
                    vm_size: "Standard_D4s_v5".to_string(),
                    os_type: Some("Linux".to_string()),
                    provisioning_state: Some("Succeeded".to_string()),
                }],
            },
        };

        let converted = Azure::to_cluster(&cluster);
        assert_eq!(converted.status, KubernetesClusterStatus::Running);
        assert_eq!(converted.name, "test-cluster");
        assert_eq!(converted.node_count, 3);
    }
}
