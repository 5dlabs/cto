//! Task Controller Configuration
//!
//! Simplified configuration structure for the new DocsRun/CodeRun controller.
//! Contains only the essential configuration needed for our current implementation.

use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::Api, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main controller configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControllerConfig {
    /// Job configuration
    pub job: JobConfig,

    /// Agent configuration
    pub agent: AgentConfig,

    /// Individual agent configurations (GitHub apps, tools, etc.)
    #[serde(default)]
    pub agents: HashMap<String, AgentDefinition>,

    /// Secrets configuration
    pub secrets: SecretsConfig,

    /// Tool permissions configuration
    pub permissions: PermissionsConfig,

    /// Telemetry configuration
    pub telemetry: TelemetryConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Cleanup configuration
    #[serde(default)]
    pub cleanup: CleanupConfig,
}

/// Job configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JobConfig {
    /// Job timeout in seconds
    #[serde(rename = "activeDeadlineSeconds")]
    pub active_deadline_seconds: i64,
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    /// Default container image configuration (for backward compatibility)
    pub image: ImageConfig,

    /// CLI-specific image configurations
    #[serde(default)]
    pub cli_images: HashMap<String, ImageConfig>,

    /// Agent-specific CLI configurations (maps GitHub app names to default CLI configs)
    #[serde(default, rename = "agentCliConfigs")]
    pub agent_cli_configs: HashMap<String, crate::crds::CLIConfig>,

    /// Image pull secrets for private registries
    #[serde(default, rename = "imagePullSecrets")]
    pub image_pull_secrets: Vec<String>,

    /// Optional sidecar configuration
    #[serde(default, rename = "inputBridge")]
    pub input_bridge: InputBridgeConfig,

    /// Optional default ServiceAccount name to use for CodeRun jobs
    #[serde(default, rename = "serviceAccountName")]
    pub service_account_name: Option<String>,
}

/// Image configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ImageConfig {
    /// Image repository (e.g., "ghcr.io/5dlabs/claude")
    pub repository: String,

    /// Image tag (e.g., "latest", "v2.1.0")
    pub tag: String,
}

/// Sidecar (auxiliary tools) configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InputBridgeConfig {
    /// Whether the sidecar is enabled
    #[serde(default = "default_input_bridge_enabled")]
    pub enabled: bool,

    /// Sidecar image configuration
    pub image: ImageConfig,

    /// HTTP port for the sidecar
    #[serde(default = "default_input_bridge_port")]
    pub port: u16,
}

fn default_input_bridge_enabled() -> bool {
    true
}
fn default_input_bridge_port() -> u16 {
    8080
}

/// Secrets configuration - only what we actually use
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretsConfig {
    /// Anthropic API key secret name (for rotation)
    #[serde(rename = "apiKeySecretName")]
    pub api_key_secret_name: String,

    /// Anthropic API key secret key
    #[serde(rename = "apiKeySecretKey")]
    pub api_key_secret_key: String,
}

/// Tool permissions configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionsConfig {
    /// Whether to override default tool permissions
    #[serde(rename = "agentToolsOverride")]
    pub agent_tools_override: bool,

    /// Allowed tool patterns
    pub allow: Vec<String>,

    /// Denied tool patterns
    pub deny: Vec<String>,
}

/// Telemetry configuration (used in templates)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,

    /// OTLP endpoint URL
    #[serde(rename = "otlpEndpoint")]
    pub otlp_endpoint: String,

    /// OTLP protocol (grpc/http)
    #[serde(rename = "otlpProtocol")]
    pub otlp_protocol: String,

    /// Logs endpoint (for code tasks)
    #[serde(rename = "logsEndpoint")]
    pub logs_endpoint: String,

    /// Logs protocol (for code tasks)
    #[serde(rename = "logsProtocol")]
    pub logs_protocol: String,
}

/// Storage configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Storage class name for PVCs (e.g., "local-path" for local development)
    #[serde(rename = "storageClassName")]
    pub storage_class_name: Option<String>,

    /// Storage size for workspace PVCs
    #[serde(rename = "workspaceSize", default = "default_workspace_size")]
    pub workspace_size: String,
}

fn default_workspace_size() -> String {
    "10Gi".to_string()
}

