//! GCP API client implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    AccessConfigDefinition, AttachedDiskDefinition, ClusterDefinition, ClusterListResponse,
    CreateClusterRequest as GkeCreateRequest, CreateInstanceRequest as GceCreateRequest,
    GceInstance, GkeCluster, InitializeParams, InstanceListResponse, Metadata, MetadataItem,
    NetworkInterfaceDefinition, NodeConfigDefinition, NodePoolDefinition,
};
use crate::providers::traits::{
    CloudProvider, CloudProviderError, CreateClusterRequest, CreateInstanceRequest, Instance,
    InstanceStatus, KubernetesCluster, KubernetesClusterStatus,
};

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for resources.
const POLL_INTERVAL_SECS: u64 = 15;

/// GCP cloud provider.
#[derive(Clone)]
pub struct Gcp {
    /// HTTP client.
    client: Client,
    /// Project ID.
    project_id: String,
    /// Access token (from service account or user).
    access_token: String,
    /// Default zone.
    zone: String,
}

impl Gcp {
    /// Create a new GCP provider.
    ///
    /// # Arguments
    /// * `project_id` - GCP project ID
    /// * `access_token` - `OAuth2` access token
    /// * `zone` - Default zone (e.g., "us-central1-a")
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        project_id: impl Into<String>,
        access_token: impl Into<String>,
        zone: impl Into<String>,
    ) -> Result<Self, CloudProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(CloudProviderError::Http)?;

        Ok(Self {
            client,
            project_id: project_id.into(),
            access_token: access_token.into(),
            zone: zone.into(),
        })
    }

    /// Get region from zone.
    fn zone_to_region(zone: &str) -> String {
        // Remove the zone suffix (e.g., "us-central1-a" -> "us-central1")
        zone.rsplit_once('-')
            .map_or_else(|| zone.to_string(), |(region, _)| region.to_string())
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

    /// Make an authenticated POST request.
    async fn post<T, B>(&self, url: &str, body: &B) -> Result<T, CloudProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        debug!(url = %url, "POST request");

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request that returns operation.
    async fn post_operation<B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<(), CloudProviderError> {
        debug!(url = %url, "POST request (operation)");

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
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
        if status.is_success() || status == StatusCode::NOT_FOUND {
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

        if status.is_success() {
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

    /// Convert GKE cluster to our type.
    fn to_cluster(cluster: &GkeCluster) -> KubernetesCluster {
        let status = match cluster.status.as_str() {
            "PROVISIONING" => KubernetesClusterStatus::Creating,
            "RUNNING" => KubernetesClusterStatus::Running,
            "RECONCILING" | "UPDATING" => KubernetesClusterStatus::Updating,
            "STOPPING" | "DEGRADED" => KubernetesClusterStatus::Deleting,
            "ERROR" => KubernetesClusterStatus::Error,
            _ => KubernetesClusterStatus::Unknown,
        };

        let node_count = cluster.current_node_count.unwrap_or(0);
        let node_type = cluster
            .node_pools
            .first()
            .and_then(|np| np.config.as_ref())
            .map(|c| c.machine_type.clone())
            .unwrap_or_default();

        KubernetesCluster {
            id: cluster
                .self_link
                .clone()
                .unwrap_or_else(|| cluster.name.clone()),
            name: cluster.name.clone(),
            status,
            version: cluster.current_master_version.clone().unwrap_or_default(),
            region: cluster.location.clone(),
            endpoint: cluster.endpoint.clone(),
            node_count,
            node_type,
            created_at: cluster
                .create_time
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }

    /// Convert GCE instance to our type.
    fn to_instance(instance: &GceInstance) -> Instance {
        let status = match instance.status.as_str() {
            "PROVISIONING" | "STAGING" => InstanceStatus::Pending,
            "RUNNING" => InstanceStatus::Running,
            "STOPPED" | "SUSPENDED" => InstanceStatus::Stopped,
            "STOPPING" | "SUSPENDING" => InstanceStatus::Terminating,
            "TERMINATED" => InstanceStatus::Terminated,
            _ => InstanceStatus::Unknown,
        };

        let public_ip = instance
            .network_interfaces
            .first()
            .and_then(|ni| ni.access_configs.first())
            .and_then(|ac| ac.nat_i_p.clone());

        let private_ip = instance
            .network_interfaces
            .first()
            .and_then(|ni| ni.network_i_p.clone());

        // Extract machine type from URL
        let machine_type = instance
            .machine_type
            .rsplit('/')
            .next()
            .unwrap_or(&instance.machine_type)
            .to_string();

        // Extract zone from URL
        let zone = instance
            .zone
            .rsplit('/')
            .next()
            .unwrap_or(&instance.zone)
            .to_string();

        // Get image from boot disk
        let image = instance
            .disks
            .iter()
            .find(|d| d.boot == Some(true))
            .and_then(|d| d.source.clone())
            .unwrap_or_default();

        Instance {
            id: instance.id.clone(),
            name: instance.name.clone(),
            status,
            instance_type: machine_type,
            region: zone,
            public_ip,
            private_ip,
            image,
            created_at: instance
                .creation_timestamp
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl CloudProvider for Gcp {
    // ========================================================================
    // GKE (Managed Kubernetes) operations
    // ========================================================================

    async fn create_cluster(
        &self,
        req: CreateClusterRequest,
    ) -> Result<KubernetesCluster, CloudProviderError> {
        info!(
            name = %req.name,
            version = %req.version,
            region = %req.region,
            "Creating GKE cluster"
        );

        let body = GkeCreateRequest {
            cluster: ClusterDefinition {
                name: req.name.clone(),
                initial_cluster_version: Some(req.version),
                network: req.network,
                subnetwork: req.subnets.as_ref().and_then(|s| s.first().cloned()),
                node_pools: vec![NodePoolDefinition {
                    name: "default-pool".to_string(),
                    initial_node_count: req.node_count,
                    config: NodeConfigDefinition {
                        machine_type: req.node_type,
                        disk_size_gb: Some(100),
                        disk_type: Some("pd-standard".to_string()),
                        image_type: Some("COS_CONTAINERD".to_string()),
                    },
                    autoscaling: None,
                }],
            },
        };

        // Use zone or region for location
        let location =
            if req.region.contains('-') && req.region.chars().filter(|c| *c == '-').count() == 2 {
                &req.region // It's a zone
            } else {
                &self.zone
            };

        let url = format!(
            "https://container.googleapis.com/v1/projects/{}/locations/{}/clusters",
            self.project_id, location
        );

        let cluster: GkeCluster = self.post(&url, &body).await?;

        info!(
            cluster_name = %cluster.name,
            "GKE cluster creation initiated"
        );

        Ok(Self::to_cluster(&cluster))
    }

    async fn get_cluster(&self, id: &str) -> Result<KubernetesCluster, CloudProviderError> {
        // id can be cluster name or self_link
        let name = if id.starts_with("https://") {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        let url = format!(
            "https://container.googleapis.com/v1/projects/{}/locations/{}/clusters/{}",
            self.project_id,
            Self::zone_to_region(&self.zone),
            name
        );

        let cluster: GkeCluster = self.get(&url).await?;
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
        let name = if id.starts_with("https://") {
            id.rsplit('/').next().unwrap_or(id)
        } else {
            id
        };

        info!(cluster_name = %name, "Deleting GKE cluster");

        let url = format!(
            "https://container.googleapis.com/v1/projects/{}/locations/{}/clusters/{}",
            self.project_id,
            Self::zone_to_region(&self.zone),
            name
        );

        self.delete(&url).await?;

        info!(cluster_name = %name, "GKE cluster deletion initiated");
        Ok(())
    }

    async fn list_clusters(&self) -> Result<Vec<KubernetesCluster>, CloudProviderError> {
        let url = format!(
            "https://container.googleapis.com/v1/projects/{}/locations/-/clusters",
            self.project_id
        );

        let response: ClusterListResponse = self.get(&url).await?;
        Ok(response.clusters.iter().map(Self::to_cluster).collect())
    }

    async fn get_kubeconfig(&self, id: &str) -> Result<String, CloudProviderError> {
        let cluster = self.get_cluster(id).await?;

        let endpoint = cluster.endpoint.ok_or_else(|| {
            CloudProviderError::Config("Cluster endpoint not available".to_string())
        })?;

        // Generate kubeconfig for GKE
        let name = &cluster.name;
        let project = &self.project_id;
        let zone = &self.zone;
        let kubeconfig = format!(
            r"apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://{endpoint}
  name: {name}
contexts:
- context:
    cluster: {name}
    user: {name}
  name: {name}
current-context: {name}
users:
- name: {name}
  user:
    exec:
      apiVersion: client.authentication.k8s.io/v1beta1
      command: gcloud
      args:
        - container
        - clusters
        - get-credentials
        - {name}
        - --project
        - {project}
        - --zone
        - {zone}
"
        );

        Ok(kubeconfig)
    }

    // ========================================================================
    // Compute Engine (Instance) operations
    // ========================================================================

    async fn create_instance(
        &self,
        req: CreateInstanceRequest,
    ) -> Result<Instance, CloudProviderError> {
        info!(
            name = %req.name,
            instance_type = %req.instance_type,
            region = %req.region,
            "Creating Compute Engine instance"
        );

        let zone = if req.region.is_empty() {
            &self.zone
        } else {
            &req.region
        };

        let machine_type_url = format!("zones/{}/machineTypes/{}", zone, req.instance_type);

        let mut metadata_items = Vec::new();
        if let Some(user_data) = req.user_data {
            metadata_items.push(MetadataItem {
                key: "startup-script".to_string(),
                value: user_data,
            });
        }
        if !req.ssh_keys.is_empty() {
            metadata_items.push(MetadataItem {
                key: "ssh-keys".to_string(),
                value: req.ssh_keys.join("\n"),
            });
        }

        let body = GceCreateRequest {
            name: req.name.clone(),
            machine_type: machine_type_url,
            disks: vec![AttachedDiskDefinition {
                boot: true,
                auto_delete: true,
                initialize_params: InitializeParams {
                    source_image: req.image,
                    disk_size_gb: Some("100".to_string()),
                    disk_type: Some(format!("zones/{zone}/diskTypes/pd-standard")),
                },
            }],
            network_interfaces: vec![NetworkInterfaceDefinition {
                network: req.network.map(|n| format!("global/networks/{n}")),
                subnetwork: req.subnet,
                access_configs: Some(vec![AccessConfigDefinition {
                    access_type: "ONE_TO_ONE_NAT".to_string(),
                    name: "External NAT".to_string(),
                }]),
            }],
            labels: Some({
                let mut labels = std::collections::HashMap::new();
                labels.insert("name".to_string(), req.name.clone());
                labels
            }),
            metadata: if metadata_items.is_empty() {
                None
            } else {
                Some(Metadata {
                    items: metadata_items,
                })
            },
        };

        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances",
            self.project_id, zone
        );

        self.post_operation(&url, &body).await?;

        // Fetch the created instance
        info!(instance_name = %req.name, "Fetching created instance");

        // Wait a moment for the instance to be created
        tokio::time::sleep(Duration::from_secs(2)).await;

        let get_url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}",
            self.project_id, zone, req.name
        );

        let instance: GceInstance = self.get(&get_url).await?;

        info!(instance_id = %instance.id, "Instance created");

        Ok(Self::to_instance(&instance))
    }

    async fn get_instance(&self, id: &str) -> Result<Instance, CloudProviderError> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}",
            self.project_id, self.zone, id
        );

        let instance: GceInstance = self.get(&url).await?;
        Ok(Self::to_instance(&instance))
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
        info!(instance_id = %id, "Stopping instance");

        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}/stop",
            self.project_id, self.zone, id
        );

        self.post_operation(&url, &()).await?;

        info!(instance_id = %id, "Instance stop initiated");
        Ok(())
    }

    async fn start_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        info!(instance_id = %id, "Starting instance");

        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}/start",
            self.project_id, self.zone, id
        );

        self.post_operation(&url, &()).await?;

        info!(instance_id = %id, "Instance start initiated");
        Ok(())
    }

    async fn terminate_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        info!(instance_id = %id, "Terminating instance");

        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}",
            self.project_id, self.zone, id
        );

        self.delete(&url).await?;

        info!(instance_id = %id, "Instance termination initiated");
        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<Instance>, CloudProviderError> {
        let url = format!(
            "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances",
            self.project_id, self.zone
        );

        let response: InstanceListResponse = self.get(&url).await?;
        Ok(response.items.iter().map(Self::to_instance).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_to_region() {
        assert_eq!(Gcp::zone_to_region("us-central1-a"), "us-central1");
        assert_eq!(Gcp::zone_to_region("europe-west1-b"), "europe-west1");
    }

    #[test]
    fn test_instance_status_mapping() {
        let gce = GceInstance {
            id: "123456789".to_string(),
            name: "test-instance".to_string(),
            zone: "https://www.googleapis.com/compute/v1/projects/my-project/zones/us-central1-a".to_string(),
            machine_type: "https://www.googleapis.com/compute/v1/projects/my-project/zones/us-central1-a/machineTypes/e2-medium".to_string(),
            status: "RUNNING".to_string(),
            network_interfaces: vec![],
            disks: vec![],
            creation_timestamp: None,
            labels: std::collections::HashMap::new(),
        };

        let converted = Gcp::to_instance(&gce);
        assert_eq!(converted.status, InstanceStatus::Running);
        assert_eq!(converted.id, "123456789");
        assert_eq!(converted.instance_type, "e2-medium");
    }
}
