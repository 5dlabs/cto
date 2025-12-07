# Cloud Provider Implementation Plan

This document outlines the implementation status and plan for managed Kubernetes cluster providers (EKS, GKE, AKS) in the CTO Platform.

## Overview

The `cto-cloud` crate provides integrations with major cloud providers for:

1. **Managed Kubernetes** - EKS, GKE, AKS (fully managed control planes)
2. **Virtual Machines** - EC2, Compute Engine, Azure VMs (for self-managed Kubernetes like Talos)

**Key Difference from Bare-Metal**: Cloud managed Kubernetes (EKS/GKE/AKS) does NOT use Talos Linux. The cloud provider manages the control plane and node OS.

## Implementation Status

| Provider | Service | Create | Get | List | Delete | Wait | Kubeconfig | Tests |
|----------|---------|--------|-----|------|--------|------|------------|-------|
| **AWS** | EKS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **AWS** | EC2 | ✅ | ✅ | ✅ | ✅ | ✅ | N/A | ⚠️ |
| **GCP** | GKE | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **GCP** | Compute Engine | ✅ | ✅ | ✅ | ✅ | ✅ | N/A | ⚠️ |
| **Azure** | AKS | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **Azure** | VMs | ✅ | ✅ | ✅ | ✅ | ✅ | N/A | ⚠️ |

Legend: ✅ Implemented | ⚠️ Needs Unit Tests | ❌ Not Implemented

## Architecture

### CloudProvider Trait

```rust
#[async_trait]
pub trait CloudProvider: Send + Sync {
    // Managed Kubernetes operations
    async fn create_cluster(&self, req: CreateClusterRequest) -> Result<KubernetesCluster, CloudProviderError>;
    async fn get_cluster(&self, id: &str) -> Result<KubernetesCluster, CloudProviderError>;
    async fn wait_cluster_ready(&self, id: &str, timeout_secs: u64) -> Result<KubernetesCluster, CloudProviderError>;
    async fn delete_cluster(&self, id: &str) -> Result<(), CloudProviderError>;
    async fn list_clusters(&self) -> Result<Vec<KubernetesCluster>, CloudProviderError>;
    async fn get_kubeconfig(&self, id: &str) -> Result<String, CloudProviderError>;

    // Instance (VM) operations
    async fn create_instance(&self, req: CreateInstanceRequest) -> Result<Instance, CloudProviderError>;
    async fn get_instance(&self, id: &str) -> Result<Instance, CloudProviderError>;
    async fn wait_instance_ready(&self, id: &str, timeout_secs: u64) -> Result<Instance, CloudProviderError>;
    async fn stop_instance(&self, id: &str) -> Result<(), CloudProviderError>;
    async fn start_instance(&self, id: &str) -> Result<(), CloudProviderError>;
    async fn terminate_instance(&self, id: &str) -> Result<(), CloudProviderError>;
    async fn list_instances(&self) -> Result<Vec<Instance>, CloudProviderError>;
}
```

### Deployment Modes

#### 1. Managed Kubernetes (EKS, GKE, AKS)
- Cloud provider manages the control plane
- Automatic upgrades and patching available
- Integrated with cloud provider IAM, networking, monitoring
- **No Talos installation** - uses cloud provider's node OS

#### 2. Self-Managed on VMs (EC2, Compute Engine, Azure VMs)
- Full control over Kubernetes configuration
- Can use Talos Linux or other node OS
- More complex setup but more flexibility

## Provider Details

### AWS EKS

**Crate:** `crates/cloud/src/providers/aws/`

**Authentication:**
- Access Key ID + Secret Access Key
- Optional: STS assume role for cross-account access

**Key Configuration:**
- `eks_role_arn` - IAM role for EKS control plane
- `node_role_arn` - IAM role for node groups

**API Endpoints:**
- EKS: `https://eks.{region}.amazonaws.com`
- EC2: `https://ec2.{region}.amazonaws.com`

**Kubeconfig:**
Uses `aws eks get-token` for authentication via exec plugin.

**Official SDK:** `aws-sdk-eks` (available, migration recommended)

```rust
// Example usage
let aws = Aws::new(access_key, secret_key, "us-east-1")?
    .with_eks_role("arn:aws:iam::123456789012:role/EksClusterRole")
    .with_node_role("arn:aws:iam::123456789012:role/EksNodeRole");

let cluster = aws.create_cluster(CreateClusterRequest {
    name: "my-cluster".to_string(),
    version: "1.29".to_string(),
    region: "us-east-1".to_string(),
    node_count: 3,
    node_type: "m6i.large".to_string(),
    subnets: Some(vec!["subnet-xxx".to_string()]),
    ..Default::default()
}).await?;
```

### GCP GKE

**Crate:** `crates/cloud/src/providers/gcp/`

**Authentication:**
- OAuth2 access token (from service account or user credentials)

**Key Configuration:**
- `project_id` - GCP project ID
- `zone` - Default zone for resources

**API Endpoints:**
- GKE: `https://container.googleapis.com/v1`
- Compute: `https://compute.googleapis.com/compute/v1`

**Kubeconfig:**
Uses `gcloud container clusters get-credentials` for authentication via exec plugin.

**Official SDK:** `google-cloud-container-v1` (available, migration recommended)

```rust
// Example usage
let gcp = Gcp::new("my-project", access_token, "us-central1-a")?;

let cluster = gcp.create_cluster(CreateClusterRequest {
    name: "my-cluster".to_string(),
    version: "1.29".to_string(),
    region: "us-central1".to_string(),
    node_count: 3,
    node_type: "e2-standard-4".to_string(),
    ..Default::default()
}).await?;
```

