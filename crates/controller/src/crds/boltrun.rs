//! `BoltRun` Custom Resource Definition for admin provisioning tasks
//!
//! BOLT-001: BoltRun CRD Schema Definition
//! Defines BoltRun Custom Resource Definition for admin provisioning tasks like
//! cluster provisioning, debugging, upgrades, and destruction.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default timeout for BoltRun tasks
fn default_timeout() -> String {
    "30m".to_string()
}

/// Default retry limit
fn default_retry_limit() -> u32 {
    3
}

/// Default model for Bolt agent
fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

/// Task types that BoltRun can perform
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BoltTaskType {
    /// Provision a new cluster
    #[default]
    Provision,
    /// Debug an existing cluster
    Debug,
    /// Upgrade an existing cluster
    Upgrade,
    /// Destroy a cluster
    Destroy,
}

/// Bare metal provider options
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BareMetalProvider {
    #[default]
    Latitude,
    Hetzner,
    Ovh,
    Vultr,
    Scaleway,
}

/// Cluster size options
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClusterSize {
    /// 2 nodes: 1 CP + 1 worker
    Small,
    /// 4 nodes: 1 CP + 3 workers
    #[default]
    Medium,
    /// 8 nodes: 3 CP (HA) + 5 workers
    Large,
}

/// Provisioning configuration for new clusters
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionConfig {
    /// Bare metal provider to use
    pub provider: BareMetalProvider,

    /// Region to deploy in (provider-specific, e.g., "DAL", "FRA")
    pub region: String,

    /// Cluster size determining node count and specs
    #[serde(default)]
    pub cluster_size: ClusterSize,

    /// Reference to credentials in OpenBao (path)
    /// Path format: tenants/{tenant-id}/provider-creds
    pub credential_ref: String,

    /// Custom cluster name (optional, derived from tenant if not set)
    #[serde(default)]
    pub cluster_name: Option<String>,

    /// Talos Linux version to install
    #[serde(default = "default_talos_version")]
    pub talos_version: String,

    /// Enable WARP Connector for connectivity
    #[serde(default = "default_true")]
    pub enable_warp_connector: bool,

    /// Enable Cilium ClusterMesh for multi-cluster networking
    #[serde(default = "default_true")]
    pub enable_cluster_mesh: bool,

    /// GITOPS-003: Customer-owned GitOps repository URL
    /// ArgoCD on the cluster syncs from this repo (e.g., "https://github.com/customer-org/cto-argocd")
    #[serde(default)]
    pub gitops_repo: Option<String>,
}

fn default_talos_version() -> String {
    "v1.9.0".to_string()
}

fn default_true() -> bool {
    true
}

/// Execution configuration for BoltRun tasks
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionConfig {
    /// Timeout for the entire task
    #[serde(default = "default_timeout")]
    pub timeout: String,

    /// Maximum retry attempts on failure
    #[serde(default = "default_retry_limit")]
    pub retry_limit: u32,

    /// Model to use for Bolt agent
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            retry_limit: default_retry_limit(),
            model: default_model(),
        }
    }
}

/// BOLT-001: BoltRun CRD for admin provisioning tasks
///
/// CRD group: cto.5dlabs.ai, version: v1alpha1, kind: BoltRun
/// Spec includes: tenantRef, taskType (provision/debug/upgrade/destroy)
/// Spec includes: provision config (provider, region, clusterSize, credentialRef)
/// Spec includes: execution config (timeout, retryLimit, model)
/// Status includes: phase, startTime, completionTime, currentStep, logs
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(group = "cto.5dlabs.ai", version = "v1alpha1", kind = "BoltRun")]
#[kube(namespaced)]
#[kube(status = "BoltRunStatus")]
#[kube(shortname = "br")]
#[kube(printcolumn = r#"{"name":"Tenant","type":"string","jsonPath":".spec.tenantRef"}"#)]
#[kube(printcolumn = r#"{"name":"Task","type":"string","jsonPath":".spec.taskType"}"#)]
#[kube(
    printcolumn = r#"{"name":"Provider","type":"string","jsonPath":".spec.provision.provider"}"#
)]
#[kube(printcolumn = r#"{"name":"Region","type":"string","jsonPath":".spec.provision.region"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Step","type":"string","jsonPath":".status.currentStep"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
#[serde(rename_all = "camelCase")]
pub struct BoltRunSpec {
    /// Reference to the tenant this task is for
    pub tenant_ref: String,

    /// Type of task to perform
    #[serde(default)]
    pub task_type: BoltTaskType,

    /// Provisioning configuration (required for provision task type)
    #[serde(default)]
    pub provision: Option<ProvisionConfig>,

    /// Execution configuration
    #[serde(default)]
    pub execution: ExecutionConfig,

    /// BOLT-003: Reference to ExternalSecret for credential injection
    /// When set, an ExternalSecret is created to inject credentials into the pod
    #[serde(default)]
    pub external_secret_ref: Option<String>,
}

