use crate::cli::types::CLIType;
use crate::crds::CodeRun;
use crate::tasks::code::agent::AgentClassifier;
use crate::tasks::config::ControllerConfig;
use crate::tasks::template_paths::{
    CODE_CLAUDE_CONTAINER_TEMPLATE, CODE_CLAUDE_MEMORY_TEMPLATE, CODE_CLAUDE_SETTINGS_TEMPLATE,
    CODE_CODEX_CONFIG_TEMPLATE, CODE_CODEX_CONTAINER_BASE_TEMPLATE,
    CODE_CODING_GUIDELINES_TEMPLATE, CODE_CURSOR_CONTAINER_BASE_TEMPLATE,
    CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE, CODE_CURSOR_PROJECT_CONFIG_TEMPLATE,
    CODE_FACTORY_CONTAINER_BASE_TEMPLATE, CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE,
    CODE_FACTORY_PROJECT_CONFIG_TEMPLATE, CODE_GEMINI_CONTAINER_BASE_TEMPLATE,
    CODE_GEMINI_MEMORY_TEMPLATE, CODE_GITHUB_GUIDELINES_TEMPLATE, CODE_MCP_CONFIG_TEMPLATE,
    CODE_OPENCODE_CONFIG_TEMPLATE, CODE_OPENCODE_CONTAINER_BASE_TEMPLATE,
};
use crate::tasks::tool_catalog::resolve_tool_name;
use crate::tasks::types::Result;
use crate::tasks::workflow::extract_workflow_name;
use handlebars::Handlebars;

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

// Template base path (mounted from ConfigMap)
const AGENT_TEMPLATES_PATH: &str = "/agent-templates";

#[derive(Debug, Clone)]
struct CliRenderSettings {
    model: String,
    temperature: Option<f64>,
    max_output_tokens: Option<u32>,
    approval_policy: String,
    sandbox_mode: String,
    project_doc_max_bytes: u64,
    reasoning_effort: Option<String>,
    auto_level: Option<String>,
    output_format: Option<String>,
    editor_vim_mode: bool,
    toolman_url: String,
    model_provider: Value,
    raw_additional_toml: Option<String>,
    raw_additional_json: Option<String>,
    model_rotation: Vec<String>,
    list_tools_on_start: bool,
}

pub struct CodeTemplateGenerator;