### Azure AKS

**Crate:** `crates/cloud/src/providers/azure/`

**Authentication:**
- OAuth2 access token (from service principal or managed identity)

**Key Configuration:**
- `subscription_id` - Azure subscription ID
- `resource_group` - Resource group name
- `location` - Default Azure region

**API Endpoints:**
- AKS: `https://management.azure.com/.../Microsoft.ContainerService/managedClusters`
- Compute: `https://management.azure.com/.../Microsoft.Compute/virtualMachines`

**API Versions:**
- AKS: `2023-11-01`
- Compute: `2023-09-01`

**Kubeconfig:**
Retrieved via `listClusterUserCredential` API (base64 encoded).

**Official SDK:** `azure_mgmt_containerservice` (available, migration recommended)

```rust
// Example usage
let azure = Azure::new(subscription_id, "my-rg", access_token, "eastus")?;

let cluster = azure.create_cluster(CreateClusterRequest {
    name: "my-cluster".to_string(),
    version: "1.29".to_string(),
    region: "eastus".to_string(),
    node_count: 3,
    node_type: "Standard_D4s_v5".to_string(),
    ..Default::default()
}).await?;
```

## Using Existing Clusters

All providers support connecting to existing clusters:

```rust
// Option 1: List clusters and select one
let clusters = provider.list_clusters().await?;
let existing = clusters.iter().find(|c| c.name == "my-existing-cluster");

// Option 2: Get cluster directly by name/ID
let cluster = provider.get_cluster("my-existing-cluster").await?;

// Option 3: Get kubeconfig for any cluster
let kubeconfig = provider.get_kubeconfig("my-existing-cluster").await?;
```

## Recommended Improvements

### 1. Migrate to Official SDKs

The current implementations use raw HTTP requests. Migration to official SDKs provides:
- Better authentication handling (credential chains, refresh tokens)
- Request signing (AWS SigV4)
- Automatic retries and error handling
- Type-safe API bindings

**Recommended crates:**
- AWS: `aws-sdk-eks` (v1.x, stable)
- GCP: `google-cloud-container-v1` (v1.2.0)
- Azure: `azure_mgmt_containerservice` (v0.10.0, preview)

### 2. Add Comprehensive Tests

Each provider needs:
- Unit tests with `wiremock` for API mocking
- Integration tests with real cloud accounts (optional, requires credentials)
- Status mapping tests (already partial)

### 3. Add Node Pool Management

Extend the trait for node pool operations:
```rust
async fn create_node_pool(&self, cluster_id: &str, req: CreateNodePoolRequest) -> Result<NodePool, CloudProviderError>;
async fn scale_node_pool(&self, cluster_id: &str, pool_id: &str, count: i32) -> Result<(), CloudProviderError>;
async fn delete_node_pool(&self, cluster_id: &str, pool_id: &str) -> Result<(), CloudProviderError>;
```

### 4. Add Cluster Upgrades

```rust
async fn upgrade_cluster(&self, id: &str, version: &str) -> Result<KubernetesCluster, CloudProviderError>;
async fn get_available_versions(&self) -> Result<Vec<String>, CloudProviderError>;
```

## Comparison: Cloud vs Bare-Metal

| Feature | Cloud (EKS/GKE/AKS) | Bare-Metal (Latitude/etc.) |
|---------|---------------------|---------------------------|
| Control Plane | Managed by provider | Self-managed (Talos) |
| Node OS | Provider managed | Talos Linux via iPXE |
| Upgrades | Provider assisted | Manual Talos upgrades |
| Networking | Provider CNI options | Custom CNI (Cilium) |
| Cost Model | Per-cluster + nodes | Server rental only |
| Setup Time | 10-20 minutes | 30-60 minutes |

## File Structure

```
crates/cloud/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports
│   └── providers/
│       ├── mod.rs          # Provider module
│       ├── traits.rs       # CloudProvider trait & types
│       ├── aws/
│       │   ├── mod.rs
│       │   ├── client.rs   # Aws implementation
│       │   └── models.rs   # EKS/EC2 API types
│       ├── gcp/
│       │   ├── mod.rs
│       │   ├── client.rs   # Gcp implementation
│       │   └── models.rs   # GKE/Compute API types
│       └── azure/
│           ├── mod.rs
│           ├── client.rs   # Azure implementation
│           └── models.rs   # AKS/VM API types
```

## Next Steps

1. **Short-term**: Add unit tests for all providers using `wiremock`
2. **Medium-term**: Migrate AWS provider to `aws-sdk-eks`
3. **Long-term**: Add node pool management and cluster upgrade support

## References

### AWS EKS
- [EKS API Reference](https://docs.aws.amazon.com/eks/latest/APIReference/)
- [aws-sdk-eks Rust Crate](https://crates.io/crates/aws-sdk-eks)
- [EKS Code Examples](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_eks_code_examples.html)

### GCP GKE
- [GKE REST API](https://docs.cloud.google.com/kubernetes-engine/docs/reference/rest/v1/projects.locations.clusters)
- [google-cloud-container-v1 Crate](https://crates.io/crates/google-cloud-container-v1)
- [GKE Managing Clusters](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/managing-clusters)

### Azure AKS
- [AKS REST API](https://learn.microsoft.com/en-us/rest/api/aks/)
- [azure_mgmt_containerservice Crate](https://crates.io/crates/azure_mgmt_containerservice)
- [Azure SDK for Rust](https://github.com/Azure/azure-sdk-for-rust)

