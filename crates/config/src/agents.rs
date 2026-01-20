//! Agent definitions for CTO config.
//!
//! Defines the default configuration for each agent in the system.

use std::collections::HashMap;

use crate::types::{AgentConfig, AgentTools};

/// Default model for all agents.
pub const DEFAULT_MODEL: &str = "claude-opus-4-5-20251101";

/// Default CLI for all agents.
pub const DEFAULT_CLI: &str = "claude";

/// Default remote tools available to all agents.
#[must_use]
pub fn default_remote_tools() -> Vec<String> {
    vec![
        "context7_resolve_library_id".to_string(),
        "context7_get_library_docs".to_string(),
        "openmemory_openmemory_query".to_string(),
        "openmemory_openmemory_store".to_string(),
        "openmemory_openmemory_list".to_string(),
    ]
}

/// Get the default configuration for an agent by name.
#[must_use]
#[allow(clippy::too_many_lines)] // Complex function not easily split
pub fn get_agent_config(agent_name: &str) -> AgentConfig {
    match agent_name.to_lowercase().as_str() {
        "morgan" => AgentConfig {
            github_app: "5DLabs-Morgan".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "rex" => AgentConfig {
            github_app: "5DLabs-Rex".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "firecrawl_scrape".to_string(),
                        "firecrawl_search".to_string(),
                        "github_create_pull_request".to_string(),
                        "github_push_files".to_string(),
                        "github_create_branch".to_string(),
                        "github_get_file_contents".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
            subagents: None,
        },
        "grizz" => AgentConfig {
            github_app: "5DLabs-Grizz".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "nova" => AgentConfig {
            github_app: "5DLabs-Nova".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "blaze" => AgentConfig {
            github_app: "5DLabs-Blaze".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
                        "shadcn_get_component_metadata".to_string(),
                        "browser_navigate".to_string(),
                        "browser_snapshot".to_string(),
                        "browser_click".to_string(),
                        "browser_type".to_string(),
                        "browser_take_screenshot".to_string(),
                        "browser_console_messages".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: Some("shadcn".to_string()),
            features: None,
            subagents: None,
        },
        "tap" => AgentConfig {
            github_app: "5DLabs-Tap".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "spark" => AgentConfig {
            github_app: "5DLabs-Spark".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "cleo" => AgentConfig {
            github_app: "5DLabs-Cleo".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "cipher" => AgentConfig {
            github_app: "5DLabs-Cipher".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "github_list_code_scanning_alerts".to_string(),
                        "github_get_code_scanning_alert".to_string(),
                        "github_list_secret_scanning_alerts".to_string(),
                        "github_get_pull_request".to_string(),
                        "github_create_pull_request_review".to_string(),
                    ]);
                    tools
                },
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
            subagents: None,
        },
        "tess" => AgentConfig {
            github_app: "5DLabs-Tess".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "atlas" => AgentConfig {
            github_app: "5DLabs-Atlas".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
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
            subagents: None,
        },
        "bolt" => AgentConfig {
            github_app: "5DLabs-Bolt".to_string(),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
            tools: AgentTools {
                remote: {
                    let mut tools = default_remote_tools();
                    tools.extend([
                        "kubernetes_applyResource".to_string(),
                        "kubernetes_listResources".to_string(),
                        "kubernetes_getResource".to_string(),
                        "kubernetes_deleteResource".to_string(),
                        "kubernetes_getPodsLogs".to_string(),
                        "argocd_list_applications".to_string(),
                        "argocd_get_application".to_string(),
                        "argocd_sync_application".to_string(),
                        "argocd_get_application_resource_tree".to_string(),
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
            subagents: None,
        },
        // Default to Rex-like config for unknown agents
        _ => AgentConfig {
            github_app: format!("5DLabs-{}", capitalize(agent_name)),
            cli: DEFAULT_CLI.to_string(),
            model: DEFAULT_MODEL.to_string(),
            tools: AgentTools {
                remote: default_remote_tools(),
                local_servers: HashMap::new(),
            },
            frontend_stack: None,
            features: None,
            subagents: None,
        },
    }
}

/// Get all standard agent names.
#[must_use]
pub fn all_agent_names() -> Vec<&'static str> {
    vec![
        "morgan", "rex", "grizz", "nova", "blaze", "tap", "spark", "cleo", "cipher", "tess",
        "atlas", "bolt",
    ]
}

/// Get standard workflow agents (always included in configs).
#[must_use]
pub fn workflow_agents() -> Vec<&'static str> {
    vec!["cleo", "cipher", "tess", "atlas", "bolt"]
}

/// Capitalize the first letter of a string.
#[must_use]
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_agent_config_rex() {
        let config = get_agent_config("rex");
        assert_eq!(config.github_app, "5DLabs-Rex");
        assert_eq!(config.cli, DEFAULT_CLI);
        assert!(config
            .tools
            .remote
            .contains(&"github_create_pull_request".to_string()));
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
    fn test_get_agent_config_bolt() {
        let config = get_agent_config("bolt");
        assert_eq!(config.github_app, "5DLabs-Bolt");
        assert!(config
            .tools
            .remote
            .contains(&"kubernetes_applyResource".to_string()));
    }

    #[test]
    fn test_get_agent_config_case_insensitive() {
        let lower = get_agent_config("rex");
        let upper = get_agent_config("REX");
        let mixed = get_agent_config("Rex");
        assert_eq!(lower.github_app, upper.github_app);
        assert_eq!(lower.github_app, mixed.github_app);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("rex"), "Rex");
        assert_eq!(capitalize("blaze"), "Blaze");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_all_agent_names() {
        let names = all_agent_names();
        assert!(names.contains(&"rex"));
        assert!(names.contains(&"blaze"));
        assert!(names.contains(&"bolt"));
        assert_eq!(names.len(), 12);
    }
}
