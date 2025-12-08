//! AWS API client implementation.
//!
//! This client uses AWS SDK-style requests with IAM authentication.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    CreateClusterRequest as EksCreateRequest, CreateVpcConfig, DescribeInstancesResponse,
    Ec2Instance, EksCluster, RunInstancesRequest, Tag, TagSpecification,
};
use crate::providers::traits::{
    CloudProvider, CloudProviderError, CreateClusterRequest, CreateInstanceRequest, Instance,
    InstanceStatus, KubernetesCluster, KubernetesClusterStatus,
};

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for resources.
const POLL_INTERVAL_SECS: u64 = 15;

/// AWS cloud provider.
#[derive(Clone)]
pub struct Aws {
    /// HTTP client.
    client: Client,
    /// AWS access key ID.
    #[allow(dead_code)]
    access_key_id: String,
    /// AWS secret access key.
    #[allow(dead_code)]
    secret_access_key: String,
    /// AWS region.
    region: String,
    /// IAM role ARN for EKS clusters.
    eks_role_arn: Option<String>,
    /// IAM role ARN for EKS node groups.
    node_role_arn: Option<String>,
}

impl Aws {
    /// Create a new AWS provider.
    ///
    /// # Arguments
    /// * `access_key_id` - AWS access key ID
    /// * `secret_access_key` - AWS secret access key
    /// * `region` - AWS region (e.g., "us-east-1")
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, CloudProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(CloudProviderError::Http)?;