/// Phase of the BoltRun task
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum BoltRunPhase {
    /// Task is pending, waiting to start
    #[default]
    Pending,
    /// Task is initializing
    Initializing,
    /// Task is running
    Running,
    /// Task succeeded
    Succeeded,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// BOLT-004: Progress Reporting - Current step information
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoltRunStep {
    /// Step number (1-31 for full provisioning)
    pub number: u32,

    /// Total steps expected
    pub total: u32,

    /// Step name (e.g., "CreatingServers", "DeployingWarpConnector")
    pub name: String,

    /// Started at timestamp (RFC3339)
    #[serde(default)]
    pub started_at: Option<String>,

    /// Completed at timestamp (RFC3339)
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Status of the BoltRun task
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoltRunStatus {
    /// Current phase of the task
    #[serde(default)]
    pub phase: BoltRunPhase,

    /// Human-readable status message
    #[serde(default)]
    pub message: Option<String>,

    /// When the task started
    #[serde(default)]
    pub start_time: Option<String>,

    /// When the task completed (success or failure)
    #[serde(default)]
    pub completion_time: Option<String>,

    /// BOLT-004: Current step progress
    #[serde(default)]
    pub current_step: Option<BoltRunStep>,

    /// Last update timestamp
    #[serde(default)]
    pub last_update: Option<String>,

    /// Error details if failed
    #[serde(default)]
    pub error: Option<String>,

    /// Associated Kubernetes Job name
    #[serde(default)]
    pub job_name: Option<String>,

    /// Pod name running the task
    #[serde(default)]
    pub pod_name: Option<String>,

    /// URL to detailed logs
    #[serde(default)]
    pub logs_url: Option<String>,

    /// Cluster name if provisioned
    #[serde(default)]
    pub cluster_name: Option<String>,

    /// Kubeconfig path if provisioned
    #[serde(default)]
    pub kubeconfig_path: Option<String>,

    /// Retry count
    #[serde(default)]
    pub retry_count: Option<u32>,

    /// BOLT-003: ExternalSecret created for credential injection
    #[serde(default)]
    pub external_secret_name: Option<String>,

    /// Conditions for detailed status tracking
    #[serde(default)]
    pub conditions: Vec<BoltRunCondition>,
}

/// Condition for detailed BoltRun status
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoltRunCondition {
    /// Type of condition (e.g., "Ready", "ServersCreated", "ClusterBootstrapped")
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition (True, False, Unknown)
    pub status: String,

    /// Last time the condition transitioned (RFC3339)
    #[serde(default)]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition
    #[serde(default)]
    pub reason: Option<String>,

    /// Human-readable message about the condition
    #[serde(default)]
    pub message: Option<String>,
}

impl BoltRun {
    /// Generate the Job name for this BoltRun
    #[must_use]
    pub fn job_name(&self) -> String {
        format!(
            "bolt-{}",
            self.metadata
                .name
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        )
    }

    /// Generate the ExternalSecret name for credential injection
    #[must_use]
    pub fn external_secret_name(&self) -> String {
        format!(
            "{}-creds",
            self.metadata
                .name
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        )
    }

    /// Get the cluster name for this provisioning task
    #[must_use]
    pub fn cluster_name(&self) -> String {
        if let Some(ref provision) = self.spec.provision {
            if let Some(ref name) = provision.cluster_name {
                return name.clone();
            }
        }
        format!("{}-prod", self.spec.tenant_ref.replace("tenant-", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boltrun_spec_serialization() {
        let spec = BoltRunSpec {
            tenant_ref: "tenant-acme".to_string(),
            task_type: BoltTaskType::Provision,
            provision: Some(ProvisionConfig {
                provider: BareMetalProvider::Latitude,
                region: "DAL".to_string(),
                cluster_size: ClusterSize::Medium,
                credential_ref: "tenants/tenant-acme/provider-creds".to_string(),
                cluster_name: None,
                talos_version: "v1.9.0".to_string(),
                enable_warp_connector: true,
                enable_cluster_mesh: true,
                gitops_repo: Some("https://github.com/acme-corp/cto-argocd".to_string()),
            }),
            execution: ExecutionConfig::default(),
            external_secret_ref: None,
        };

        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(json.contains("tenant-acme"));
        assert!(json.contains("provision"));
        assert!(json.contains("latitude"));
    }

    #[test]
    fn test_boltrun_status_phases() {
        let status = BoltRunStatus {
            phase: BoltRunPhase::Running,
            message: Some("Creating servers".to_string()),
            current_step: Some(BoltRunStep {
                number: 4,
                total: 31,
                name: "CreatingServers".to_string(),
                started_at: Some("2026-01-20T15:00:00Z".to_string()),
                completed_at: None,
            }),
            ..Default::default()
        };

        assert_eq!(status.phase, BoltRunPhase::Running);
        assert!(status.current_step.is_some());
        let step = status.current_step.unwrap();
        assert_eq!(step.number, 4);
        assert_eq!(step.name, "CreatingServers");
    }

    #[test]
    fn test_cluster_size_configs() {
        assert_eq!(
            serde_json::to_string(&ClusterSize::Small).unwrap(),
            "\"small\""
        );
        assert_eq!(
            serde_json::to_string(&ClusterSize::Medium).unwrap(),
            "\"medium\""
        );
        assert_eq!(
            serde_json::to_string(&ClusterSize::Large).unwrap(),
            "\"large\""
        );
    }
}