/// Cleanup configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    /// Whether automatic cleanup is enabled
    #[serde(default = "default_cleanup_enabled")]
    pub enabled: bool,

    /// Minutes to wait before cleaning up completed (successful) jobs
    #[serde(
        rename = "completedJobDelayMinutes",
        default = "default_completed_delay"
    )]
    pub completed_job_delay_minutes: u64,

    /// Minutes to wait before cleaning up failed jobs
    #[serde(rename = "failedJobDelayMinutes", default = "default_failed_delay")]
    pub failed_job_delay_minutes: u64,

    /// Whether to delete the ConfigMap when cleaning up the job
    #[serde(rename = "deleteConfigMap", default = "default_delete_configmap")]
    pub delete_configmap: bool,
}

fn default_cleanup_enabled() -> bool {
    true
}

fn default_completed_delay() -> u64 {
    5 // 5 minutes
}

fn default_failed_delay() -> u64 {
    60 // 60 minutes (1 hour)
}

fn default_delete_configmap() -> bool {
    true
}

/// Individual agent definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentDefinition {
    /// GitHub app name for this agent
    #[serde(rename = "githubApp")]
    pub github_app: String,

    /// Tool configuration for this agent
    #[serde(default)]
    pub tools: Option<AgentTools>,
}

/// Tool configuration for an agent
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentTools {
    /// Remote tools available to this agent
    #[serde(default)]
    pub remote: Vec<String>,

    /// Local server configurations
    #[serde(default, rename = "localServers")]
    pub local_servers: Option<LocalServerConfigs>,
}

/// Local server configurations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalServerConfigs {
    /// Filesystem server configuration
    pub filesystem: LocalServerConfig,

    /// Git server configuration
    pub git: LocalServerConfig,
}

/// Configuration for a local MCP server
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalServerConfig {
    /// Whether this server is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Tools available from this server
    #[serde(default)]
    pub tools: Vec<String>,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        CleanupConfig {
            enabled: default_cleanup_enabled(),
            completed_job_delay_minutes: default_completed_delay(),
            failed_job_delay_minutes: default_failed_delay(),
            delete_configmap: default_delete_configmap(),
        }
    }
}

impl ControllerConfig {
    /// Validate that configuration has required fields
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.agent.image.repository == "MISSING_IMAGE_CONFIG"
            || self.agent.image.tag == "MISSING_IMAGE_CONFIG"
        {
            return Err(anyhow::anyhow!(
                "Agent image configuration is missing! This indicates the controller ConfigMap was not loaded properly. \
                Please ensure the 'agent.image.repository' and 'agent.image.tag' are set in the Helm values."
            ));
        }

