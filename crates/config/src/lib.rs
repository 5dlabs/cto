//! CTO Configuration Library
//!
//! Shared configuration types and generation for CTO Play workflows.
//!
//! This crate provides:
//! - Configuration types (`CtoConfig`, `AgentConfig`, etc.)
//! - Agent definitions with default tools
//! - Tool mappings for task-based analysis
//! - Config generation functions
//!
//! # Example
//!
//! ```rust
//! use config::{generate_project_config, ProjectConfigInput};
//!
//! let input = ProjectConfigInput {
//!     repository_url: Some("https://github.com/myorg/myrepo".to_string()),
//!     project_name: Some("My Project".to_string()),
//!     team_id: "team-123".to_string(),
//!     ..Default::default()
//! };
//!
//! let config = generate_project_config(&input);
//! let json = config.to_json().unwrap();
//! ```

pub mod agents;
pub mod generator;
pub mod tools;
pub mod types;

// Re-export main types for convenience
pub use agents::{
    all_agent_names, capitalize, default_remote_tools, get_agent_config, workflow_agents,
    DEFAULT_CLI, DEFAULT_MODEL,
};
pub use generator::{
    derive_service_name, generate_config_with_tasks, generate_project_config,
    generate_project_config_json, ProjectConfigInput,
};
pub use tools::{
    analyze_agent_tasks_for_tools, analyze_all_tasks_for_tools, analyze_content_for_tools,
    analyze_task_for_tools, ToolAnalyzable, TECH_TOOL_MAPPINGS,
};
pub use types::{
    AgentConfig, AgentTools, CtoConfig, Defaults, IntakeDefaults, IntakeModels, LinearDefaults,
    LinearIntakeSettings, PlayDefaults, SubagentConfig, CTO_CONFIG_VERSION,
};
