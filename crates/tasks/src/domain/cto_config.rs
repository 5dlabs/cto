//! CTO Config Generation - Generate project-specific cto-config.json from tasks.
//!
//! This module generates a `cto-config.json` file during intake that defines:
//! - Which agents are needed for the project (based on task agent hints)
//! - CLI settings per agent
//! - Number of iterations/retries
//! - Repository settings

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};

/// CTO Config version
pub const CTO_CONFIG_VERSION: &str = "1.0";

/// Agent tool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentTools {
    /// Remote tools from platform tools-server
    #[serde(default)]
    pub remote: Vec<String>,

    /// Local MCP servers to spawn per-agent
    #[serde(default, rename = "localServers")]
    pub local_servers: HashMap<String, serde_json::Value>,
}

/// Individual agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// GitHub App name for this agent
    #[serde(rename = "githubApp")]
    pub github_app: String,

    /// CLI to use (claude, codex, gemini, opencode)
    pub cli: String,

    /// AI model to use
    pub model: String,

    /// MCP tools configuration
    #[serde(default)]
    pub tools: AgentTools,

    /// Frontend stack (for Blaze only)
    #[serde(skip_serializing_if = "Option::is_none", rename = "frontendStack")]
    pub frontend_stack: Option<String>,

    /// Feature flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

/// Play workflow defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayDefaults {
    /// Default AI model
    pub model: String,

    /// Default CLI
    pub cli: String,

    /// Implementation agent GitHub App
    #[serde(rename = "implementationAgent")]
    pub implementation_agent: String,

    /// Frontend agent GitHub App
    #[serde(rename = "frontendAgent")]
    pub frontend_agent: String,

    /// Quality agent GitHub App
    #[serde(rename = "qualityAgent")]
    pub quality_agent: String,

    /// Security agent GitHub App
    #[serde(rename = "securityAgent")]
    pub security_agent: String,

    /// Testing agent GitHub App
    #[serde(rename = "testingAgent")]
    pub testing_agent: String,

    /// Target repository
    pub repository: String,

    /// Service name for workspace isolation
    pub service: String,

    /// Docs repository
    #[serde(rename = "docsRepository")]
    pub docs_repository: String,

    /// Docs project directory
    #[serde(rename = "docsProjectDirectory")]
    pub docs_project_directory: String,

    /// Working directory
    #[serde(rename = "workingDirectory")]
    pub working_directory: String,

    /// Maximum retries for all agents
    #[serde(rename = "maxRetries")]
    pub max_retries: u32,

    /// Implementation agent max retries
    #[serde(rename = "implementationMaxRetries")]
    pub implementation_max_retries: u32,

    /// Frontend agent max retries
    #[serde(rename = "frontendMaxRetries")]
    pub frontend_max_retries: u32,

    /// Quality agent max retries
    #[serde(rename = "qualityMaxRetries")]
    pub quality_max_retries: u32,

    /// Security agent max retries
    #[serde(rename = "securityMaxRetries")]
    pub security_max_retries: u32,

    /// Testing agent max retries
    #[serde(rename = "testingMaxRetries")]
    pub testing_max_retries: u32,

    /// Auto-merge after approval
    #[serde(rename = "autoMerge")]
    pub auto_merge: bool,

    /// Enable parallel execution
    #[serde(rename = "parallelExecution")]
    pub parallel_execution: bool,
}

/// Intake execution mode
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IntakeMode {
    /// Direct API calls via `tasks intake` binary (default, faster)
    #[default]
    Api,
    /// Use AI CLI (claude, codex, etc.) for intake
    Cli,
}

/// Intake workflow defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeDefaults {
    /// GitHub App for intake
    #[serde(rename = "githubApp")]
    pub github_app: String,

    /// Execution mode: "api" (direct calls) or "cli" (use AI CLI)
    #[serde(default)]
    pub mode: IntakeMode,

    /// CLI to use (only used when mode = "cli")
    /// Options: claude, codex, gemini, opencode
    pub cli: String,

    /// Container image for intake workflow
    /// - API mode: use "runtime" (smaller, has tasks CLI)
    /// - CLI mode: use "factory" (has all AI CLIs)
    #[serde(default = "default_intake_image")]
    pub image: String,

    /// Primary model configuration
    pub primary: ModelConfig,

    /// Research model configuration
    pub research: ModelConfig,

    /// Fallback model configuration
    pub fallback: ModelConfig,
}