        Ok(Self {
            client,
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            region: region.into(),
            eks_role_arn: None,
            node_role_arn: None,
        })
    }

    /// Set the EKS cluster role ARN.
    #[must_use]
    pub fn with_eks_role(mut self, role_arn: impl Into<String>) -> Self {
        self.eks_role_arn = Some(role_arn.into());
        self
    }

    /// Set the EKS node group role ARN.
    #[must_use]
    pub fn with_node_role(mut self, role_arn: impl Into<String>) -> Self {
        self.node_role_arn = Some(role_arn.into());
        self
    }

    /// Get EKS API endpoint.
    fn eks_endpoint(&self) -> String {
        format!("https://eks.{}.amazonaws.com", self.region)
    }

    /// Get EC2 API endpoint.
    fn ec2_endpoint(&self) -> String {
        format!("https://ec2.{}.amazonaws.com", self.region)
    }

    /// Sign and execute an AWS request.
    /// Note: In production, use aws-sigv4 crate for proper request signing.
    async fn aws_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T, CloudProviderError> {
        debug!(url = %url, method = %method, "AWS request");

        // Note: This is a simplified implementation.
        // In production, implement AWS SigV4 signing.
        let mut request = self.client.request(method, url);

        // Add basic headers (real implementation needs SigV4 signing)
        request = request.header("Content-Type", "application/json").header(
            "X-Amz-Date",
            chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
        );

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        self.handle_response(response).await
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

    /// Convert EKS cluster to our type.
    fn to_cluster(cluster: &EksCluster, node_count: i32, node_type: &str) -> KubernetesCluster {
        let status = match cluster.status.as_str() {
            "CREATING" => KubernetesClusterStatus::Creating,
            "ACTIVE" => KubernetesClusterStatus::Running,
            "UPDATING" => KubernetesClusterStatus::Updating,
            "DELETING" => KubernetesClusterStatus::Deleting,
            "FAILED" => KubernetesClusterStatus::Error,
            _ => KubernetesClusterStatus::Unknown,
        };

        // Extract region from ARN
        let region = cluster
            .arn
            .split(':')
            .nth(3)
            .unwrap_or("unknown")
            .to_string();

        KubernetesCluster {
            id: cluster.arn.clone(),
            name: cluster.name.clone(),
            status,
            version: cluster.version.clone(),
            region,
            endpoint: cluster.endpoint.clone(),
            node_count,
            node_type: node_type.to_string(),
            created_at: cluster
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }

    /// Convert EC2 instance to our type.
    fn to_instance(ec2: &Ec2Instance) -> Instance {
        let status = match ec2.state.name.as_str() {
            "pending" => InstanceStatus::Pending,
            "running" => InstanceStatus::Running,
            "stopped" | "stopping" => InstanceStatus::Stopped,
            "shutting-down" => InstanceStatus::Terminating,
            "terminated" => InstanceStatus::Terminated,
            _ => InstanceStatus::Unknown,
        };

        let name = ec2
            .tags
            .iter()
            .find(|t| t.key == "Name")
            .map_or_else(|| ec2.instance_id.clone(), |t| t.value.clone());

        let region = ec2
            .placement
            .as_ref()
            .map(|p| p.availability_zone.clone())
            .unwrap_or_default();

        Instance {
            id: ec2.instance_id.clone(),
            name,
            status,
            instance_type: ec2.instance_type.clone(),
            region,
            public_ip: ec2.public_ip_address.clone(),
            private_ip: ec2.private_ip_address.clone(),
            image: ec2.image_id.clone(),
            created_at: ec2
                .launch_time
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }
}

#[async_trait]
impl CloudProvider for Aws {
    // ========================================================================
    // EKS (Managed Kubernetes) operations
    // ========================================================================

    async fn create_cluster(
        &self,
        req: CreateClusterRequest,
    ) -> Result<KubernetesCluster, CloudProviderError> {
        let role_arn = self
            .eks_role_arn
            .as_ref()
            .ok_or_else(|| CloudProviderError::Config("EKS role ARN not configured".to_string()))?;

        info!(
            name = %req.name,
            version = %req.version,
            region = %req.region,
            "Creating EKS cluster"
        );

        let subnets = req.subnets.unwrap_or_default();
        if subnets.is_empty() {
            return Err(CloudProviderError::Config(
                "Subnets are required for EKS cluster creation".to_string(),
            ));
        }

        let body = EksCreateRequest {
            name: req.name.clone(),
            version: Some(req.version),
            role_arn: role_arn.clone(),
            resources_vpc_config: CreateVpcConfig {
                subnet_ids: subnets,
                security_group_ids: None,
                endpoint_public_access: Some(true),
                endpoint_private_access: Some(true),
            },
            tags: None,
        };

        let url = format!("{}/clusters", self.eks_endpoint());
        let cluster: EksCluster = self
            .aws_request(reqwest::Method::POST, &url, Some(&body))
            .await?;

        info!(
            cluster_name = %cluster.name,
            arn = %cluster.arn,
            "EKS cluster created"
        );

        Ok(Self::to_cluster(&cluster, req.node_count, &req.node_type))
    }

    async fn get_cluster(&self, id: &str) -> Result<KubernetesCluster, CloudProviderError> {
        // Extract cluster name from ARN if needed
        let name = if id.starts_with("arn:") {
            id.split('/').next_back().unwrap_or(id)
        } else {
            id
        };

        let url = format!("{}/clusters/{}", self.eks_endpoint(), name);
        let cluster: EksCluster = self
            .aws_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;

        // In real implementation, get node count from node groups
        Ok(Self::to_cluster(&cluster, 0, ""))
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
        let name = if id.starts_with("arn:") {
            id.split('/').next_back().unwrap_or(id)
        } else {
            id
        };

        info!(cluster_name = %name, "Deleting EKS cluster");

        let url = format!("{}/clusters/{}", self.eks_endpoint(), name);
        self.aws_request::<serde_json::Value>(reqwest::Method::DELETE, &url, None::<&()>)
            .await?;

        info!(cluster_name = %name, "EKS cluster deleted");
        Ok(())
    }

    async fn list_clusters(&self) -> Result<Vec<KubernetesCluster>, CloudProviderError> {
        let url = format!("{}/clusters", self.eks_endpoint());
        let response: serde_json::Value = self
            .aws_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;

        let cluster_names: Vec<String> = response
            .get("clusters")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let mut clusters = Vec::new();
        for name in cluster_names {
            if let Ok(cluster) = self.get_cluster(&name).await {
                clusters.push(cluster);
            }
        }

        Ok(clusters)
    }

    async fn get_kubeconfig(&self, id: &str) -> Result<String, CloudProviderError> {
        let cluster = self.get_cluster(id).await?;

        let endpoint = cluster.endpoint.ok_or_else(|| {
            CloudProviderError::Config("Cluster endpoint not available".to_string())
        })?;

        // Generate kubeconfig for EKS
        let name = &cluster.name;
        let region = &self.region;
        let kubeconfig = format!(
            r"apiVersion: v1
kind: Config
clusters:
- cluster:
    server: {endpoint}
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
      command: aws
      args:
        - eks
        - get-token
        - --cluster-name
        - {name}
        - --region
        - {region}
"
        );

        Ok(kubeconfig)
    }

    // ========================================================================
    // EC2 (Instance) operations
    // ========================================================================

    async fn create_instance(
        &self,
        req: CreateInstanceRequest,
    ) -> Result<Instance, CloudProviderError> {
        info!(
            name = %req.name,
            instance_type = %req.instance_type,
            region = %req.region,
            "Creating EC2 instance"
        );

        let body = RunInstancesRequest {
            image_id: req.image,
            instance_type: req.instance_type,
            min_count: 1,
            max_count: 1,
            key_name: req.ssh_keys.first().cloned(),
            security_group_ids: None,
            subnet_id: req.subnet,
            user_data: req
                .user_data
                .map(|s| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s)),
            tag_specifications: Some(vec![TagSpecification {
                resource_type: "instance".to_string(),
                tags: vec![Tag {
                    key: "Name".to_string(),
                    value: req.name,
                }],
            }]),
        };

        let url = format!(
            "{}/?Action=RunInstances&Version=2016-11-15",
            self.ec2_endpoint()
        );
        let response: serde_json::Value = self
            .aws_request(reqwest::Method::POST, &url, Some(&body))
            .await?;

        // Parse response to get instance
        let instances: Vec<Ec2Instance> = response
            .get("Instances")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let ec2 = instances.first().ok_or_else(|| CloudProviderError::Api {
            status: 500,
            message: "No instance returned from creation".to_string(),
        })?;

        info!(instance_id = %ec2.instance_id, "EC2 instance created");

        Ok(Self::to_instance(ec2))
    }

    async fn get_instance(&self, id: &str) -> Result<Instance, CloudProviderError> {
        let url = format!(
            "{}/?Action=DescribeInstances&Version=2016-11-15&InstanceId.1={}",
            self.ec2_endpoint(),
            id
        );

        let response: DescribeInstancesResponse = self
            .aws_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;

        let ec2 = response
            .reservations
            .first()
            .and_then(|r| r.instances.first())
            .ok_or_else(|| CloudProviderError::NotFound(format!("Instance not found: {id}")))?;

        Ok(Self::to_instance(ec2))
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
            "{}/?Action=StopInstances&Version=2016-11-15&InstanceId.1={}",
            self.ec2_endpoint(),
            id
        );

        self.aws_request::<serde_json::Value>(reqwest::Method::POST, &url, None::<&()>)
            .await?;

        info!(instance_id = %id, "Instance stop initiated");
        Ok(())
    }

    async fn start_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        info!(instance_id = %id, "Starting instance");

        let url = format!(
            "{}/?Action=StartInstances&Version=2016-11-15&InstanceId.1={}",
            self.ec2_endpoint(),
            id
        );

        self.aws_request::<serde_json::Value>(reqwest::Method::POST, &url, None::<&()>)
            .await?;

        info!(instance_id = %id, "Instance start initiated");
        Ok(())
    }

    async fn terminate_instance(&self, id: &str) -> Result<(), CloudProviderError> {
        info!(instance_id = %id, "Terminating instance");

        let url = format!(
            "{}/?Action=TerminateInstances&Version=2016-11-15&InstanceId.1={}",
            self.ec2_endpoint(),
            id
        );

        self.aws_request::<serde_json::Value>(reqwest::Method::POST, &url, None::<&()>)
            .await?;

        info!(instance_id = %id, "Instance termination initiated");
        Ok(())
    }

    async fn list_instances(&self) -> Result<Vec<Instance>, CloudProviderError> {
        let url = format!(
            "{}/?Action=DescribeInstances&Version=2016-11-15",
            self.ec2_endpoint()
        );

        let response: DescribeInstancesResponse = self
            .aws_request(reqwest::Method::GET, &url, None::<&()>)
            .await?;

        let instances: Vec<Instance> = response
            .reservations
            .iter()
            .flat_map(|r| r.instances.iter())
            .map(Self::to_instance)
            .collect();

        Ok(instances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::aws::models::{InstanceState, Placement};

    #[test]
    fn test_instance_status_mapping() {
        let ec2 = Ec2Instance {
            instance_id: "i-1234567890abcdef0".to_string(),
            instance_type: "m6i.large".to_string(),
            state: InstanceState {
                code: 16,
                name: "running".to_string(),
            },
            public_ip_address: Some("1.2.3.4".to_string()),
            private_ip_address: Some("10.0.0.1".to_string()),
            image_id: "ami-12345678".to_string(),
            placement: Some(Placement {
                availability_zone: "us-east-1a".to_string(),
                tenancy: None,
            }),
            launch_time: None,
            tags: vec![Tag {
                key: "Name".to_string(),
                value: "test-instance".to_string(),
            }],
            key_name: Some("my-key".to_string()),
            vpc_id: None,
            subnet_id: None,
        };

        let converted = Aws::to_instance(&ec2);
        assert_eq!(converted.status, InstanceStatus::Running);
        assert_eq!(converted.id, "i-1234567890abcdef0");
        assert_eq!(converted.name, "test-instance");
    }
}
