use serde::{Deserialize, Serialize};

/// Installation profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallProfile {
    /// Minimal profile - Local development
    /// - 8GB RAM requirement
    /// - Core components only (`ArgoCD`, Argo Workflows, Controller)
    /// - No monitoring or databases
    Minimal,

    /// Standard profile - Team development
    /// - 16GB RAM requirement
    /// - Full monitoring stack
    /// - Database operators
    Standard,

    /// Production profile - Enterprise deployment
    /// - 32GB RAM requirement
    /// - High availability
    /// - Enterprise security features
    Production,
}

impl InstallProfile {
    pub fn name(&self) -> &str {
        match self {
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Production => "production",
        }
    }

    pub fn memory_requirement_gb(&self) -> usize {
        match self {
            Self::Minimal => 8,
            Self::Standard => 16,
            Self::Production => 32,
        }
    }

    pub fn cpu_requirement(&self) -> usize {
        match self {
            Self::Minimal => 4,
            Self::Standard => 8,
            Self::Production => 16,
        }
    }
}

/// Cluster type for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    /// Local kind cluster
    Kind,
    /// Remote Kubernetes cluster
    Remote,
}

/// Installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Installation profile
    pub profile: InstallProfile,

    /// Cluster type
    pub cluster_type: ClusterType,

    /// Kubernetes namespace
    pub namespace: String,

    /// GitHub organization
    pub github_org: Option<String>,

    /// GitHub repository
    pub github_repo: Option<String>,

    /// Container registry URL
    pub registry: String,

    /// Registry namespace (for multi-tenant registries)
    pub registry_namespace: Option<String>,

    /// Custom domain
    pub domain: Option<String>,

    /// Install monitoring stack
    pub install_monitoring: bool,

    /// Install database operators
    pub install_databases: bool,

    /// Auto-generate cto-config.json
    pub auto_generate_config: bool,
}

impl InstallConfig {
    pub fn get_registry_prefix(&self) -> String {
        if let Some(ns) = &self.registry_namespace {
            format!("{}/{}", self.registry, ns)
        } else {
            self.registry.clone()
        }
    }
}

/// CTO platform configuration (cto-config.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtoConfig {
    pub version: String,
    pub defaults: CtoDefaults,
    pub agents: std::collections::HashMap<String, AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtoDefaults {
    pub docs: DocsDefaults,
    pub code: CodeDefaults,
    pub play: PlayDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsDefaults {
    pub model: String,
    #[serde(rename = "githubApp")]
    pub github_app: String,
    #[serde(rename = "includeCodebase")]
    pub include_codebase: bool,
    #[serde(rename = "sourceBranch")]
    pub source_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDefaults {
    pub model: String,
    #[serde(rename = "githubApp")]
    pub github_app: String,
    #[serde(rename = "continueSession")]
    pub continue_session: bool,
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,
    #[serde(rename = "overwriteMemory")]
    pub overwrite_memory: bool,
    pub repository: String,
    #[serde(rename = "docsRepository")]
    pub docs_repository: String,
    #[serde(rename = "docsProjectDirectory")]
    pub docs_project_directory: String,
    pub service: String,
    pub cli: String,
    #[serde(rename = "maxRetries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayDefaults {
    pub model: String,
    pub cli: String,
    #[serde(rename = "implementationAgent")]
    pub implementation_agent: String,
    #[serde(rename = "frontendAgent")]
    pub frontend_agent: String,
    #[serde(rename = "qualityAgent")]
    pub quality_agent: String,
    #[serde(rename = "testingAgent")]
    pub testing_agent: String,
    pub repository: String,
    pub service: String,
    #[serde(rename = "docsRepository")]
    pub docs_repository: String,
    #[serde(rename = "docsProjectDirectory")]
    pub docs_project_directory: String,
    #[serde(rename = "maxRetries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "githubApp")]
    pub github_app: String,
    pub cli: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<AgentTools>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTools {
    pub remote: Vec<String>,
    #[serde(rename = "localServers")]
    pub local_servers: std::collections::HashMap<String, LocalServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServer {
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
    pub tools: Vec<String>,
}

impl CtoConfig {
    /// Create a default configuration
    pub fn default_config(install_config: &InstallConfig) -> Self {
        let repo = install_config
            .github_org
            .as_ref()
            .and_then(|org| {
                install_config
                    .github_repo
                    .as_ref()
                    .map(|repo| format!("{org}/{repo}"))
            })
            .unwrap_or_else(|| "your-org/your-repo".to_string());

        Self {
            version: "1.0".to_string(),
            defaults: CtoDefaults {
                docs: DocsDefaults {
                    model: "claude-opus-4-1-20250805".to_string(),
                    github_app: "Your-Morgan".to_string(),
                    include_codebase: false,
                    source_branch: "main".to_string(),
                },
                code: CodeDefaults {
                    model: "gpt-4o".to_string(),
                    github_app: "Your-Rex".to_string(),
                    continue_session: false,
                    working_directory: ".".to_string(),
                    overwrite_memory: false,
                    repository: repo.clone(),
                    docs_repository: repo.clone(),
                    docs_project_directory: "docs".to_string(),
                    service: "cto".to_string(),
                    cli: "codex".to_string(),
                    max_retries: 10,
                },
                play: PlayDefaults {
                    model: "claude-sonnet-4-5-20250929".to_string(),
                    cli: "factory".to_string(),
                    implementation_agent: "Your-Rex".to_string(),
                    frontend_agent: "Your-Blaze".to_string(),
                    quality_agent: "Your-Cleo".to_string(),
                    testing_agent: "Your-Tess".to_string(),
                    repository: repo,
                    service: "platform".to_string(),
                    docs_repository: "your-org/docs".to_string(),
                    docs_project_directory: "docs".to_string(),
                    max_retries: 10,
                },
            },
            agents: std::collections::HashMap::new(),
        }
    }
}