fn default_intake_image() -> String {
    "ghcr.io/5dlabs/runtime:latest".to_string()
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model ID
    pub model: String,

    /// Provider name
    pub provider: String,
}

/// All default configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    /// Intake workflow defaults
    pub intake: IntakeDefaults,

    /// Play workflow defaults
    pub play: PlayDefaults,
}

/// Complete CTO Config structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtoConfig {
    /// Config version
    pub version: String,

    /// Default configurations for workflows
    pub defaults: Defaults,

    /// Agent configurations (only agents needed for this project)
    pub agents: HashMap<String, AgentConfig>,
}

impl Default for CtoConfig {
    fn default() -> Self {
        Self {
            version: CTO_CONFIG_VERSION.to_string(),
            defaults: Defaults {
                intake: IntakeDefaults {
                    github_app: "5DLabs-Morgan".to_string(),
                    mode: IntakeMode::Api, // Default to API mode (faster, uses tasks CLI)
                    cli: "claude".to_string(), // Used when mode = Cli
                    image: default_intake_image(),
                    primary: ModelConfig {
                        model: "claude-opus-4-5-20250929".to_string(),
                        provider: "anthropic".to_string(),
                    },
                    research: ModelConfig {
                        model: "claude-opus-4-5-20250929".to_string(),
                        provider: "anthropic".to_string(),
                    },
                    fallback: ModelConfig {
                        model: "claude-sonnet-4-20250514".to_string(),
                        provider: "anthropic".to_string(),
                    },
                },
                play: PlayDefaults {
                    model: "claude-sonnet-4-20250514".to_string(),
                    cli: "claude".to_string(),
                    implementation_agent: "5DLabs-Rex".to_string(),
                    frontend_agent: "5DLabs-Blaze".to_string(),
                    quality_agent: "5DLabs-Cleo".to_string(),
                    security_agent: "5DLabs-Cipher".to_string(),
                    testing_agent: "5DLabs-Tess".to_string(),
                    repository: String::new(),
                    service: String::new(),
                    docs_repository: String::new(),
                    docs_project_directory: String::new(),
                    working_directory: ".".to_string(),
                    max_retries: 10,
                    implementation_max_retries: 10,
                    frontend_max_retries: 10,
                    quality_max_retries: 5,
                    security_max_retries: 2,
                    testing_max_retries: 5,
                    auto_merge: false,
                    parallel_execution: true,
                },
            },
            agents: HashMap::new(),
        }
    }
}

/// Default remote tools for all agents
fn default_remote_tools() -> Vec<String> {
    vec![
        "context7_resolve_library_id".to_string(),
        "context7_get_library_docs".to_string(),
        "openmemory_openmemory_query".to_string(),
        "openmemory_openmemory_store".to_string(),
        "openmemory_openmemory_list".to_string(),
    ]
}

/// Technology keywords mapped to tools
struct TechToolMapping {
    keywords: &'static [&'static str],
    tools: &'static [&'static str],
}