impl CodeTemplateGenerator {
    /// Generate all template files for a code task
    pub fn generate_all_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        match Self::determine_cli_type(code_run) {
            CLIType::Codex => Self::generate_codex_templates(code_run, config),
            CLIType::Cursor => Self::generate_cursor_templates(code_run, config),
            CLIType::Factory => Self::generate_factory_templates(code_run, config),
            CLIType::OpenCode => Self::generate_opencode_templates(code_run, config),
            CLIType::Gemini => Self::generate_gemini_templates(code_run, config),
            _ => Self::generate_claude_templates(code_run, config),
        }
    }

    fn determine_cli_type(code_run: &CodeRun) -> CLIType {
        code_run
            .spec
            .cli_config
            .as_ref()
            .map_or(CLIType::Claude, |cfg| cfg.cli_type)
    }

    fn generate_claude_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Enrich cli_config with agent-level settings (like modelRotation)
        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(code_run, &enriched_cli_config)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(code_run, config, &enriched_cli_config)?,
        );
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );
        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_cursor_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        // Enrich cli_config with agent-level settings (like modelRotation)
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_cursor_container_script(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "AGENTS.md".to_string(),
            Self::generate_cursor_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "cursor-cli-config.json".to_string(),
            Self::generate_cursor_global_config(
                code_run,
                &enriched_cli_config,
                &client_config_value,
                &remote_tools,
            )?,
        );

        templates.insert(
            "cursor-cli.json".to_string(),
            Self::generate_cursor_project_permissions()?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    fn generate_opencode_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        // Enrich cli_config with agent-level settings (like modelRotation)
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_opencode_container_script(
                code_run,
                &enriched_cli_config,
                &remote_tools,
            )?,
        );

        templates.insert(
            "AGENTS.md".to_string(),
            Self::generate_opencode_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "opencode-config.json".to_string(),
            Self::generate_opencode_config(
                code_run,
                &enriched_cli_config,
                &client_config_value,
                &remote_tools,
            )?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    fn generate_gemini_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        // Enrich cli_config with agent-level settings
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_gemini_container_script(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "GEMINI.md".to_string(),
            Self::generate_gemini_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    fn generate_cursor_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let base_template = Self::load_template(CODE_CURSOR_CONTAINER_BASE_TEMPLATE)?;
        handlebars
            .register_partial("cursor_container_base", base_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Cursor container base partial: {e}"
                ))
            })?;

        let template_path = Self::get_cursor_container_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("cursor_container", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Cursor container template {template_path}: {e}"
                ))
            })?;

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);
        let model = render_settings.model.clone();
        let cli_type = Self::determine_cli_type(code_run).to_string();

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "model": model.clone(),
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
        });

        handlebars
            .render("cursor_container", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Cursor container template: {e}"
                ))
            })
    }

    fn generate_cursor_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template_path = Self::get_cursor_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);
        let model = render_settings.model.clone();
        let cli_type = Self::determine_cli_type(code_run).to_string();

        handlebars
            .register_template_string("cursor_agents", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Cursor AGENTS.md template {template_path}: {e}"
                ))
            })?;

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "working_directory": Self::get_working_directory(code_run),
            "workflow_name": workflow_name,
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "toolman": {
                "tools": remote_tools,
            },
        });

        handlebars.render("cursor_agents", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Cursor AGENTS.md template: {e}"
            ))
        })
    }

    fn generate_cursor_global_config(
        code_run: &CodeRun,
        cli_config: &Value,
        _client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE)?;

        handlebars
            .register_template_string("cursor_cli_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Cursor CLI config template: {e}"
                ))
            })?;

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let mut context = json!({
            "model": render_settings.model,
            "temperature": render_settings.temperature,
            "max_output_tokens": render_settings.max_output_tokens,
            "approval_policy": render_settings.approval_policy,
            "sandbox_mode": render_settings.sandbox_mode,
            "project_doc_max_bytes": render_settings.project_doc_max_bytes,
            "editor_vim_mode": render_settings.editor_vim_mode,
            "toolman": {
                "url": render_settings.toolman_url,
                "tools": remote_tools,
            },
        });

        if let Some(raw_json) = &render_settings.raw_additional_json {
            context
                .as_object_mut()
                .expect("context is an object")
                .insert(
                    "raw_additional_json".to_string(),
                    Value::String(raw_json.clone()),
                );
        }

        // Cursor CLI does not currently expose reasoning-effort toggles; ignore any value supplied.

        handlebars
            .render("cursor_cli_config", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Cursor CLI config template: {e}"
                ))
            })
    }

    fn generate_cursor_project_permissions() -> Result<String> {
        Self::load_template(CODE_CURSOR_PROJECT_CONFIG_TEMPLATE)
    }

    fn generate_factory_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        // Enrich cli_config with agent-level settings (like modelRotation)
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_factory_container_script(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "AGENTS.md".to_string(),
            Self::generate_factory_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "factory-cli-config.json".to_string(),
            Self::generate_factory_global_config(
                code_run,
                &enriched_cli_config,
                &client_config_value,
                &remote_tools,
            )?,
        );

        templates.insert(
            "factory-cli.json".to_string(),
            Self::generate_factory_project_permissions()?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    fn generate_factory_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let base_template = Self::load_template(CODE_FACTORY_CONTAINER_BASE_TEMPLATE)?;
        handlebars
            .register_partial("factory_container_base", base_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Factory container base partial: {e}"
                ))
            })?;

        let template_path = Self::get_factory_container_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("factory_container", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Factory container template {template_path}: {e}"
                ))
            })?;

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "model": render_settings.model,
            "auto_level": render_settings.auto_level,
            "output_format": render_settings.output_format,
            "model_rotation": render_settings.model_rotation,
            "list_tools_on_start": render_settings.list_tools_on_start,
            "cli": {
                "type": Self::determine_cli_type(code_run).to_string(),
                "model": render_settings.model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
        });

        handlebars
            .render("factory_container", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Factory container template: {e}"
                ))
            })
    }

    fn generate_factory_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template_path = Self::get_factory_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("factory_agents", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Factory AGENTS.md template {template_path}: {e}"
                ))
            })?;

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);
        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": render_settings.model,
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "working_directory": Self::get_working_directory(code_run),
            "workflow_name": workflow_name,
            "cli_type": Self::determine_cli_type(code_run).to_string(),
            "cli": {
                "type": Self::determine_cli_type(code_run).to_string(),
                "model": render_settings.model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "toolman": {
                "tools": remote_tools,
            },
        });

        handlebars.render("factory_agents", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Factory AGENTS.md template: {e}"
            ))
        })
    }

    fn generate_factory_global_config(
        code_run: &CodeRun,
        cli_config: &Value,
        client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE)?;
        handlebars
            .register_template_string("factory_cli_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Factory CLI config template: {e}"
                ))
            })?;

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let mut context = json!({
            "model": render_settings.model,
            "temperature": render_settings.temperature,
            "max_output_tokens": render_settings.max_output_tokens,
            "approval_policy": render_settings.approval_policy,
            "sandbox_mode": render_settings.sandbox_mode,
            "project_doc_max_bytes": render_settings.project_doc_max_bytes,
            "editor_vim_mode": render_settings.editor_vim_mode,
            "reasoning_effort": render_settings.reasoning_effort,
            "auto_level": render_settings.auto_level,
            "toolman": {
                "url": render_settings.toolman_url,
                "tools": remote_tools,
            },
            "cli_config": cli_config,
            "client_config": client_config,
        });

        if let Some(raw_json) = &render_settings.raw_additional_json {
            context
                .as_object_mut()
                .expect("context is an object")
                .insert(
                    "raw_additional_json".to_string(),
                    Value::String(raw_json.clone()),
                );
        }

        handlebars
            .render("factory_cli_config", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Factory CLI config template: {e}"
                ))
            })
    }

    fn generate_factory_project_permissions() -> Result<String> {
        Self::load_template(CODE_FACTORY_PROJECT_CONFIG_TEMPLATE)
    }

    fn generate_container_script(code_run: &CodeRun, cli_config: &Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Select agent-specific template based on github_app field
        let template_path = Self::get_agent_container_template(code_run);

        // Try to load agent-specific template, fall back to default if not found
        let template = match Self::load_template(&template_path) {
            Ok(content) => content,
            Err(_) if !template_path.ends_with("container.sh.hbs") => {
                // If agent-specific template not found, try default
                debug!(
                    "Agent-specific template {} not found, falling back to default",
                    template_path
                );
                Self::load_template(CODE_CLAUDE_CONTAINER_TEMPLATE)?
            }
            Err(e) => return Err(e),
        };

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "attempts": retry_count + 1,  // Current attempt number (1-indexed)
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
            "cli_config": cli_config,
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    #[allow(clippy::too_many_lines, clippy::items_after_statements)]
    fn generate_claude_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template = Self::load_template(CODE_CLAUDE_MEMORY_TEMPLATE)?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        // Derive allowed env var name lists for inclusion in CLAUDE.md
        // 1) Workflow-provided env names (keys only)
        let workflow_env_vars: Vec<String> = code_run.spec.env.keys().cloned().collect();

        // 2) From requirements.yaml (environment keys and mapped secret key names)
        let mut req_env_vars: Vec<String> = Vec::new();
        let mut req_secret_sources: Vec<String> = Vec::new();

        // Base64 decode helper import for task_requirements parsing
        use base64::{engine::general_purpose, Engine as _};

        if let Some(req_b64) = &code_run.spec.task_requirements {
            if !req_b64.trim().is_empty() {
                if let Ok(decoded) = general_purpose::STANDARD.decode(req_b64) {
                    if let Ok(req_yaml) = serde_yaml::from_slice::<serde_yaml::Value>(&decoded) {
                        if let Some(env_map) =
                            req_yaml.get("environment").and_then(|e| e.as_mapping())
                        {
                            for (k, _v) in env_map {
                                if let Some(key) = k.as_str() {
                                    req_env_vars.push(key.to_string());
                                }
                            }
                        }
                        if let Some(secrets) = req_yaml.get("secrets").and_then(|s| s.as_sequence())
                        {
                            for secret in secrets {
                                if let Some(m) = secret.as_mapping() {
                                    if let Some(name) = m
                                        .get(serde_yaml::Value::from("name"))
                                        .and_then(|n| n.as_str())
                                    {
                                        req_secret_sources.push(name.to_string());
                                    }
                                    // If there are key mappings, surface the env var names (right-hand side)
                                    if let Some(keys) = m
                                        .get(serde_yaml::Value::from("keys"))
                                        .and_then(|k| k.as_sequence())
                                    {
                                        for entry in keys {
                                            if let Some(map) = entry.as_mapping() {
                                                for (_k8s_key, env_name) in map {
                                                    if let Some(n) = env_name.as_str() {
                                                        req_env_vars.push(n.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // De-duplicate and sort for stable output
        use std::collections::BTreeSet;
        let wf_set: BTreeSet<_> = workflow_env_vars.into_iter().collect();
        let req_env_set: BTreeSet<_> = req_env_vars.into_iter().collect();
        let req_src_set: BTreeSet<_> = req_secret_sources.into_iter().collect();
        let workflow_env_vars: Vec<_> = wf_set.into_iter().collect();
        let requirements_env_vars: Vec<_> = req_env_set.into_iter().collect();
        let requirements_secret_sources: Vec<_> = req_src_set.into_iter().collect();

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let cli_type = Self::determine_cli_type(code_run).to_string();

        // Extract model from cli_config like other templates do
        let cli_model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&code_run.spec.model)
            .to_string();

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": cli_model.clone(), // Use cli_model for consistency
            "context_version": code_run.spec.context_version,
            "workflow_env_vars": workflow_env_vars,
            "requirements_env_vars": requirements_env_vars,
            "requirements_secret_sources": requirements_secret_sources,
            "cli_config": cli_config,
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": cli_model, // Use cli_model instead of code_run.spec.model
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "toolman": {
                "tools": remote_tools,
            },
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(
        code_run: &CodeRun,
        config: &ControllerConfig,
        cli_config: &Value,
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_CLAUDE_SETTINGS_TEMPLATE)?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        let context = json!({
            "model": code_run.spec.model,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key,
            "working_directory": code_run.spec.working_directory.as_deref().unwrap_or("."),
            "cli_config": cli_config,
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render settings.json: {e}"))
        })
    }

    fn generate_mcp_config(_code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        // MCP config is currently static, so just load and return the template content
        Self::load_template(CODE_MCP_CONFIG_TEMPLATE)
    }

    fn generate_codex_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let base_template = Self::load_template(CODE_CODEX_CONTAINER_BASE_TEMPLATE)?;
        handlebars
            .register_partial("codex_container_base", base_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Codex container base partial: {e}"
                ))
            })?;

        let template_path = Self::get_codex_container_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("codex_container", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Codex container template {template_path}: {e}"
                ))
            })?;

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_model = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or_else(|| code_run.spec.model.clone(), |cfg| cfg.model.clone());

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "cli": {
                "type": Self::determine_cli_type(code_run).to_string(),
                "model": cli_model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
        });

        handlebars.render("codex_container", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Codex container template: {e}"
            ))
        })
    }

    fn generate_codex_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template_path = Self::get_codex_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("codex_agents", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Codex AGENTS.md template {template_path}: {e}"
                ))
            })?;

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Extract model from cli_config like other templates do
        let model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&code_run.spec.model)
            .to_string();
        let cli_type = Self::determine_cli_type(code_run).to_string();

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "working_directory": Self::get_working_directory(code_run),
            "workflow_name": workflow_name,
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "toolman": {
                "tools": remote_tools,
            },
        });

        handlebars.render("codex_agents", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Codex AGENTS.md template: {e}"
            ))
        })
    }

    /// Enrich `cli_config` with agent-level configuration from `ControllerConfig`
    /// This allows agent-level settings (like modelRotation) to be used as defaults
    fn enrich_cli_config_from_agent(
        cli_config: Value,
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Value {
        let mut enriched = cli_config;

        // Extract agent name from github_app field using AgentClassifier
        let classifier = AgentClassifier::new();
        let agent_name = code_run
            .spec
            .github_app
            .as_deref()
            .and_then(|app| classifier.extract_agent_name(app).ok())
            .unwrap_or_default();

        if let Some(agent_config) = config.agents.get(&agent_name) {
            // If agent has model rotation config, inject it into cli_config
            if let Some(model_rotation) = &agent_config.model_rotation {
                if model_rotation.enabled && !model_rotation.models.is_empty() {
                    // Only inject if not already present in cli_config
                    if enriched.get("modelRotation").is_none() {
                        enriched["modelRotation"] = json!(model_rotation.models);
                    }
                }
            }
        }

        enriched
    }

    #[allow(clippy::too_many_lines)]
    fn build_cli_render_settings(code_run: &CodeRun, cli_config: &Value) -> CliRenderSettings {
        let settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&code_run.spec.model)
            .to_string();

        let max_output_tokens = cli_config
            .get("maxTokens")
            .and_then(Value::as_u64)
            .or_else(|| {
                cli_config
                    .get("modelMaxOutputTokens")
                    .and_then(Value::as_u64)
            })
            .and_then(|v| u32::try_from(v).ok());

        let temperature = cli_config
            .get("temperature")
            .and_then(Value::as_f64)
            .or_else(|| settings.get("temperature").and_then(Value::as_f64));

        let mut approval_policy = settings
            .get("approvalPolicy")
            .and_then(Value::as_str)
            .unwrap_or("never")
            .to_string();

        if !approval_policy.eq_ignore_ascii_case("never") {
            approval_policy = "never".to_string();
        }

        let sandbox_mode = settings
            .get("sandboxPreset")
            .or_else(|| settings.get("sandboxMode"))
            .or_else(|| settings.get("sandbox"))
            .and_then(Value::as_str)
            .unwrap_or("danger-full-access")
            .to_string();

        let project_doc_max_bytes = settings
            .get("projectDocMaxBytes")
            .and_then(Value::as_u64)
            .unwrap_or(32_768);

        let reasoning_effort = cli_config
            .get("reasoningEffort")
            .and_then(Value::as_str)
            .or_else(|| settings.get("reasoningEffort").and_then(Value::as_str))
            .or_else(|| settings.get("modelReasoningEffort").and_then(Value::as_str))
            .map(std::string::ToString::to_string);

        let auto_level = settings
            .get("autoLevel")
            .and_then(Value::as_str)
            .or_else(|| cli_config.get("autoLevel").and_then(Value::as_str))
            .map(std::string::ToString::to_string)
            .or_else(|| reasoning_effort.clone());

        let output_format = settings
            .get("outputFormat")
            .or_else(|| settings.get("output_format"))
            .or_else(|| cli_config.get("outputFormat"))
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string);

        let editor_vim_mode = settings
            .get("editor")
            .and_then(Value::as_object)
            .and_then(|editor| editor.get("vimMode").and_then(Value::as_bool))
            .unwrap_or(false);

        let mut toolman_url = settings
            .get("toolmanUrl")
            .and_then(Value::as_str)
            .map_or_else(
                || {
                    std::env::var("TOOLMAN_SERVER_URL").unwrap_or_else(|_| {
                        "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string()
                    })
                },
                std::string::ToString::to_string,
            );
        toolman_url = toolman_url.trim_end_matches('/').to_string();

        let model_provider = settings
            .get("modelProvider")
            .and_then(Value::as_object)
            .map_or_else(
                || {
                    json!({
                        "name": "OpenAI",
                        "base_url": "https://api.openai.com/v1",
                        "env_key": "OPENAI_API_KEY",
                        "wire_api": "chat"
                    })
                },
                |provider| {
                    let get = |key: &str| provider.get(key).and_then(Value::as_str);
                    json!({
                        "name": get("name").unwrap_or("OpenAI"),
                        "base_url": get("base_url")
                            .or_else(|| get("baseUrl"))
                            .unwrap_or("https://api.openai.com/v1"),
                        "env_key": get("env_key")
                            .or_else(|| get("envKey"))
                            .unwrap_or("OPENAI_API_KEY"),
                        "wire_api": get("wire_api")
                            .or_else(|| get("wireApi"))
                            .unwrap_or("chat"),
                        "request_max_retries": provider
                            .get("request_max_retries")
                            .and_then(Value::as_u64),
                        "stream_max_retries": provider
                            .get("stream_max_retries")
                            .and_then(Value::as_u64),
                    })
                },
            );

        let raw_additional_toml = settings
            .get("rawToml")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
            .or_else(|| {
                settings
                    .get("raw_config")
                    .and_then(Value::as_str)
                    .map(std::string::ToString::to_string)
            });

        let raw_additional_json = settings
            .get("rawJson")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
            .or_else(|| {
                settings
                    .get("raw_json")
                    .and_then(Value::as_str)
                    .map(std::string::ToString::to_string)
            });
        let model_rotation = settings
            .get("modelRotation")
            .or_else(|| settings.get("modelCycle"))
            .or_else(|| cli_config.get("modelRotation"))
            .or_else(|| cli_config.get("modelCycle"))
            .and_then(|value| {
                // Handle both JSON array and JSON string representations
                match value {
                    Value::Array(arr) => Some(
                        arr.iter()
                            .filter_map(Value::as_str)
                            .map(std::string::ToString::to_string)
                            .collect::<Vec<_>>(),
                    ),
                    Value::String(s) => {
                        // Try to parse as JSON array
                        serde_json::from_str::<Vec<String>>(s).ok()
                    }
                    _ => None,
                }
            })
            .unwrap_or_default();
        let list_tools_on_start = settings
            .get("listToolsOnStart")
            .or_else(|| settings.get("listTools"))
            .or_else(|| cli_config.get("listToolsOnStart"))
            .or_else(|| cli_config.get("listTools"))
            .and_then(|value| match value {
                Value::Bool(flag) => Some(*flag),
                Value::String(s) => {
                    let normalized = s.trim().to_ascii_lowercase();
                    match normalized.as_str() {
                        "true" | "1" | "yes" | "on" => Some(true),
                        "false" | "0" | "no" | "off" => Some(false),
                        _ => None,
                    }
                }
                Value::Number(num) => num.as_i64().map(|int_val| int_val != 0),
                _ => None,
            })
            .unwrap_or(false);

        CliRenderSettings {
            model,
            temperature,
            max_output_tokens,
            approval_policy,
            sandbox_mode,
            project_doc_max_bytes,
            reasoning_effort,
            auto_level,
            output_format,
            editor_vim_mode,
            toolman_url,
            model_provider,
            raw_additional_toml,
            raw_additional_json,
            model_rotation,
            list_tools_on_start,
        }
    }

    fn extract_remote_tools(client_config_value: &Value) -> Vec<String> {
        client_config_value
            .get("remoteTools")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn generate_codex_config(
        code_run: &CodeRun,
        cli_config: &Value,
        client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_CODEX_CONFIG_TEMPLATE)?;

        handlebars
            .register_template_string("codex_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Codex config template: {e}"
                ))
            })?;
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let context = json!({
            "model": render_settings.model,
            "temperature": render_settings.temperature,
            "max_output_tokens": render_settings.max_output_tokens,
            "model_reasoning_effort": render_settings.reasoning_effort,
            "approval_policy": render_settings.approval_policy,
            "sandbox_mode": render_settings.sandbox_mode,
            "project_doc_max_bytes": render_settings.project_doc_max_bytes,
            "toolman": {
                "url": render_settings.toolman_url,
                "tools": remote_tools,
            },
            "model_provider": render_settings.model_provider,
            "cli_config": cli_config,
            "client_config": client_config,
            "raw_additional_toml": render_settings.raw_additional_toml,
        });

        handlebars.render("codex_config", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Codex config template: {e}"
            ))
        })
    }

    fn generate_codex_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        // Enrich cli_config with agent-level settings (like modelRotation)
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);

        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        templates.insert("client-config.json".to_string(), client_config);

        templates.insert(
            "container.sh".to_string(),
            Self::generate_codex_container_script(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "AGENTS.md".to_string(),
            Self::generate_codex_memory(code_run, &enriched_cli_config, &remote_tools)?,
        );

        templates.insert(
            "codex-config.toml".to_string(),
            Self::generate_codex_config(
                code_run,
                &enriched_cli_config,
                &client_config_value,
                &remote_tools,
            )?,
        );

        // Reuse shared guidance and hook generation across CLIs
        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        for (filename, content) in Self::generate_hook_scripts(code_run) {
            templates.insert(format!("hooks-{filename}"), content);
        }

        // Provide shared MCP configuration for Codex as well (Toolman passthrough)
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    #[allow(clippy::too_many_lines, clippy::items_after_statements)]
    fn generate_client_config(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        use serde_json::to_string_pretty;

        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        debug!(
            "🐛 DEBUG: generate_client_config called for github_app='{}'",
            github_app
        );
        debug!(
            "🐛 DEBUG: Available agents in config: {:?}",
            config.agents.keys().collect::<Vec<_>>()
        );
        debug!(
            "🐛 DEBUG: Agent github_app mappings: {:?}",
            config
                .agents
                .iter()
                .map(|(k, v)| (k, &v.github_app))
                .collect::<Vec<_>>()
        );

        // Helper: sanitize an arbitrary localServers value by dropping nulls and non-object entries
        let sanitize_local_servers = |v: &Value| -> Value {
            match v {
                Value::Object(map) => {
                    let mut out = serde_json::Map::new();
                    for (k, val) in map {
                        if let Value::Object(obj) = val {
                            out.insert(k.clone(), Value::Object(obj.clone()));
                        }
                    }
                    Value::Object(out)
                }
                _ => json!({}),
            }
        };

        // Helper to normalize a tools-shaped JSON ({"remote": [...], "localServers": {...}})
        // into the client-config.json shape without injecting additional servers.
        // Drops any null server entries produced by partial YAML (e.g., serverX: ~).
        let normalize_tools_to_client_config = |tools_value: Value| -> Value {
            let remote_tools = tools_value
                .get("remote")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let local_servers =
                sanitize_local_servers(tools_value.get("localServers").unwrap_or(&json!({})));
            let mut client = json!({
                "remoteTools": remote_tools,
                "localServers": local_servers
            });
            Self::normalize_remote_tools(&mut client);
            client
        };

        // Sanitize helper for full client-config objects (drop null/non-object server entries)
        let sanitize_client_local_servers = |v: &mut Value| {
            if let Some(ls) = v.get_mut("localServers") {
                let sanitized = sanitize_local_servers(ls);
                *ls = sanitized;
            }
        };

        // Helper: build client-config.json from strongly-typed AgentTools (Helm),
        // including only servers that are explicitly enabled. Generic over server names.
        let client_from_agent_tools = |tools: &crate::tasks::config::AgentTools| -> Value {
            let remote_tools: Value = json!(tools.remote.clone());

            let mut local_servers_obj = serde_json::Map::new();
            if let Some(ref servers) = tools.local_servers {
                for (name, cfg) in servers {
                    if cfg.enabled {
                        let mut obj = serde_json::Map::new();
                        if !cfg.tools.is_empty() {
                            obj.insert("tools".to_string(), json!(cfg.tools.clone()));
                        }
                        if let Some(cmd) = &cfg.command {
                            obj.insert("command".to_string(), json!(cmd));
                        }
                        if let Some(args) = &cfg.args {
                            obj.insert("args".to_string(), json!(args));
                        }
                        if let Some(wd) = &cfg.working_directory {
                            obj.insert("workingDirectory".to_string(), json!(wd));
                        }
                        local_servers_obj.insert(name.clone(), Value::Object(obj));
                    }
                }
            }

            let mut client = Value::Object(
                vec![
                    ("remoteTools".to_string(), remote_tools),
                    ("localServers".to_string(), Value::Object(local_servers_obj)),
                ]
                .into_iter()
                .collect(),
            );
            Self::normalize_remote_tools(&mut client);
            client
        };

        // Small helpers for merge logic
        let collect_string_array = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(std::string::ToString::to_string))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        };

        let merge_client_configs = |base: &Value, overlay: &Value| -> Value {
            // Merge remoteTools as a union preserving base order
            let mut merged_remote =
                collect_string_array(base.get("remoteTools").unwrap_or(&json!([])));
            let overlay_remote =
                collect_string_array(overlay.get("remoteTools").unwrap_or(&json!([])));
            for t in overlay_remote {
                if !merged_remote.contains(&t) {
                    merged_remote.push(t);
                }
            }

            // Merge localServers per server key, deep-merging fields; tools arrays are unioned
            let mut merged_local = serde_json::Map::new();
            let base_ls = base.get("localServers").and_then(|v| v.as_object());
            let overlay_ls = overlay.get("localServers").and_then(|v| v.as_object());

            // Collect all server keys
            use std::collections::BTreeSet;
            let mut keys = BTreeSet::new();
            if let Some(m) = base_ls {
                keys.extend(m.keys().cloned());
            }
            if let Some(m) = overlay_ls {
                keys.extend(m.keys().cloned());
            }

            for k in keys {
                let b = base_ls.and_then(|m| m.get(&k));
                let o = overlay_ls.and_then(|m| m.get(&k));

                let merged_val = match (b, o) {
                    (Some(Value::Object(bm)), Some(Value::Object(om))) => {
                        let mut out = bm.clone();
                        // tools union if present
                        let base_tools =
                            collect_string_array(out.get("tools").unwrap_or(&json!([])));
                        let overlay_tools =
                            collect_string_array(om.get("tools").unwrap_or(&json!([])));
                        if !base_tools.is_empty() || !overlay_tools.is_empty() {
                            let mut union = base_tools;
                            for t in overlay_tools {
                                if !union.contains(&t) {
                                    union.push(t);
                                }
                            }
                            out.insert("tools".to_string(), json!(union));
                        }
                        // Overlay scalar/object fields from overlay
                        for (ok, ov) in om {
                            if ok == "tools" {
                                continue;
                            }
                            out.insert(ok.clone(), ov.clone());
                        }
                        Value::Object(out)
                    }
                    (Some(bv), None) => bv.clone(),
                    (None, Some(ov)) => ov.clone(),
                    _ => json!({}),
                };
                merged_local.insert(k, merged_val);
            }

            json!({
                "remoteTools": merged_remote,
                "localServers": Value::Object(merged_local)
            })
        };

        // 1) Check CodeRun annotations for client-side tool configs first
        if let Some(annotations) = &code_run.metadata.annotations {
            if let Some(tools_config_str) = annotations.get("agents.platform/tools-config") {
                debug!(
                    "🐛 DEBUG: Found tools-config annotation: '{}'",
                    tools_config_str
                );
                debug!(
                    "🐛 DEBUG: Annotation trimmed: '{}', is_empty: {}, equals '{{}}': {}",
                    tools_config_str.trim(),
                    tools_config_str.trim().is_empty(),
                    tools_config_str == "{}"
                );
                if !tools_config_str.trim().is_empty() && tools_config_str != "{}" {
                    debug!(
                        "code: using tools config from CodeRun annotation for '{}'",
                        github_app
                    );

                    // Parse the tools config from annotation
                    match serde_json::from_str::<Value>(tools_config_str) {
                        Ok(mut tools_value) => {
                            // Accept both "tools" shape (remote + localServers) and full client-config shape (remoteTools + localServers)
                            let looks_like_client_cfg = tools_value.get("remoteTools").is_some()
                                || tools_value.get("localServers").is_some();

                            // Build overlay client config from annotation
                            let mut overlay_client = if looks_like_client_cfg {
                                if tools_value.get("remoteTools").is_none() {
                                    tools_value["remoteTools"] = json!([]);
                                }
                                if tools_value.get("localServers").is_none() {
                                    tools_value["localServers"] = json!({});
                                }
                                tools_value
                            } else {
                                normalize_tools_to_client_config(tools_value)
                            };
                            // Drop any null/non-object local server entries provided by client overlay
                            sanitize_client_local_servers(&mut overlay_client);

                            // Build base client config from Helm agent config (defaults)
                            let mut base_client = if let Some(agent_cfg) =
                                config.agents.values().find(|a| a.github_app == github_app)
                            {
                                if let Some(client_cfg) = &agent_cfg.client_config {
                                    client_cfg.clone()
                                } else if let Some(tools) = &agent_cfg.tools {
                                    client_from_agent_tools(tools)
                                } else {
                                    json!({ "remoteTools": [], "localServers": {} })
                                }
                            } else {
                                json!({ "remoteTools": [], "localServers": {} })
                            };
                            // Sanitize base as well
                            sanitize_client_local_servers(&mut base_client);

                            // Merge base (Helm defaults) + overlay (MCP client additions)
                            let mut merged = merge_client_configs(&base_client, &overlay_client);
                            // Final sanitize of merged result
                            sanitize_client_local_servers(&mut merged);
                            Self::normalize_remote_tools(&mut merged);
                            return to_string_pretty(&merged).map_err(|e| {
                                crate::tasks::types::Error::ConfigError(format!(
                                    "Failed to serialize merged clientConfig: {e}"
                                ))
                            });
                        }
                        Err(e) => {
                            debug!("code: failed to parse tools config annotation ({}), falling back to agent config", e);
                        }
                    }
                }
            }
        }

        // 2) Fall back to agent config from Helm values
        debug!(
            "🐛 DEBUG: Falling back to Helm agent config for github_app='{}'",
            github_app
        );
        if let Some(agent_cfg) = config.agents.values().find(|a| a.github_app == github_app) {
            debug!("🐛 DEBUG: Found matching agent config!");
            debug!(
                "code: matched agent config for githubApp='{}' (tools_present={}, clientConfig_present={})",
                github_app,
                agent_cfg.tools.is_some(),
                agent_cfg.client_config.is_some()
            );

            // 2a) Verbatim clientConfig
            if let Some(client_cfg) = &agent_cfg.client_config {
                debug!("code: using verbatim clientConfig for '{}'", github_app);
                let mut cfg = client_cfg.clone();
                sanitize_client_local_servers(&mut cfg);
                Self::normalize_remote_tools(&mut cfg);
                return to_string_pretty(&cfg).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to serialize clientConfig: {e}"
                    ))
                });
            }

            // 2b) Convert tools → client-config.json
            if let Some(tools) = &agent_cfg.tools {
                debug!(
                    "code: building clientConfig from tools for '{}' (remote_count={}, local_present={})",
                    github_app,
                    tools.remote.len(),
                    tools.local_servers.is_some()
                );

                // Build client-config including only explicitly enabled local servers
                let mut client = client_from_agent_tools(tools);
                sanitize_client_local_servers(&mut client);
                Self::normalize_remote_tools(&mut client);
                return to_string_pretty(&client).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to serialize tools-based clientConfig: {e}"
                    ))
                });
            }
        }

        // 3) No clientConfig/tools provided → minimal JSON object
        debug!("🐛 DEBUG: No matching agent found in Helm config!");
        debug!(
            "code: no tools/clientConfig found for '{}', using minimal config",
            github_app
        );
        // Always emit at least the two top-level keys so downstream validators don't treat it as empty
        // Do not inject any local servers by default; rely on Helm defaults or client config.
        let mut minimal_client = json!({ "remoteTools": [], "localServers": {} });
        Self::normalize_remote_tools(&mut minimal_client);

        to_string_pretty(&minimal_client).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize empty clientConfig: {e}"
            ))
        })
    }

    fn normalize_remote_tools(config: &mut Value) {
        if let Some(Value::Array(remote_tools)) = config.get_mut("remoteTools") {
            let mut normalized = Vec::new();
            let mut seen = HashSet::new();
            let mut dropped = Vec::new();

            for tool in remote_tools.iter() {
                let Some(name) = tool.as_str() else {
                    continue;
                };

                match resolve_tool_name(name) {
                    Some(canonical) => {
                        if seen.insert(canonical.clone()) {
                            if canonical != name {
                                debug!(
                                    original = name,
                                    canonical, "Normalized remote tool name using Toolman catalog"
                                );
                            }
                            normalized.push(Value::String(canonical));
                        }
                    }
                    None => dropped.push(name.to_string()),
                }
            }

            if !dropped.is_empty() {
                warn!(
                    tools = ?dropped,
                    "Removed unknown remote tools; not present in Toolman catalog"
                );
            }

            *remote_tools = normalized;
        }
    }

    fn generate_opencode_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let base_template = Self::load_template(CODE_OPENCODE_CONTAINER_BASE_TEMPLATE)?;
        handlebars
            .register_partial("opencode_container_base", base_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register OpenCode container base partial: {e}"
                ))
            })?;

        let template_path = Self::get_opencode_container_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("opencode_container", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register OpenCode container template {template_path}: {e}"
                ))
            })?;

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run
                .spec
                .docs_project_directory
                .as_deref()
                .unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "model": render_settings.model,
            "cli": {
                "type": Self::determine_cli_type(code_run).to_string(),
                "model": render_settings.model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
        });

        handlebars
            .render("opencode_container", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render OpenCode container template: {e}"
                ))
            })
    }

    fn generate_opencode_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template_path = Self::get_opencode_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("opencode_agents", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register OpenCode memory template {template_path}: {e}"
                ))
            })?;

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Extract model from cli_config like other templates do
        let model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&code_run.spec.model)
            .to_string();
        let cli_type = Self::determine_cli_type(code_run).to_string();

        let context = json!({
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "workflow_name": workflow_name,
            "toolman": {
                "tools": remote_tools,
            },
            "cli_config": cli_config,
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
        });

        handlebars.render("opencode_agents", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render OpenCode OPENCODE.md template: {e}"
            ))
        })
    }

    fn generate_opencode_config(
        code_run: &CodeRun,
        cli_config: &Value,
        client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_OPENCODE_CONFIG_TEMPLATE)?;

        handlebars
            .register_template_string("opencode_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register OpenCode config template: {e}"
                ))
            })?;

        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let provider_obj = render_settings
            .model_provider
            .as_object()
            .cloned()
            .unwrap_or_default();
        let provider_name = provider_obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("openai")
            .to_string();
        let provider_env_key = provider_obj
            .get("env_key")
            .or_else(|| provider_obj.get("envKey"))
            .and_then(Value::as_str)
            .unwrap_or("OPENAI_API_KEY")
            .to_string();
        let provider_base_url = provider_obj
            .get("base_url")
            .or_else(|| provider_obj.get("baseUrl"))
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string);

        let instructions_plain = cli_config
            .get("instructions")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
            .or_else(|| {
                cli_config
                    .get("memory")
                    .and_then(Value::as_str)
                    .map(std::string::ToString::to_string)
            });

        let local_servers_value = client_config
            .get("localServers")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let local_servers_serialized = if local_servers_value
            .as_object()
            .is_none_or(serde_json::Map::is_empty)
        {
            None
        } else {
            Some(
                serde_json::to_string_pretty(&local_servers_value)
                    .unwrap_or_else(|_| "{}".to_string()),
            )
        };

        let correlation_id = format!("task-{}", code_run.spec.task_id);

        let context = json!({
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "correlation_id": correlation_id,
                "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
                "cli": Self::determine_cli_type(code_run).to_string(),
            },
            "agent": {
                "model": render_settings.model,
                "temperature": render_settings.temperature,
                "max_output_tokens": render_settings.max_output_tokens,
                "instructions": instructions_plain,
                "remote_tools": remote_tools,
                "local_servers": local_servers_serialized,
                "toolman_url": render_settings.toolman_url,
                "provider": {
                    "name": provider_name,
                    "envKey": provider_env_key,
                    "base_url": provider_base_url,
                },
            },
        });

        handlebars.render("opencode_config", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render OpenCode config template: {e}"
            ))
        })
    }

    fn generate_gemini_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let base_template = Self::load_template(CODE_GEMINI_CONTAINER_BASE_TEMPLATE)?;
        handlebars
            .register_partial("gemini_container_base", base_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Gemini container base partial: {e}"
                ))
            })?;

        let template_path = Self::get_gemini_container_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("gemini_container", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Gemini container template {template_path}: {e}"
                ))
            })?;

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        // Parse continue_session from spec or env
        let continue_session = code_run.spec.continue_session
            || code_run
                .spec
                .env
                .get("CONTINUE_SESSION")
                .is_some_and(|v| v == "true");

        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "working_directory": Self::get_working_directory(code_run),
            "docs_branch": code_run.spec.docs_branch,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "remote_tools": remote_tools,
            "settings": cli_settings,
            "continue_session": continue_session,
            "overwrite_memory": code_run.spec.overwrite_memory,
            "attempts": retry_count + 1,
        });

        handlebars
            .render("gemini_container", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Gemini container script: {e}"
                ))
            })
    }

    #[allow(clippy::too_many_lines)]
    fn generate_gemini_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        use std::collections::BTreeSet;

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared agent system prompt partials
        Self::register_agent_partials(&mut handlebars)?;

        let template_path = Self::get_gemini_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("gemini_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Gemini memory template: {e}"
                ))
            })?;

        let workflow_name = extract_workflow_name(code_run)
            .unwrap_or_else(|_| format!("play-task-{}-workflow", code_run.spec.task_id));

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Derive allowed env var name lists for inclusion in memory
        // 1) Workflow-provided env names (keys only)
        let workflow_env_vars: Vec<String> = code_run.spec.env.keys().cloned().collect();

        // 2) From requirements.yaml (environment keys and mapped secret key names)
        let mut req_env_vars: Vec<String> = Vec::new();
        let mut req_secret_sources: Vec<String> = Vec::new();

        if let Some(req_b64) = &code_run.spec.task_requirements {
            if !req_b64.trim().is_empty() {
                if let Ok(decoded) = general_purpose::STANDARD.decode(req_b64) {
                    if let Ok(req_yaml) = serde_yaml::from_slice::<serde_yaml::Value>(&decoded) {
                        if let Some(env_map) =
                            req_yaml.get("environment").and_then(|e| e.as_mapping())
                        {
                            for (k, _v) in env_map {
                                if let Some(key) = k.as_str() {
                                    req_env_vars.push(key.to_string());
                                }
                            }
                        }
                        if let Some(secrets) = req_yaml.get("secrets").and_then(|s| s.as_sequence())
                        {
                            for secret in secrets {
                                if let Some(m) = secret.as_mapping() {
                                    if let Some(name) = m
                                        .get(serde_yaml::Value::from("name"))
                                        .and_then(|n| n.as_str())
                                    {
                                        req_secret_sources.push(name.to_string());
                                    }
                                    // If there are key mappings, surface the env var names (right-hand side)
                                    if let Some(keys) = m
                                        .get(serde_yaml::Value::from("keys"))
                                        .and_then(|k| k.as_sequence())
                                    {
                                        for entry in keys {
                                            if let Some(map) = entry.as_mapping() {
                                                for (_k8s_key, env_name) in map {
                                                    if let Some(n) = env_name.as_str() {
                                                        req_env_vars.push(n.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // De-duplicate and sort for stable output
        let wf_set: BTreeSet<_> = workflow_env_vars.into_iter().collect();
        let req_env_set: BTreeSet<_> = req_env_vars.into_iter().collect();
        let req_src_set: BTreeSet<_> = req_secret_sources.into_iter().collect();
        let workflow_env_vars: Vec<_> = wf_set.into_iter().collect();
        let requirements_env_vars: Vec<_> = req_env_set.into_iter().collect();
        let requirements_secret_sources: Vec<_> = req_src_set.into_iter().collect();

        let cli_type = Self::determine_cli_type(code_run).to_string();

        // Extract model from cli_config like other templates do
        let cli_model = cli_config
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(&code_run.spec.model)
            .to_string();

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "workflow_name": workflow_name,
            "remote_tools": remote_tools,
            "settings": cli_settings,
            "workflow_env_vars": workflow_env_vars,
            "requirements_env_vars": requirements_env_vars,
            "requirements_secret_sources": requirements_secret_sources,
            "cli_type": cli_type,
            "cli": {
                "type": cli_type,
                "model": cli_model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "toolman": {
                "tools": remote_tools,
            },
        });

        handlebars.render("gemini_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render Gemini memory template: {e}"
            ))
        })
    }

    fn generate_coding_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_CODING_GUIDELINES_TEMPLATE)?;

        handlebars
            .register_template_string("coding_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register coding-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
        });

        handlebars
            .render("coding_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render coding-guidelines.md: {e}"
                ))
            })
    }

    fn generate_github_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_GITHUB_GUIDELINES_TEMPLATE)?;

        handlebars
            .register_template_string("github_guidelines", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register github-guidelines.md template: {e}"
                ))
            })?;

        let context = json!({
            "service": code_run.spec.service,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
        });

        handlebars
            .render("github_guidelines", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render github-guidelines.md: {e}"
                ))
            })
    }

    fn generate_hook_scripts(code_run: &CodeRun) -> BTreeMap<String, String> {
        let mut hook_scripts = BTreeMap::new();
        let cli_key = code_run.spec.cli_config.as_ref().map_or_else(
            || CLIType::Claude.to_string(),
            |cfg| cfg.cli_type.to_string(),
        );

        let hook_prefixes = vec![
            format!("code_{}_hooks_", cli_key),
            "code_shared_hooks_".to_string(),
            "code_hooks_".to_string(), // legacy prefix
        ];

        debug!(
            cli = %cli_key,
            prefixes = ?hook_prefixes,
            "Scanning for code hook templates"
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(AGENT_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if std::path::Path::new(filename)
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("hbs"))
                            {
                                if let Some(prefix) = hook_prefixes
                                    .iter()
                                    .find(|prefix| filename.starts_with(prefix.as_str()))
                                {
                                    let hook_name =
                                        filename.strip_prefix(prefix).unwrap_or(filename);

                                    match std::fs::read_to_string(&path) {
                                        Ok(template_content) => {
                                            debug!(
                                                "Loaded code hook template: {} (from {})",
                                                hook_name, filename
                                            );

                                            let mut handlebars = Handlebars::new();
                                            handlebars.set_strict_mode(false);

                                            if let Err(e) = handlebars
                                                .register_template_string("hook", template_content)
                                            {
                                                debug!(
                                                    "Failed to register hook template {}: {}",
                                                    hook_name, e
                                                );
                                                continue;
                                            }

                                            let context = json!({
                                                "task_id": code_run.spec.task_id,
                                                "service": code_run.spec.service,
                                                "repository_url": code_run.spec.repository_url,
                                                "docs_repository_url": code_run.spec.docs_repository_url,
                                                "working_directory": Self::get_working_directory(code_run),
                                                "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
                                            });

                                            match handlebars.render("hook", &context) {
                                                Ok(rendered_script) => {
                                                    // Remove .hbs extension for the final filename
                                                    let script_name = hook_name
                                                        .strip_suffix(".hbs")
                                                        .unwrap_or(hook_name);
                                                    hook_scripts.insert(
                                                        script_name.to_string(),
                                                        rendered_script,
                                                    );
                                                }
                                                Err(e) => {
                                                    debug!(
                                                        "Failed to render code hook script {}: {}",
                                                        hook_name, e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            debug!(
                                                "Failed to load code hook template {}: {}",
                                                filename, e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        hook_scripts
    }

    /// Get working directory (defaults to service name if not specified)
    fn get_working_directory(code_run: &CodeRun) -> &str {
        match &code_run.spec.working_directory {
            Some(wd) if !wd.is_empty() => wd,
            _ => &code_run.spec.service,
        }
    }

    /// Get continue session flag - true for retries or user-requested continuation
    fn get_continue_session(code_run: &CodeRun) -> bool {
        // Continue if it's a retry attempt OR user explicitly requested it
        let retry_count = code_run
            .status
            .as_ref()
            .map_or(0, |s| s.retry_count.unwrap_or(0));
        retry_count > 0 || code_run.spec.continue_session
    }

    /// Select the appropriate container template based on the `github_app` field
    fn get_agent_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle (retry > 0)
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        // Map GitHub App to agent-specific container template
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => {
                if is_remediation {
                    "claude/container-rex-remediation.sh.hbs"
                } else {
                    "claude/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "claude/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "claude/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "claude/container-cleo.sh.hbs",
            "5DLabs-Tess" => "claude/container.sh.hbs",  // Use default container for Tess
            "5DLabs-Atlas" => "integration/container-atlas.sh.hbs",
            "5DLabs-Bolt" => "integration/container-bolt.sh.hbs",
            _ => {
                // Default to the generic container template for unknown agents
                debug!(
                    "No agent-specific template for '{}', using default container.sh.hbs",
                    github_app
                );
                "claude/container.sh.hbs"
            }
        };

        format!("code/{template_name}")
    }

    fn get_codex_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => {
                if is_remediation {
                    "code/codex/container-rex-remediation.sh.hbs"
                } else {
                    "code/codex/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "code/codex/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "code/codex/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "code/codex/container-cleo.sh.hbs",
            "5DLabs-Tess" => "code/codex/container-tess.sh.hbs",
            "5DLabs-Atlas" => "code/integration/container-atlas.sh.hbs",
            "5DLabs-Bolt" => "code/integration/container-bolt.sh.hbs",
            _ => "code/codex/container.sh.hbs",
        };

        template_name.to_string()
    }

    fn get_codex_memory_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => "code/codex/agents-rex.md.hbs",
            "5DLabs-Blaze" => "code/codex/agents-blaze.md.hbs",
            "5DLabs-Cipher" => "code/codex/agents-cipher.md.hbs",
            "5DLabs-Cleo" => "code/codex/agents-cleo.md.hbs",
            "5DLabs-Tess" => "agents/tess-system-prompt.md.hbs",
            "5DLabs-Atlas" => "agents/atlas-system-prompt.md.hbs",
            "5DLabs-Bolt" => "agents/bolt-system-prompt.md.hbs",
            _ => "code/codex/agents.md.hbs",
        };

        template_name.to_string()
    }

    fn get_opencode_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" | "5DLabs-Rex-Remediation" => {
                if is_remediation {
                    "code/opencode/container-rex-remediation.sh.hbs"
                } else {
                    "code/opencode/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "code/opencode/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "code/opencode/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "code/opencode/container-cleo.sh.hbs",
            "5DLabs-Tess" => "code/opencode/container-tess.sh.hbs",
            "5DLabs-Atlas" => "code/integration/container-atlas.sh.hbs",
            "5DLabs-Bolt" => "code/integration/container-bolt.sh.hbs",
            _ => "code/opencode/container.sh.hbs",
        };

        template_name.to_string()
    }

    fn get_opencode_memory_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" | "5DLabs-Rex-Remediation" => {
                "code/opencode/agents-rex.md.hbs"
            }
            "5DLabs-Blaze" => "code/opencode/agents-blaze.md.hbs",
            "5DLabs-Cipher" => "code/opencode/agents-cipher.md.hbs",
            "5DLabs-Cleo" => "code/opencode/agents-cleo.md.hbs",
            "5DLabs-Tess" => "agents/tess-system-prompt.md.hbs",
            "5DLabs-Atlas" => "agents/atlas-system-prompt.md.hbs",
            "5DLabs-Bolt" => "agents/bolt-system-prompt.md.hbs",
            _ => "code/opencode/agents.md.hbs",
        };

        template_name.to_string()
    }

    fn get_gemini_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => {
                if is_remediation {
                    "code/gemini/container-rex-remediation.sh.hbs"
                } else {
                    "code/gemini/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "code/gemini/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "code/gemini/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "code/gemini/container-cleo.sh.hbs",
            "5DLabs-Tess" => "code/gemini/container-tess.sh.hbs",
            _ => "code/gemini/container.sh.hbs",
        };

        template_name.to_string()
    }

    fn get_cursor_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => {
                if is_remediation {
                    "code/cursor/container-rex-remediation.sh.hbs"
                } else {
                    "code/cursor/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "code/cursor/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "code/cursor/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "code/cursor/container-cleo.sh.hbs",
            "5DLabs-Tess" => "code/cursor/container-tess.sh.hbs",
            "5DLabs-Atlas" => "code/integration/container-atlas.sh.hbs",
            "5DLabs-Bolt" => "code/integration/container-bolt.sh.hbs",
            _ => "code/cursor/container.sh.hbs",
        };

        template_name.to_string()
    }

    fn get_cursor_memory_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => "code/cursor/agents-rex.md.hbs",
            "5DLabs-Blaze" => "code/cursor/agents-blaze.md.hbs",
            "5DLabs-Cipher" => "code/cursor/agents-cipher.md.hbs",
            "5DLabs-Cleo" => "code/cursor/agents-cleo.md.hbs",
            "5DLabs-Tess" => "agents/tess-system-prompt.md.hbs",
            "5DLabs-Atlas" => "agents/atlas-system-prompt.md.hbs",
            "5DLabs-Bolt" => "agents/bolt-system-prompt.md.hbs",
            _ => "code/cursor/agents.md.hbs",
        };

        template_name.to_string()
    }

    fn get_gemini_memory_template(_code_run: &CodeRun) -> String {
        // Currently using single shared memory template for all Gemini agents
        // Can be extended in the future for agent-specific templates similar to other CLIs
        CODE_GEMINI_MEMORY_TEMPLATE.to_string()
    }

    fn get_factory_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Check if this is a remediation cycle
        let retry_count = code_run
            .status
            .as_ref()
            .and_then(|s| s.retry_count)
            .unwrap_or(0);

        let is_remediation = retry_count > 0;

        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" | "5DLabs-Rex-Remediation" => {
                if is_remediation {
                    "code/factory/container-rex-remediation.sh.hbs"
                } else {
                    "code/factory/container-rex.sh.hbs"
                }
            }
            "5DLabs-Blaze" => "code/factory/container-blaze.sh.hbs",
            "5DLabs-Cipher" => "code/factory/container-cipher.sh.hbs",
            "5DLabs-Cleo" => "code/factory/container-cleo.sh.hbs",
            "5DLabs-Tess" => "code/factory/container-tess.sh.hbs",
            "5DLabs-Atlas" => "code/integration/container-atlas.sh.hbs",
            "5DLabs-Bolt" => "code/integration/container-bolt.sh.hbs",
            _ => "code/factory/container.sh.hbs",
        };

        template_name.to_string()
    }

    fn get_factory_memory_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Morgan" => "code/factory/agents-rex.md.hbs",
            "5DLabs-Blaze" => "code/factory/agents-blaze.md.hbs",
            "5DLabs-Cipher" => "code/factory/agents-cipher.md.hbs",
            "5DLabs-Cleo" => "code/factory/agents-cleo.md.hbs",
            "5DLabs-Tess" => "agents/tess-system-prompt.md.hbs",
            "5DLabs-Atlas" => "agents/atlas-system-prompt.md.hbs",
            "5DLabs-Bolt" => "agents/bolt-system-prompt.md.hbs",
            _ => "code/factory/agents.md.hbs",
        };

        template_name.to_string()
    }

    /// Extract agent name from GitHub app identifier
    #[allow(dead_code)]
    fn extract_agent_name_from_github_app(github_app: &str) -> Result<String> {
        if github_app.is_empty() {
            return Err(crate::tasks::types::Error::ConfigError(
                "No GitHub app specified for agent identification".to_string(),
            ));
        }

        // Map GitHub app names to agent names
        let agent_name = match github_app {
            "5DLabs-Morgan" => "morgan",
            "5DLabs-Rex" => "rex",
            "5DLabs-Blaze" => "blaze",
            "5DLabs-Cipher" => "cipher",
            "5DLabs-Cleo" => "cleo",
            "5DLabs-Tess" => "tess",
            "5DLabs-Atlas" => "atlas",
            "5DLabs-Bolt" => "bolt",
            _ => {
                return Err(crate::tasks::types::Error::ConfigError(format!(
                    "Unknown GitHub app '{github_app}' - no corresponding agent found"
                )));
            }
        };

        Ok(agent_name.to_string())
    }

    /// Get agent tool configuration from controller config
    #[allow(dead_code)]
    fn get_agent_tools(
        agent_name: &str,
        config: &ControllerConfig,
    ) -> crate::tasks::config::AgentTools {
        use crate::tasks::config::AgentTools;

        // Try to get agent tools from controller config
        if let Some(agent_config) = config.agents.get(agent_name) {
            if let Some(tools) = &agent_config.tools {
                return AgentTools {
                    remote: tools.remote.clone(),
                    local_servers: tools.local_servers.clone(),
                };
            }
        }

        // Fallback to default configuration if agent not found or no tools configured
        debug!(
            "No agent-specific tools found for '{}', using defaults",
            agent_name
        );
        AgentTools {
            remote: vec![],
            local_servers: None,
        }
    }

    /// Load a template file from the mounted `ConfigMap`
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(AGENT_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading code template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load code template {relative_path} (key: {configmap_key}): {e}"
            ))
        })
    }

    /// Register shared agent system prompt partials
    /// These partials are used by agent-specific templates via {{> agents/partial-name}}
    fn register_agent_partials(handlebars: &mut Handlebars) -> Result<()> {
        // List of shared agent system prompt partials that need to be registered
        let agent_partials = vec![
            "agents/cipher-system-prompt",
            "agents/cleo-system-prompt",
            "agents/rex-system-prompt",
            "agents/tess-system-prompt",
            "agents/system-prompt",
        ];

        let mut failed_partials = Vec::new();

        for partial_name in agent_partials {
            // Load the partial template from ConfigMap
            // The ConfigMap key uses underscores instead of slashes (e.g., agents_cipher-system-prompt.md.hbs)
            let template_path = format!("{partial_name}.md.hbs");
            match Self::load_template(&template_path) {
                Ok(content) => {
                    handlebars
                        .register_partial(partial_name, content)
                        .map_err(|e| {
                            crate::tasks::types::Error::ConfigError(format!(
                                "Failed to register agent partial {partial_name}: {e}"
                            ))
                        })?;
                    debug!("Successfully registered agent partial: {}", partial_name);
                }
                Err(e) => {
                    // Warn but don't fail - the partial may not be needed for this specific agent
                    warn!(
                        "Failed to load agent partial {partial_name} from ConfigMap (path: {template_path}): {e}. \
                        Templates referencing this partial will fail to render."
                    );
                    failed_partials.push(partial_name);
                }
            }
        }

        // Log summary of partial registration
        if !failed_partials.is_empty() {
            warn!(
                "Agent partial registration incomplete. {} partials failed to load: {:?}. \
                Ensure the agent-templates ConfigMaps are properly mounted at {}",
                failed_partials.len(),
                failed_partials,
                AGENT_TEMPLATES_PATH
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::{CodeRun, CodeRunSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::{BTreeMap, HashMap};

    fn create_test_code_run(github_app: Option<String>) -> CodeRun {
        CodeRun {
            metadata: ObjectMeta {
                name: Some("test-run".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                cli_config: None,
                task_id: 1,
                service: "test-service".to_string(),
                repository_url: "https://github.com/test/repo".to_string(),
                docs_repository_url: "https://github.com/test/docs".to_string(),
                docs_project_directory: None,
                working_directory: None,
                model: "sonnet".to_string(),
                github_user: None,
                github_app,
                context_version: 1,
                continue_session: false,
                overwrite_memory: false,
                docs_branch: "main".to_string(),
                env: HashMap::new(),
                env_from_secrets: Vec::new(),
                enable_docker: true,
                task_requirements: None,
                service_account_name: None,
            },
            status: None,
        }
    }

    #[test]
    fn test_rex_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/claude/container-rex.sh.hbs");
    }

    #[test]
    fn test_cleo_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Cleo".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/claude/container-cleo.sh.hbs");
    }

    #[test]
    fn test_tess_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Tess".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        // For CLIType::Claude (default), path is prefixed with "code/" in template loading
        assert_eq!(template_path, "code/claude/container.sh.hbs");
    }

    #[test]
    fn test_cipher_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Cipher".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/claude/container-cipher.sh.hbs");
    }

    #[test]
    fn test_atlas_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Atlas".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/integration/container-atlas.sh.hbs");
    }

    #[test]
    fn test_bolt_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Bolt".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/integration/container-bolt.sh.hbs");
    }

    #[test]
    fn test_default_template_selection() {
        let code_run = create_test_code_run(None);
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/claude/container.sh.hbs");
    }

    #[test]
    fn test_extract_agent_name_from_github_app() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let agent_name = CodeTemplateGenerator::extract_agent_name_from_github_app(
            code_run.spec.github_app.as_deref().unwrap(),
        )
        .unwrap();
        assert_eq!(agent_name, "rex");
    }

    #[test]
    fn test_extract_agent_name_unknown_app() {
        let result = CodeTemplateGenerator::extract_agent_name_from_github_app("Unknown-App");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown GitHub app"));
    }

    #[test]
    fn test_extract_agent_name_empty_app() {
        let result = CodeTemplateGenerator::extract_agent_name_from_github_app("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No GitHub app specified"));
    }

    #[test]
    fn test_get_agent_tools_with_config() {
        use crate::tasks::config::{
            AgentDefinition, AgentTools, ControllerConfig, LocalServerConfig,
        };

        let mut config = ControllerConfig::default();
        let agent_tools = {
            use std::collections::BTreeMap;
            let mut servers = BTreeMap::new();
            servers.insert(
                "serverA".to_string(),
                LocalServerConfig {
                    enabled: true,
                    tools: vec!["read_file".to_string(), "write_file".to_string()],
                    command: None,
                    args: None,
                    working_directory: None,
                },
            );
            servers.insert(
                "serverB".to_string(),
                LocalServerConfig {
                    enabled: false,
                    tools: vec![],
                    command: None,
                    args: None,
                    working_directory: None,
                },
            );
            AgentTools {
                remote: vec![
                    "memory_create_entities".to_string(),
                    "brave_search_brave_web_search".to_string(),
                ],
                local_servers: Some(servers),
            }
        };

        config.agents.insert(
            "test-agent".to_string(),
            AgentDefinition {
                github_app: "Test-App".to_string(),
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(agent_tools.clone()),
                client_config: None,
                model_rotation: None,
            },
        );

        let code_run = create_test_code_run(Some("Test-App".to_string()));
        let _result = CodeTemplateGenerator::generate_client_config(&code_run, &config);

        // This will fail because the extract_agent_name_from_github_app doesn't know about Test-App
        // But we can test the logic by mocking or by using a known agent
        let known_code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));

        // Add rex agent to config with explicit clientConfig
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(agent_tools),
                model_rotation: None,
                client_config: Some(serde_json::json!({
                    "remoteTools": ["memory_create_entities", "brave_search_brave_web_search"],
                    "localServers": {
                        "serverA": {
                            "command": "npx",
                            "args": ["-y", "@example/mcp-server", "/workspace"],
                            "tools": ["read_file", "write_file"]
                        }
                    }
                })),
            },
        );

        let result = CodeTemplateGenerator::generate_client_config(&known_code_run, &config);
        assert!(result.is_ok());

        let client_config: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();

        // Verify structure
        assert!(client_config["remoteTools"].is_array());
        assert!(client_config["localServers"].is_object());

        // Verify remote tools
        let remote_tools = client_config["remoteTools"].as_array().unwrap();
        assert_eq!(remote_tools.len(), 2);
        assert!(remote_tools.contains(&serde_json::json!("memory_create_entities")));
        assert!(remote_tools.contains(&serde_json::json!("brave_search_brave_web_search")));

        // Verify local servers (generic server names)
        let local_servers = client_config["localServers"].as_object().unwrap();
        assert!(local_servers.contains_key("serverA"));
        assert!(!local_servers.contains_key("serverB"));

        let server_a = &local_servers["serverA"];
        assert_eq!(server_a["command"], "npx");
        assert!(server_a["tools"].is_array());
    }

    #[test]
    fn test_cursor_container_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_cursor_container_template(&code_run);
        assert_eq!(template_path, "code/cursor/container-rex.sh.hbs");
    }

    #[test]
    fn test_cursor_memory_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Cleo".to_string()));
        let template_path = CodeTemplateGenerator::get_cursor_memory_template(&code_run);
        assert_eq!(template_path, "code/cursor/agents-cleo.md.hbs");
    }

    #[test]
    fn test_merge_client_config_overlay_on_helm_defaults() {
        use crate::tasks::config::{
            AgentDefinition, AgentTools, ControllerConfig, LocalServerConfig,
        };

        // Helm defaults for rex
        let mut config = ControllerConfig::default();
        let helm_tools = {
            use std::collections::BTreeMap;
            let mut servers = BTreeMap::new();
            servers.insert(
                "serverA".to_string(),
                LocalServerConfig {
                    enabled: true,
                    tools: vec!["read_file".to_string(), "write_file".to_string()],
                    command: None,
                    args: None,
                    working_directory: None,
                },
            );
            AgentTools {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: Some(servers),
            }
        };
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(helm_tools),
                client_config: None,
                model_rotation: None,
            },
        );

        // CodeRun with annotation overlay client config
        let mut code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let mut ann = BTreeMap::new();
        ann.insert(
            "agents.platform/tools-config".to_string(),
            serde_json::json!({
                "remoteTools": ["brave_search_brave_web_search"],
                "localServers": {
                    "serverA": {
                        "tools": ["list_directory"],
                        "workingDirectory": "overlay_dir"
                    }
                }
            })
            .to_string(),
        );
        code_run.metadata.annotations = Some(ann);

        let result = CodeTemplateGenerator::generate_client_config(&code_run, &config).unwrap();
        let client_config: serde_json::Value = serde_json::from_str(&result).unwrap();

        // remoteTools should be union of helm + overlay (order preserved: helm, then overlay)
        let remote = client_config["remoteTools"].as_array().unwrap();
        assert!(remote.contains(&serde_json::json!("memory_create_entities")));
        assert!(remote.contains(&serde_json::json!("brave_search_brave_web_search")));

        // serverA tools should include helm tools plus overlay tool
        let fs = &client_config["localServers"]["serverA"];
        let tools = fs["tools"].as_array().unwrap();
        assert!(tools.contains(&serde_json::json!("read_file")));
        assert!(tools.contains(&serde_json::json!("write_file")));
        assert!(tools.contains(&serde_json::json!("list_directory")));

        // workingDirectory should be overlaid
        assert_eq!(fs["workingDirectory"], "overlay_dir");
    }
}
