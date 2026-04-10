//! Tenant Custom Resource Definition
//!
//! Defines the Tenant CRD schema matching the Kubernetes manifest.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tenant resource specification
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "cto.5dlabs.ai",
    version = "v1alpha1",
    kind = "Tenant",
    namespaced = false,
    status = "TenantStatus",
    shortname = "tn",
    printcolumn = r#"{"name":"Tier", "type":"string", "jsonPath":".spec.tier"}"#,
    printcolumn = r#"{"name":"Phase", "type":"string", "jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Namespace", "type":"string", "jsonPath":".status.namespace"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct TenantSpec {
    /// Owner information for the tenant
    pub owner: TenantOwner,

    /// Subscription tier
    #[serde(default = "default_tier")]
    pub tier: TenantTier,

    /// GitHub integration configuration
    #[serde(default)]
    pub github: Option<GitHubConfig>,

    /// GitLab integration configuration (dual SCM support)
    #[serde(default)]
    pub gitlab: Option<GitLabConfig>,

    /// Active SCM provider for this tenant
    #[serde(default)]
    pub scm_provider: Option<ScmProviderType>,

    /// AI provider configuration
    #[serde(default)]
    pub ai_provider: Option<AiProviderConfig>,

    /// Agent configuration
    #[serde(default)]
    pub agents: Option<AgentConfig>,

    /// Infrastructure configuration
    #[serde(default)]
    pub infrastructure: Option<InfrastructureConfig>,

    /// Resource quotas for the tenant
    #[serde(default)]
    pub resource_quotas: Option<ResourceQuotas>,
}

fn default_tier() -> TenantTier {
    TenantTier::Starter
}

/// Owner information
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TenantOwner {
    /// Email of the tenant owner
    pub email: String,
    /// GitHub user ID
    #[serde(default)]
    pub github_id: Option<String>,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
}

/// Subscription tiers
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TenantTier {
    #[default]
    Starter,
    Pro,
    Team,
    Enterprise,
}

/// GitHub integration configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitHubConfig {
    /// GitHub App installation ID
    #[serde(default)]
    pub installation_id: Option<i64>,
    /// List of authorized repositories
    #[serde(default)]
    pub repos: Vec<GitHubRepo>,
}

/// GitHub repository reference
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepo {
    pub owner: String,
    pub name: String,
    #[serde(default)]
    pub full_name: Option<String>,
}

/// SCM provider selection for the tenant
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScmProviderType {
    #[default]
    GitHub,
    GitLab,
}

/// GitLab integration configuration (dual SCM support)
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitLabConfig {
    /// Self-hosted GitLab hostname (e.g., "git.5dlabs.ai")
    #[serde(default = "default_gitlab_host")]
    pub host: String,
    /// GitLab group (namespace) for projects
    pub group: String,
    /// Reference to secret containing the GitLab PAT
    #[serde(default)]
    pub token_secret: Option<String>,
    /// List of authorized GitLab projects
    #[serde(default)]
    pub projects: Vec<GitLabProject>,
}

fn default_gitlab_host() -> String {
    "git.5dlabs.ai".to_string()
}

/// GitLab project reference
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GitLabProject {
    pub group: String,
    pub name: String,
    #[serde(default)]
    pub path_with_namespace: Option<String>,
}

/// AI provider configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    /// Primary AI provider type
    #[serde(default = "default_ai_provider")]
    pub provider_type: AiProviderType,
    /// Reference to secret in `OpenBao`
    #[serde(default)]
    pub secret_ref: Option<String>,
    /// Additional provider configurations
    #[serde(default)]
    pub additional_providers: Vec<AdditionalProvider>,
}

fn default_ai_provider() -> AiProviderType {
    AiProviderType::Anthropic
}

/// Supported AI provider types
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderType {
    #[default]
    Anthropic,
    OpenAi,
    Google,
}

/// Additional AI provider
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalProvider {
    pub provider_type: AiProviderType,
    pub secret_ref: String,
}

/// Agent configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// List of enabled agents
    #[serde(default)]
    pub enabled: Vec<AgentType>,
    /// Default CLI for agents
    #[serde(default = "default_cli")]
    pub default_cli: CliType,
}

fn default_cli() -> CliType {
    CliType::Claude
}

/// Available agent types
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Morgan,
    Rex,
    Blaze,
    Nova,
    Grizz,
    Tap,
    Spark,
    Vex,
    Cleo,
    Cipher,
    Tess,
    Atlas,
    Bolt,
    Stitch,
}

/// Available CLI types
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CliType {
    #[default]
    Claude,
    Cursor,
    Codex,
    Copilot,
    Factory,
    Gemini,
    Kimi,
    OpenCode,
}

/// Infrastructure configuration
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InfrastructureConfig {
    /// Infrastructure mode
    #[serde(default = "default_infra_mode")]
    pub mode: InfraMode,
    /// Bare metal provider (for managed mode)
    #[serde(default)]
    pub provider: Option<BareMetalProvider>,
    /// Deployment region
    #[serde(default)]
    pub region: Option<String>,
    /// Reference to existing cluster (for BYOC mode)
    #[serde(default)]
    pub cluster_ref: Option<String>,
}

fn default_infra_mode() -> InfraMode {
    InfraMode::CtoCloud
}

/// Infrastructure modes
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InfraMode {
    #[default]
    CtoCloud,
    Managed,
    Byoc,
}

/// Bare metal providers
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BareMetalProvider {
    Latitude,
    Hetzner,
    Ovh,
    Vultr,
    Scaleway,
    Cherry,
    DigitalOcean,
    Onprem,
}

/// Resource quotas
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceQuotas {
    /// Maximum concurrent agent runs
    #[serde(default = "default_max_agents")]
    pub max_concurrent_agents: i32,
    /// Maximum storage in GB
    #[serde(default = "default_max_storage")]
    pub max_storage_gb: i32,
    /// Maximum projects per month
    #[serde(default = "default_max_projects")]
    pub max_projects_per_month: i32,
}

fn default_max_agents() -> i32 {
    3
}
fn default_max_storage() -> i32 {
    10
}
fn default_max_projects() -> i32 {
    5
}

/// Tenant status
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TenantStatus {
    /// Current phase of the tenant
    #[serde(default)]
    pub phase: TenantPhase,
    /// Kubernetes namespace for this tenant
    #[serde(default)]
    pub namespace: Option<String>,
    /// `ArgoCD` Application name
    #[serde(default)]
    pub argocd_app: Option<String>,
    /// `ExternalSecret` resource name
    #[serde(default)]
    pub external_secret_name: Option<String>,
    /// Status conditions
    #[serde(default)]
    pub conditions: Vec<TenantCondition>,
    /// Last observed generation
    #[serde(default)]
    pub observed_generation: Option<i64>,
}

/// Tenant lifecycle phases
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum TenantPhase {
    #[default]
    Pending,
    Provisioning,
    Ready,
    Error,
    Suspended,
}

/// Status condition
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TenantCondition {
    pub condition_type: String,
    pub status: String,
    #[serde(default)]
    pub last_transition_time: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl Tenant {
    /// Generate the namespace name for this tenant
    #[must_use]
    pub fn namespace_name(&self) -> String {
        format!(
            "tenant-{}",
            self.metadata
                .name
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        )
    }

    /// Generate the `ArgoCD` application name
    #[must_use]
    pub fn argocd_app_name(&self) -> String {
        format!("{}-agents", self.namespace_name())
    }

    /// Generate the external secret name
    #[must_use]
    pub fn external_secret_name(&self) -> String {
        format!("{}-secrets", self.namespace_name())
    }
}