/// Tool mappings based on technology keywords in task content
const TECH_TOOL_MAPPINGS: &[TechToolMapping] = &[
    // Database tools
    TechToolMapping {
        keywords: &["postgresql", "postgres", "pg_", "psql", "sqlx"],
        tools: &["postgres_query", "postgres_execute"],
    },
    TechToolMapping {
        keywords: &["redis", "valkey", "cache", "session"],
        tools: &["redis_get", "redis_set", "redis_del"],
    },
    TechToolMapping {
        keywords: &["mongodb", "mongo", "document store"],
        tools: &["mongodb_query", "mongodb_aggregate"],
    },
    TechToolMapping {
        keywords: &["elasticsearch", "opensearch", "full-text search"],
        tools: &["elasticsearch_search", "elasticsearch_index"],
    },
    // Storage tools
    TechToolMapping {
        keywords: &["s3", "object storage", "file upload", "seaweedfs", "minio"],
        tools: &["s3_list", "s3_get", "s3_put"],
    },
    // Messaging tools
    TechToolMapping {
        keywords: &["kafka", "event stream", "message queue"],
        tools: &["kafka_produce", "kafka_consume"],
    },
    TechToolMapping {
        keywords: &["rabbitmq", "amqp", "message broker"],
        tools: &["rabbitmq_publish", "rabbitmq_consume"],
    },
    TechToolMapping {
        keywords: &["nats", "jetstream"],
        tools: &["nats_publish", "nats_subscribe"],
    },
    // API tools
    TechToolMapping {
        keywords: &["graphql", "apollo", "schema"],
        tools: &["graphql_query", "graphql_introspect"],
    },
    TechToolMapping {
        keywords: &["websocket", "real-time", "socket.io", "ws://"],
        tools: &["websocket_connect", "websocket_send"],
    },
    TechToolMapping {
        keywords: &["grpc", "protobuf", "proto"],
        tools: &["grpc_call", "grpc_stream"],
    },
    // Infrastructure tools (for Bolt)
    TechToolMapping {
        keywords: &["kubernetes", "k8s", "deployment", "service", "ingress"],
        tools: &[
            "kubernetes_applyResource",
            "kubernetes_listResources",
            "kubernetes_getResource",
            "kubernetes_deleteResource",
        ],
    },
    TechToolMapping {
        keywords: &["helm", "chart"],
        tools: &["helm_install", "helm_upgrade", "helm_list"],
    },
    TechToolMapping {
        keywords: &["cloudnative-pg", "cnpg", "postgresql operator"],
        tools: &["kubernetes_applyResource", "kubernetes_getPodsLogs"],
    },
    TechToolMapping {
        keywords: &["redis operator", "redis cluster"],
        tools: &["kubernetes_applyResource", "kubernetes_getPodsLogs"],
    },
    // Frontend tools (for Blaze)
    TechToolMapping {
        keywords: &["shadcn", "radix", "ui component"],
        tools: &[
            "shadcn_list_components",
            "shadcn_get_component",
            "shadcn_get_component_demo",
            "shadcn_get_component_metadata",
        ],
    },
    TechToolMapping {
        keywords: &["tanstack", "react-query", "react-table"],
        tools: &["context7_get_library_docs"],
    },
    TechToolMapping {
        keywords: &["tailwind", "css", "styling"],
        tools: &["context7_get_library_docs"],
    },
    // Mobile tools (for Tap)
    TechToolMapping {
        keywords: &["expo", "react-native", "mobile"],
        tools: &["xcodebuild_simulator_build", "xcodebuild_run_tests"],
    },
    TechToolMapping {
        keywords: &["ios", "swift", "xcode"],
        tools: &[
            "xcodebuild_simulator_build",
            "xcodebuild_device_build",
            "xcodebuild_run_tests",
        ],
    },
    // Desktop tools (for Spark)
    TechToolMapping {
        keywords: &["electron", "desktop app"],
        tools: &["xcodebuild_macos_build"],
    },
    // Auth tools
    TechToolMapping {
        keywords: &["authentication", "auth", "oauth", "jwt", "better-auth"],
        tools: &["better_auth_generate_schema", "better_auth_add_plugin"],
    },
    // Testing tools
    TechToolMapping {
        keywords: &["playwright", "e2e test", "browser test"],
        tools: &[
            "browser_navigate",
            "browser_click",
            "browser_type",
            "browser_snapshot",
        ],
    },
    TechToolMapping {
        keywords: &["vitest", "jest", "unit test"],
        tools: &["shell_execute"],
    },
    // Search tools
    TechToolMapping {
        keywords: &["web search", "research", "documentation"],
        tools: &["firecrawl_scrape", "firecrawl_search", "brave_search"],
    },
];

/// Analyze task content and return additional tools needed
fn analyze_task_for_tools(task: &Task) -> HashSet<String> {
    let mut tools = HashSet::new();

    // Combine title, description, and details for analysis
    let content = format!(
        "{} {} {}",
        task.title.to_lowercase(),
        task.description.to_lowercase(),
        task.details.to_lowercase()
    );

    // Check each mapping
    for mapping in TECH_TOOL_MAPPINGS {
        let has_keyword = mapping.keywords.iter().any(|kw| content.contains(kw));
        if has_keyword {
            for tool in mapping.tools {
                tools.insert((*tool).to_string());
            }
        }
    }

    tools
}

