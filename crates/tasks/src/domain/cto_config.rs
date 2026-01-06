//! CTO Config Generation - Generate project-specific cto-config.json from tasks.
//!
//! This module re-exports types from the shared `cto-config` crate and provides
//! task-specific wrappers for config generation.

use std::path::Path;

use tokio::fs;

use crate::entities::Task;
use crate::errors::{TasksError, TasksResult};

// Re-export all types from the shared crate
pub use cto_config::{
    // Functions
    all_agent_names,
    analyze_agent_tasks_for_tools,
    analyze_all_tasks_for_tools,
    analyze_content_for_tools,
    analyze_task_for_tools,
    capitalize,
    default_remote_tools,
    derive_service_name,
    generate_config_with_tasks,
    generate_project_config,
    generate_project_config_json,
    get_agent_config,
    workflow_agents,
    // Types
    AgentConfig,
    AgentTools,
    CtoConfig,
    Defaults,
    IntakeDefaults,
    IntakeModels,
    LinearDefaults,
    LinearIntakeSettings,
    PlayDefaults,
    ProjectConfigInput,
    // Traits
    ToolAnalyzable,
    // Constants
    CTO_CONFIG_VERSION,
    DEFAULT_CLI,
    DEFAULT_MODEL,
    // Tool mappings
    TECH_TOOL_MAPPINGS,
};

/// Implement `ToolAnalyzable` for Task to enable task-based config generation.
impl ToolAnalyzable for Task {
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
        self.agent_hint.as_deref()
    }
}

/// Generate CTO config from tasks.
///
/// This is a convenience wrapper that creates a `ProjectConfigInput` and calls
/// the shared crate's `generate_config_with_tasks` function.
///
/// # Arguments
/// * `tasks` - Tasks to analyze for agent and tool requirements
/// * `repository` - Repository in org/repo format (e.g., "5dlabs/myapp")
/// * `service` - Service name for workspace isolation
/// * `docs_repository` - Repository for documentation
/// * `docs_project_directory` - Directory within docs repository
#[must_use]
pub fn generate_cto_config(
    tasks: &[Task],
    repository: &str,
    service: &str,
    docs_repository: &str,
    docs_project_directory: &str,
) -> CtoConfig {
    let input = ProjectConfigInput {
        repository_url: Some(format!("https://github.com/{repository}")),
        project_name: Some(service.to_string()),
        team_id: String::new(), // Will be set by caller if needed
        source_branch: None,
        docs_repository: Some(docs_repository.to_string()),
        docs_project_directory: Some(docs_project_directory.to_string()),
    };

    // Use the shared crate's implementation
    let mut config = generate_config_with_tasks(&input, tasks);

    // Override with exact values (the shared crate normalizes repository URL)
    config.defaults.play.repository = repository.to_string();
    config.defaults.play.service = service.to_string();
    config.defaults.play.docs_repository = docs_repository.to_string();
    config.defaults.play.docs_project_directory = docs_project_directory.to_string();

    config
}

/// Save CTO config to file.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub async fn save_cto_config(config: &CtoConfig, output_dir: &Path) -> TasksResult<()> {
    let config_path = output_dir.join("cto-config.json");

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| TasksError::FileWriteError {
                path: parent.display().to_string(),
                reason: e.to_string(),
            })?;
    }

    let content = config.to_json().map_err(|e| TasksError::FileWriteError {
        path: config_path.display().to_string(),
        reason: e.to_string(),
    })?;

    fs::write(&config_path, content)
        .await
        .map_err(|e| TasksError::FileWriteError {
            path: config_path.display().to_string(),
            reason: e.to_string(),
        })?;

    tracing::info!(
        "Generated cto-config.json with {} agents",
        config.agents.len()
    );

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
        assert!(blaze
            .tools
            .remote
            .contains(&"shadcn_list_components".to_string()));

        // Bolt should have kubernetes tools
        let bolt = config.agents.get("bolt").unwrap();
        assert!(bolt
            .tools
            .remote
            .contains(&"kubernetes_applyResource".to_string()));

        // Support agents should have global tech tools for context
        let cleo = config.agents.get("cleo").unwrap();
        assert!(cleo.tools.remote.contains(&"postgres_query".to_string()));
    }

    #[test]
    fn test_agent_hint_used_for_routing() {
        let mut task = Task::new("1", "Test Task", "Test description");
        task.agent_hint = Some("rex".to_string());

        let tasks = vec![task];
        let config = generate_cto_config(&tasks, "5dlabs/test", "test", "5dlabs/test", "docs/test");

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