        // If input bridge is enabled, ensure its image is configured
        if self.agent.input_bridge.enabled
            && (self.agent.input_bridge.image.repository.trim().is_empty()
                || self.agent.input_bridge.image.tag.trim().is_empty()
                || self.agent.input_bridge.image.repository == "MISSING_IMAGE_CONFIG"
                || self.agent.input_bridge.image.tag == "MISSING_IMAGE_CONFIG")
        {
            return Err(anyhow::anyhow!(
                "Input bridge is enabled but image is not configured. Please set 'agent.inputBridge.image.repository' and 'agent.inputBridge.image.tag' in Helm values."
            ));
        }
        Ok(())
    }

    /// Load configuration from mounted ConfigMap file
    pub fn from_mounted_file(config_path: &str) -> Result<Self, anyhow::Error> {
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;

        let config: ControllerConfig = serde_yaml::from_str(&config_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse config YAML: {}", e))?;

        Ok(config)
    }

    /// Load configuration from a `ConfigMap` (legacy API-based method)
    pub async fn from_configmap(
        client: &Client,
        namespace: &str,
        name: &str,
    ) -> Result<Self, anyhow::Error> {
        let api: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
        let cm = api.get(name).await?;

        let data = cm
            .data
            .ok_or_else(|| anyhow::anyhow!("ConfigMap has no data"))?;
        let config_str = data
            .get("config.yaml")
            .ok_or_else(|| anyhow::anyhow!("ConfigMap missing config.yaml"))?;

        let config: ControllerConfig = serde_yaml::from_str(config_str)?;
        Ok(config)
    }
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            job: JobConfig {
                active_deadline_seconds: 7200, // 2 hours
            },
            agent: AgentConfig {
                image: ImageConfig {
                    repository: "MISSING_IMAGE_CONFIG".to_string(),
                    tag: "MISSING_IMAGE_CONFIG".to_string(),
                },
                cli_images: HashMap::new(),
                agent_cli_configs: HashMap::new(),
                image_pull_secrets: vec!["ghcr-secret".to_string()],
                input_bridge: InputBridgeConfig {
                    enabled: true,
                    image: ImageConfig {
                        repository: "ghcr.io/5dlabs/input-bridge".to_string(),
                        tag: "latest".to_string(),
                    },
                    port: 8080,
                },
                service_account_name: None,
            },
            agents: HashMap::new(),
            secrets: SecretsConfig {
                api_key_secret_name: "orchestrator-secrets".to_string(),
                api_key_secret_key: "ANTHROPIC_API_KEY".to_string(),
            },
            permissions: PermissionsConfig {
                agent_tools_override: false,
                allow: vec![
                    "Bash(*)".to_string(),
                    "Edit(*)".to_string(),
                    "Read(*)".to_string(),
                    "Write(*)".to_string(),
                    "MultiEdit(*)".to_string(),
                    "Glob(*)".to_string(),
                    "Grep(*)".to_string(),
                    "LS(*)".to_string(),
                ],
                deny: vec![
                    "Bash(npm:install*, yarn:install*, cargo:install*, docker:*, kubectl:*, rm:-rf*, git:*)".to_string(),
                ],
            },
            // Telemetry configuration with environment variable overrides:
            // - OTLP_ENDPOINT: OTLP traces endpoint (default: http://localhost:4317)
            // - LOGS_ENDPOINT: Logs endpoint (default: http://localhost:4318)
            // - LOGS_PROTOCOL: Logs protocol (default: http)
            telemetry: TelemetryConfig {
                enabled: false,
                otlp_endpoint: std::env::var("OTLP_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4317".to_string()),
                otlp_protocol: "grpc".to_string(),
                logs_endpoint: std::env::var("LOGS_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:4318".to_string()),
                logs_protocol: std::env::var("LOGS_PROTOCOL")
                    .unwrap_or_else(|_| "http".to_string()),
            },
            storage: StorageConfig {
                storage_class_name: None, // Let K8s use default storage class
                workspace_size: "10Gi".to_string(),
            },
            cleanup: CleanupConfig {
                enabled: true,
                completed_job_delay_minutes: 5,
                failed_job_delay_minutes: 60,
                delete_configmap: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
job:
  activeDeadlineSeconds: 3600

agent:
  image:
    repository: "test/image"
    tag: "latest"

secrets:
  apiKeySecretName: "test-secret"
  apiKeySecretKey: "key"

permissions:
  agentToolsOverride: true
  allow: ["*"]
  deny: []

telemetry:
  enabled: true
  otlpEndpoint: "localhost:4317"
  otlpProtocol: "grpc"
  logsEndpoint: "localhost:4318"
  logsProtocol: "http"

storage:
  storageClassName: "local-path"
  workspaceSize: "5Gi"

cleanup:
  enabled: true
  completedJobDelayMinutes: 5
  failedJobDelayMinutes: 60
  deleteConfigMap: true
"#;

        let config: ControllerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.job.active_deadline_seconds, 3600);
        assert_eq!(config.agent.image.repository, "test/image");
        assert!(config.telemetry.enabled);
        assert_eq!(config.permissions.allow, vec!["*"]);
        assert!(config.cleanup.enabled);
        assert_eq!(config.cleanup.completed_job_delay_minutes, 5);
        assert_eq!(config.cleanup.failed_job_delay_minutes, 60);
    }

    #[test]
    fn test_default_config() {
        let config = ControllerConfig::default();
        assert_eq!(config.job.active_deadline_seconds, 7200);
        assert_eq!(config.agent.image.repository, "MISSING_IMAGE_CONFIG");
        assert_eq!(config.secrets.api_key_secret_name, "orchestrator-secrets");
        assert!(!config.telemetry.enabled);
        assert!(!config.permissions.agent_tools_override);
    }
}