/// Analyze all tasks for an agent and return tools needed
fn analyze_agent_tasks_for_tools(tasks: &[Task], agent_name: &str) -> HashSet<String> {
    let mut tools = HashSet::new();

    for task in tasks {
        // Use agent_hint to determine which agent this task belongs to
        let task_agent = task.agent_hint.as_ref().map(|s: &String| s.to_lowercase());

        if let Some(agent) = task_agent {
            if agent == agent_name.to_lowercase() {
                tools.extend(analyze_task_for_tools(task));
            }
        }
    }

    tools
}

/// Get agent configuration based on agent hint
fn get_agent_config(agent_hint: &str) -> AgentConfig {
    match agent_hint.to_lowercase().as_str() {
        "morgan" => AgentConfig {
            github_app: "5DLabs-Morgan".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "firecrawl_scrape".to_string(),
                        "firecrawl_search".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "rex" => AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "grizz" => AgentConfig {
            github_app: "5DLabs-Grizz".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "nova" => AgentConfig {
            github_app: "5DLabs-Nova".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "blaze" => AgentConfig {
            github_app: "5DLabs-Blaze".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                        "shadcn_list_components".to_string(),
                        "shadcn_get_component".to_string(),
                        "shadcn_get_component_demo".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: Some("shadcn".to_string()),
            features: None,
        },
        "tap" => AgentConfig {
            github_app: "5DLabs-Tap".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "spark" => AgentConfig {
            github_app: "5DLabs-Spark".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "cleo" => AgentConfig {
            github_app: "5DLabs-Cleo".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_get_pull_request".to_string(),
                        "github_get_pull_request_files".to_string(),
                        "github_create_pull_request_review".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "cipher" => AgentConfig {
            github_app: "5DLabs-Cipher".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_list_code_scanning_alerts".to_string(),
                        "github_get_code_scanning_alert".to_string(),
                        "github_get_pull_request".to_string(),
                        "github_create_pull_request_review".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "tess" => AgentConfig {
            github_app: "5DLabs-Tess".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_get_pull_request".to_string(),
                        "github_create_pull_request_review".to_string(),
                        "kubernetes_listResources".to_string(),
                        "kubernetes_getPodsLogs".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "atlas" => AgentConfig {
            github_app: "5DLabs-Atlas".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_get_pull_request".to_string(),
                        "github_merge_pull_request".to_string(),
                        "github_create_pull_request_review".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        "bolt" => AgentConfig {
            github_app: "5DLabs-Bolt".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "kubernetes_applyResource".to_string(),
                        "kubernetes_listResources".to_string(),
                        "kubernetes_getResource".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
        // Default to Rex for unknown agents
        _ => AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: "claude".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: AgentTools {
                remote: default_remote_tools(),
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
        },
    }
}

/// Generate CTO config from tasks.
///
/// Analyzes the tasks to determine which agents are needed based on their
/// `agent_hint` field and generates a project-specific configuration.
/// Also analyzes task content to determine which tools each agent needs.
pub fn generate_cto_config(
    tasks: &[Task],
    repository: &str,
    service: &str,
    docs_repository: &str,
    docs_project_directory: &str,
) -> CtoConfig {
    // Collect unique agents needed
    let mut needed_agents: HashSet<String> = HashSet::new();

    // Always include bolt (infrastructure) and support agents
    needed_agents.insert("bolt".to_string());
    needed_agents.insert("cleo".to_string());
    needed_agents.insert("cipher".to_string());
    needed_agents.insert("tess".to_string());
    needed_agents.insert("atlas".to_string());

    // Add agents based on agent_hint from tasks
    for task in tasks {
        // Use agent_hint to determine which agent this task needs
        let agent_name = task.agent_hint.as_ref().map(|s: &String| s.to_lowercase());

        if let Some(agent) = agent_name {
            needed_agents.insert(agent);
        }
    }

    // Analyze all tasks for global technology requirements
    // (support agents need to know about tech used across all tasks)
    let mut global_tech_tools: HashSet<String> = HashSet::new();
    for task in tasks {
        global_tech_tools.extend(analyze_task_for_tools(task));
    }

    // Build agent configurations with task-specific tools
    let mut agents = HashMap::new();
    for agent_name in &needed_agents {
        let mut agent_config = get_agent_config(agent_name);

        // Analyze tasks assigned to this agent for additional tools
        let task_tools = analyze_agent_tasks_for_tools(tasks, agent_name);

        // Add task-specific tools to the agent's remote tools
        for tool in task_tools {
            if !agent_config.tools.remote.contains(&tool) {
                agent_config.tools.remote.push(tool);
            }
        }

        // Support agents (cleo, cipher, tess) get global tech tools for context
        if ["cleo", "cipher", "tess"].contains(&agent_name.as_str()) {
            for tool in &global_tech_tools {
                if !agent_config.tools.remote.contains(tool) {
                    agent_config.tools.remote.push(tool.clone());
                }
            }
        }

        // Sort tools for consistent output
        agent_config.tools.remote.sort();
        agent_config.tools.remote.dedup();

        agents.insert(agent_name.clone(), agent_config);
    }

    // Determine primary implementation agent (most common)
    let implementation_agents = ["rex", "grizz", "nova"];
    let frontend_agents = ["blaze", "tap", "spark"];

    let primary_impl = implementation_agents
        .iter()
        .find(|a| needed_agents.contains(**a))
        .unwrap_or(&"rex");

    let primary_frontend = frontend_agents
        .iter()
        .find(|a| needed_agents.contains(**a))
        .unwrap_or(&"blaze");

    let mut config = CtoConfig::default();

    // Set play defaults
    config.defaults.play.repository = repository.to_string();
    config.defaults.play.service = service.to_string();
    config.defaults.play.docs_repository = docs_repository.to_string();
    config.defaults.play.docs_project_directory = docs_project_directory.to_string();
    config.defaults.play.implementation_agent = format!("5DLabs-{}", capitalize(primary_impl));
    config.defaults.play.frontend_agent = format!("5DLabs-{}", capitalize(primary_frontend));

    // Set agents
    config.agents = agents;

    config
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Save CTO config to file.
pub async fn save_cto_config(config: &CtoConfig, output_dir: &Path) -> TasksResult<()> {
    let config_path = output_dir.join("cto-config.json");

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| TasksError::FileWriteError {
            path: parent.display().to_string(),
            reason: e.to_string(),
        })?;
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)
        .await
        .map_err(|e| TasksError::FileWriteError {
            path: config_path.display().to_string(),
            reason: e.to_string(),
        })?;

    tracing::info!("Generated cto-config.json with {} agents", config.agents.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, title: &str, description: &str, agent: &str) -> Task {
        let mut task = Task::new(id, title, description);
        task.agent_hint = Some(agent.to_string());
        task
    }

    fn make_task_with_details(
        id: &str,
        title: &str,
        description: &str,
        details: &str,
        agent: &str,
    ) -> Task {
        let mut task = Task::new(id, title, description);
        task.agent_hint = Some(agent.to_string());
        task.details = details.to_string();
        task
    }

    #[test]
    fn test_generate_cto_config_basic() {
        let tasks = vec![
            make_task("1", "Setup PostgreSQL", "Database setup", "bolt"),
            make_task("2", "Build API", "Rust backend", "rex"),
            make_task("3", "Build Dashboard", "React frontend", "blaze"),
        ];

        let config = generate_cto_config(
            &tasks,
            "5dlabs/alerthub",
            "alerthub",
            "5dlabs/alerthub",
            "docs/alerthub",
        );

        assert_eq!(config.version, "1.0");
        assert!(config.agents.contains_key("bolt"));
        assert!(config.agents.contains_key("rex"));
        assert!(config.agents.contains_key("blaze"));
        assert!(config.agents.contains_key("cleo")); // Always included
        assert!(config.agents.contains_key("tess")); // Always included
    }

    #[test]
    fn test_task_analysis_postgres() {
        let task = make_task_with_details(
            "1",
            "Setup Database",
            "Configure PostgreSQL database",
            "Use sqlx for database access",
            "rex",
        );

        let tools = analyze_task_for_tools(&task);
        assert!(tools.contains("postgres_query"));
        assert!(tools.contains("postgres_execute"));
    }

    #[test]
    fn test_task_analysis_redis() {
        let task = make_task(
            "1",
            "Add Caching Layer",
            "Implement Redis caching for sessions",
            "nova",
        );

        let tools = analyze_task_for_tools(&task);
        assert!(tools.contains("redis_get"));
        assert!(tools.contains("redis_set"));
    }

    #[test]
    fn test_task_analysis_shadcn() {
        let task = make_task_with_details(
            "1",
            "Build Dashboard UI",
            "Create admin dashboard with React",
            "Use shadcn/ui components for the interface",
            "blaze",
        );

        let tools = analyze_task_for_tools(&task);
        assert!(tools.contains("shadcn_list_components"));
        assert!(tools.contains("shadcn_get_component"));
    }

    #[test]
    fn test_task_analysis_kubernetes() {
        let task = make_task_with_details(
            "1",
            "Deploy Infrastructure",
            "Set up Kubernetes resources",
            "Create deployment and service manifests",
            "bolt",
        );

        let tools = analyze_task_for_tools(&task);
        assert!(tools.contains("kubernetes_applyResource"));
        assert!(tools.contains("kubernetes_listResources"));
    }

    #[test]
    fn test_generate_config_with_task_tools() {
        let tasks = vec![
            make_task_with_details(
                "1",
                "Setup PostgreSQL Database",
                "Deploy PostgreSQL using CloudNative-PG operator",
                "Create Cluster CR for database provisioning",
                "bolt",
            ),
            make_task_with_details(
                "2",
                "Build Alert API",
                "Create Rust API with PostgreSQL and Redis",
                "Use sqlx for postgres and implement redis caching",
                "rex",
            ),
            make_task_with_details(
                "3",
                "Build Dashboard",
                "Create React dashboard with shadcn/ui",
                "Use shadcn components for UI elements",
                "blaze",
            ),
        ];

        let config = generate_cto_config(
            &tasks,
            "5dlabs/alerthub",
            "alerthub",
            "5dlabs/alerthub",
            "docs/alerthub",
        );

        // Rex should have postgres and redis tools
        let rex = config.agents.get("rex").unwrap();
        assert!(rex.tools.remote.contains(&"postgres_query".to_string()));
        assert!(rex.tools.remote.contains(&"redis_get".to_string()));

        // Blaze should have shadcn tools
        let blaze = config.agents.get("blaze").unwrap();
        assert!(blaze.tools.remote.contains(&"shadcn_list_components".to_string()));

        // Bolt should have kubernetes tools
        let bolt = config.agents.get("bolt").unwrap();
        assert!(bolt.tools.remote.contains(&"kubernetes_applyResource".to_string()));

        // Support agents should have global tech tools for context
        let cleo = config.agents.get("cleo").unwrap();
        assert!(cleo.tools.remote.contains(&"postgres_query".to_string()));
    }

    #[test]
    fn test_agent_hint_used_for_routing() {
        // Test that agent_hint determines which agent gets the task
        let mut task = Task::new("1", "Test Task", "Test description");
        task.agent_hint = Some("rex".to_string());

        let tasks = vec![task];
        let config = generate_cto_config(
            &tasks,
            "5dlabs/test",
            "test",
            "5dlabs/test",
            "docs/test",
        );

        // Rex should be included based on agent_hint
        assert!(config.agents.contains_key("rex"));
    }

    #[test]
    fn test_get_agent_config_rex() {
        let config = get_agent_config("rex");
        assert_eq!(config.github_app, "5DLabs-Rex");
        assert_eq!(config.cli, "claude");
        assert!(config
            .tools
            .remote
            .contains(&"context7_resolve_library_id".to_string()));
    }

    #[test]
    fn test_get_agent_config_blaze() {
        let config = get_agent_config("blaze");
        assert_eq!(config.github_app, "5DLabs-Blaze");
        assert_eq!(config.frontend_stack, Some("shadcn".to_string()));
        assert!(config
            .tools
            .remote
            .contains(&"shadcn_list_components".to_string()));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("rex"), "Rex");
        assert_eq!(capitalize("blaze"), "Blaze");
        assert_eq!(capitalize(""), "");
    }
}
