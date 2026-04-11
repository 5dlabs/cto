//! CTO Config generation logic.
//!
//! Provides functions to generate `CtoConfig` for projects.

use std::collections::{HashMap, HashSet};

use crate::agents::{capitalize, get_agent_config, workflow_agents};
use crate::tools::{analyze_agent_tasks_for_tools, analyze_all_tasks_for_tools, ToolAnalyzable};
use crate::types::{
    AcpDefaults, CtoConfig, Defaults, IntakeDefaults, IntakeModels, LinearDefaults,
    LinearIntakeSettings, MultiModelConfig, PlayDefaults, CTO_CONFIG_VERSION,
};

/// Input for generating a project CTO config.
#[derive(Debug, Clone, Default)]
pub struct ProjectConfigInput {
    /// Repository URL or org/repo format.
    pub repository_url: Option<String>,
    /// Project name for service derivation.
    pub project_name: Option<String>,
    /// Linear team ID.
    pub team_id: String,
    /// Source branch for PRs (defaults to "main").
    pub source_branch: Option<String>,
    /// Docs repository (defaults to same as repository).
    pub docs_repository: Option<String>,
    /// Docs project directory.
    pub docs_project_directory: Option<String>,
}

impl ProjectConfigInput {
    /// Extract repository in org/repo format from URL.
    ///
    /// Returns empty string if no repository URL is provided.
    /// The intake workflow will create a new repository in this case.
    #[must_use]
    pub fn repository(&self) -> String {
        self.repository_url
            .as_ref()
            .and_then(|url| {
                url.strip_prefix("https://github.com/")
                    .or_else(|| url.strip_prefix("git@github.com:"))
                    .map(|s| s.trim_end_matches(".git").to_string())
            })
            .unwrap_or_default()
    }

    /// Derive service name from project name.
    #[must_use]
    pub fn service(&self) -> String {
        let name = self.project_name.as_deref().unwrap_or("unnamed-service");
        derive_service_name(name)
    }
}

/// Derive a service name from a project name.
///
/// Converts to lowercase, replaces spaces with hyphens, and removes special characters.
#[must_use]
pub fn derive_service_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate a CTO config for a project without task analysis.
///
/// This creates a config with all standard agents and project-specific defaults.
#[must_use]
pub fn generate_project_config(input: &ProjectConfigInput) -> CtoConfig {
    let repository = input.repository();
    let service = input.service();

    let mut config = CtoConfig {
        version: CTO_CONFIG_VERSION.to_string(),
        org_name: "5DLabs".to_string(),
        defaults: Defaults {
            intake: IntakeDefaults {
                github_app: "5DLabs-Morgan".to_string(),
                cli: "claude".to_string(),
                include_codebase: false,
                source_branch: input
                    .source_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string()),
                models: IntakeModels {
                    primary: "claude-opus-4-5-20251101".to_string(),
                    research: "claude-opus-4-5-20251101".to_string(),
                    fallback: "claude-opus-4-5-20251101".to_string(),
                    cli_models: HashMap::new(),
                },
                auto_append_deploy_task: false,
                multi_model: MultiModelConfig::default(),
            },
            linear: LinearDefaults {
                team_id: input.team_id.clone(),
                pm_server_url: "https://pm.5dlabs.ai".to_string(),
                intake: LinearIntakeSettings {
                    create_project: true,
                    project_template: "Play Workflow".to_string(),
                },
            },
            acp: AcpDefaults::default(),
            play: PlayDefaults {
                // Agent names are derived from orgName, no need to set them explicitly
                repository: repository.clone(),
                service,
                docs_repository: input
                    .docs_repository
                    .clone()
                    .unwrap_or_else(|| repository.clone()),
                docs_project_directory: input
                    .docs_project_directory
                    .clone()
                    .unwrap_or_else(|| "docs".to_string()),
                ..PlayDefaults::default()
            },
            skills_repo: None,
            skills_project: None,
        },
        agents: HashMap::new(),
    };

    // Add all standard agents
    let standard_agents = [
        "morgan", "rex", "blaze", "cleo", "tess", "cipher", "atlas", "bolt",
    ];
    for agent_name in standard_agents {
        let agent_config = get_agent_config(agent_name);
        config.agents.insert(agent_name.to_string(), agent_config);
    }

    config
}

/// Generate a CTO config with task-based tool analysis.
///
/// Analyzes the provided tasks to determine which agents are needed
/// and what tools each agent should have access to.
#[must_use]
pub fn generate_config_with_tasks<T: ToolAnalyzable>(
    input: &ProjectConfigInput,
    tasks: &[T],
) -> CtoConfig {
    let repository = input.repository();
    let service = input.service();

    // Collect unique agents needed from tasks
    let mut needed_agents: HashSet<String> = HashSet::new();

    // Always include workflow agents
    for agent in workflow_agents() {
        needed_agents.insert(agent.to_string());
    }

    // Add agents based on task hints
    for task in tasks {
        if let Some(hint) = task.agent_hint() {
            needed_agents.insert(hint.to_lowercase());
        }
    }

    // Analyze all tasks for global technology requirements
    let global_tech_tools = analyze_all_tasks_for_tools(tasks);

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

        // Support agents get global tech tools for context
        if ["cleo", "cipher", "tess"].contains(&agent_name.as_str()) {
            for tool in &global_tech_tools {
                if !agent_config.tools.remote.contains(tool) {
                    agent_config.tools.remote.push(tool.clone());
                }
            }
        }

        // Sort and dedupe tools for consistent output
        agent_config.tools.remote.sort();
        agent_config.tools.remote.dedup();

        agents.insert(agent_name.clone(), agent_config);
    }

    // Determine primary agents
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

    CtoConfig {
        version: CTO_CONFIG_VERSION.to_string(),
        org_name: "5DLabs".to_string(),
        defaults: Defaults {
            intake: IntakeDefaults {
                github_app: "5DLabs-Morgan".to_string(),
                cli: "claude".to_string(),
                include_codebase: false,
                source_branch: input
                    .source_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string()),
                models: IntakeModels {
                    primary: "claude-opus-4-5-20251101".to_string(),
                    research: "claude-opus-4-5-20251101".to_string(),
                    fallback: "claude-opus-4-5-20251101".to_string(),
                    cli_models: HashMap::new(),
                },
                auto_append_deploy_task: false,
                multi_model: MultiModelConfig::default(),
            },
            linear: LinearDefaults {
                team_id: input.team_id.clone(),
                pm_server_url: "https://pm.5dlabs.ai".to_string(),
                intake: LinearIntakeSettings {
                    create_project: true,
                    project_template: "Play Workflow".to_string(),
                },
            },
            acp: AcpDefaults::default(),
            play: PlayDefaults {
                // Override implementation/frontend agents based on task analysis
                implementation_agent: Some(format!("5DLabs-{}", capitalize(primary_impl))),
                frontend_agent: Some(format!("5DLabs-{}", capitalize(primary_frontend))),
                // Other agents use defaults derived from orgName
                repository: repository.clone(),
                service,
                docs_repository: input
                    .docs_repository
                    .clone()
                    .unwrap_or_else(|| repository.clone()),
                docs_project_directory: input
                    .docs_project_directory
                    .clone()
                    .unwrap_or_else(|| "docs".to_string()),
                ..PlayDefaults::default()
            },
            skills_repo: None,
            skills_project: None,
        },
        agents,
    }
}

/// Generate a CTO config JSON string for a project.
///
/// This is a convenience function that generates a config and serializes it.
#[must_use]
pub fn generate_project_config_json(input: &ProjectConfigInput) -> String {
    let config = generate_project_config(input);
    config.to_json().unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTask {
        title: String,
        description: String,
        details: String,
        agent: Option<String>,
    }

    impl ToolAnalyzable for TestTask {
        fn title(&self) -> &str {
            &self.title
        }
        fn description(&self) -> &str {
            &self.description
        }
        fn details(&self) -> &str {
            &self.details
        }
        fn agent_hint(&self) -> Option<&str> {
            self.agent.as_deref()
        }
    }

    #[test]
    fn test_derive_service_name() {
        assert_eq!(derive_service_name("My Project"), "my-project");
        assert_eq!(derive_service_name("my-project"), "my-project");
        assert_eq!(derive_service_name("My Cool Project!"), "my-cool-project");
        assert_eq!(derive_service_name("  spaces  "), "spaces");
    }

    #[test]
    fn test_project_config_input_repository() {
        let input = ProjectConfigInput {
            repository_url: Some("https://github.com/myorg/myrepo".to_string()),
            ..Default::default()
        };
        assert_eq!(input.repository(), "myorg/myrepo");

        let input_git = ProjectConfigInput {
            repository_url: Some("https://github.com/myorg/myrepo.git".to_string()),
            ..Default::default()
        };
        assert_eq!(input_git.repository(), "myorg/myrepo");

        let input_ssh = ProjectConfigInput {
            repository_url: Some("git@github.com:myorg/myrepo.git".to_string()),
            ..Default::default()
        };
        assert_eq!(input_ssh.repository(), "myorg/myrepo");
    }

    #[test]
    fn test_generate_project_config() {
        let input = ProjectConfigInput {
            repository_url: Some("https://github.com/5dlabs/test".to_string()),
            project_name: Some("Test Project".to_string()),
            team_id: "team-123".to_string(),
            ..Default::default()
        };

        let config = generate_project_config(&input);

        assert_eq!(config.version, CTO_CONFIG_VERSION);
        assert_eq!(config.defaults.play.repository, "5dlabs/test");
        assert_eq!(config.defaults.play.service, "test-project");
        assert_eq!(config.defaults.linear.team_id, "team-123");
        assert!(config.agents.contains_key("rex"));
        assert!(config.agents.contains_key("blaze"));
    }

    #[test]
    fn test_generate_config_with_tasks() {
        let input = ProjectConfigInput {
            repository_url: Some("https://github.com/5dlabs/test".to_string()),
            project_name: Some("Test".to_string()),
            team_id: "team-123".to_string(),
            ..Default::default()
        };

        let tasks = vec![
            TestTask {
                title: "Setup Database".to_string(),
                description: "Configure PostgreSQL".to_string(),
                details: String::new(),
                agent: Some("rex".to_string()),
            },
            TestTask {
                title: "Build UI".to_string(),
                description: "Use shadcn components".to_string(),
                details: String::new(),
                agent: Some("blaze".to_string()),
            },
        ];

        let config = generate_config_with_tasks(&input, &tasks);

        // Rex should have postgres tools
        let rex = config.agents.get("rex").unwrap();
        assert!(rex.tools.remote.contains(&"postgres_query".to_string()));

        // Blaze should have shadcn tools
        let blaze = config.agents.get("blaze").unwrap();
        assert!(blaze
            .tools
            .remote
            .contains(&"shadcn_list_components".to_string()));

        // Support agents should have global tech tools
        let cleo = config.agents.get("cleo").unwrap();
        assert!(cleo.tools.remote.contains(&"postgres_query".to_string()));
    }

    #[test]
    fn test_generate_project_config_json() {
        let input = ProjectConfigInput {
            repository_url: Some("https://github.com/5dlabs/test".to_string()),
            project_name: Some("Test".to_string()),
            team_id: "team-123".to_string(),
            ..Default::default()
        };

        let json = generate_project_config_json(&input);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["agents"]["rex"].is_object());
    }
}
