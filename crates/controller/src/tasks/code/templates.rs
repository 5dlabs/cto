use crate::cli::types::CLIType;
use crate::crds::CodeRun;
use crate::tasks::code::agent::AgentClassifier;
use crate::tasks::config::ControllerConfig;
use crate::tasks::template_paths::{
    CODE_CLAUDE_CONTAINER_TEMPLATE, CODE_CODEX_CONTAINER_BASE_TEMPLATE,
    CODE_CODING_GUIDELINES_TEMPLATE, CODE_CURSOR_CONTAINER_BASE_TEMPLATE,
    CODE_FACTORY_CONTAINER_BASE_TEMPLATE, CODE_GEMINI_CONTAINER_BASE_TEMPLATE,
    CODE_GITHUB_GUIDELINES_TEMPLATE, CODE_MCP_CONFIG_TEMPLATE,
    CODE_OPENCODE_CONTAINER_BASE_TEMPLATE,
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

// Template base path (embedded in Docker image at /app/templates)
// Set via AGENT_TEMPLATES_PATH env var in deployment
const DEFAULT_AGENT_TEMPLATES_PATH: &str = "/app/templates";

/// Get the agent templates directory path.
/// Uses `AGENT_TEMPLATES_PATH` env var if set, otherwise defaults to `/app/templates`.
fn get_templates_path() -> String {
    std::env::var("AGENT_TEMPLATES_PATH")
        .unwrap_or_else(|_| DEFAULT_AGENT_TEMPLATES_PATH.to_string())
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields are set but not currently used after template migration
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
    tools_url: String,
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
        // Check run_type first for review/remediate tasks
        match code_run.spec.run_type.as_str() {
            "review" => return Self::generate_review_templates(code_run, config),
            "remediate" => return Self::generate_remediate_templates(code_run, config),
            _ => {}
        }

        // Fall through to CLI-based dispatch for other run types
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
            Self::generate_cursor_project_permissions(
                code_run,
                &enriched_cli_config,
                &remote_tools,
            )?,
        );

        templates.insert(
            "cursor-mcp.json".to_string(),
            Self::generate_cursor_mcp_config(code_run, &enriched_cli_config, &remote_tools)?,
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

        templates.insert(
            "settings.json".to_string(),
            Self::generate_gemini_settings(code_run, &enriched_cli_config)?,
        );

        Ok(templates)
    }

    fn generate_gemini_settings(_code_run: &CodeRun, cli_config: &Value) -> Result<String> {
        // Build context config
        let context = json!({
            "fileName": ["AGENTS.md", "GEMINI.md"],
            "fileFiltering": "gitignore"
        });

        // Build advanced config with optional bug command
        let mut advanced = json!({});
        if let Some(bug_cmd) = cli_config.get("bug_command").and_then(Value::as_str) {
            advanced["bugCommand"] = json!(bug_cmd);
        }

        // Build sandboxing config
        let sandbox_profile = cli_config
            .get("sandbox_profile")
            .and_then(Value::as_str)
            .unwrap_or("permissive");
        let sandboxing = json!({
            "profile": sandbox_profile
        });

        // Build complete settings config (no template needed - serialize directly)
        let settings = json!({
            "context": context,
            "theme": "default",
            "advanced": advanced,
            "sandboxing": sandboxing,
            "checkpointing": {
                "enabled": true
            }
        });

        serde_json::to_string_pretty(&settings).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Gemini settings.json: {e}"
            ))
        })
    }

    fn generate_cursor_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the cli_execute partial
        Self::register_cli_invoke_partial(&mut handlebars, CLIType::Cursor)?;

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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "source_branch": code_run.spec.docs_branch,
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
            "enable_docker": code_run.spec.enable_docker,
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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "tools": {
                "tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
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
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        // Build model config
        let mut model_config = json!({
            "default": render_settings.model
        });
        if let Some(temp) = render_settings.temperature {
            model_config["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = render_settings.max_output_tokens {
            model_config["maxOutputTokens"] = json!(max_tokens);
        }

        // Build MCP servers config
        let mut mcp_servers = json!({});
        if !render_settings.tools_url.is_empty() {
            let mut tools_server = json!({
                "command": "tools",
                "args": ["--url", render_settings.tools_url],
                "env": {
                    "TOOLS_SERVER_URL": render_settings.tools_url
                }
            });
            if !remote_tools.is_empty() {
                tools_server["availableTools"] = json!(remote_tools);
            }
            mcp_servers["tools"] = tools_server;
        }

        // Build automation config
        let automation = json!({
            "approvalPolicy": render_settings.approval_policy,
            "sandboxMode": render_settings.sandbox_mode,
            "projectDocMaxBytes": render_settings.project_doc_max_bytes
        });

        // Build complete config (no template needed - serialize directly)
        let config = json!({
            "version": 1,
            "hasChangedDefaultModel": true,
            "model": model_config,
            "editor": {
                "vimMode": render_settings.editor_vim_mode
            },
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)" ],
                "deny": []
            },
            "automation": automation,
            "mcp": {
                "servers": mcp_servers
            }
        });

        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Cursor CLI config: {e}"
            ))
        })
    }

    fn generate_cursor_project_permissions(
        _code_run: &CodeRun,
        _cli_config: &Value,
        _remote_tools: &[String],
    ) -> Result<String> {
        // Generate Cursor project permissions config (not MCP config)
        let config = json!({
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)" ],
                "deny": []
            }
        });
        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Cursor project permissions: {e}"
            ))
        })
    }

    fn generate_cursor_mcp_config(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        // Build MCP servers config (no template needed - serialize directly)
        let mut mcp_servers = json!({});

        if !render_settings.tools_url.is_empty() {
            let mut tools_server = json!({
                "command": "tools",
                "args": ["--url", render_settings.tools_url, "--working-dir", "/workspace"],
                "env": {
                    "TOOLS_SERVER_URL": render_settings.tools_url
                }
            });
            if !remote_tools.is_empty() {
                tools_server["availableTools"] = json!(remote_tools);
            }
            mcp_servers["tools"] = tools_server;
        }

        let config = json!({
            "mcpServers": mcp_servers
        });

        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Cursor MCP config: {e}"
            ))
        })
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

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the cli_execute partial
        Self::register_cli_invoke_partial(&mut handlebars, CLIType::Factory)?;

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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        // Extract watch-specific variables from env
        let iteration = code_run
            .spec
            .env
            .get("ITERATION")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        let max_iterations = code_run
            .spec
            .env
            .get("MAX_ITERATIONS")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);
        let target_repository = code_run
            .spec
            .env
            .get("TARGET_REPOSITORY")
            .cloned()
            .unwrap_or_default();
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("cto");

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "enable_docker": code_run.spec.enable_docker,
            // Watch-specific context
            "iteration": iteration,
            "max_iterations": max_iterations,
            "target_repository": target_repository,
            "namespace": namespace,
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
        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let cli_settings = cli_config
            .get("settings")
            .cloned()
            .unwrap_or_else(|| json!({}));

        // Extract iteration from env or settings for watch workflows
        let iteration = code_run
            .spec
            .env
            .get("ITERATION")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": render_settings.model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "iteration": iteration,
            "cli": {
                "type": Self::determine_cli_type(code_run).to_string(),
                "model": render_settings.model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            "tools": {
                "tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
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
        _client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        // Build model config
        let mut model_config = json!({
            "default": render_settings.model
        });
        if let Some(ref effort) = render_settings.reasoning_effort {
            model_config["reasoningEffort"] = json!(effort);
        }
        if let Some(temp) = render_settings.temperature {
            model_config["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = render_settings.max_output_tokens {
            model_config["maxOutputTokens"] = json!(max_tokens);
        }

        // Determine auto run level
        let auto_level = render_settings
            .auto_level
            .as_ref()
            .or(render_settings.reasoning_effort.as_ref())
            .map_or("high", String::as_str);

        // Build execution config
        let execution = json!({
            "approvalPolicy": render_settings.approval_policy,
            "sandboxMode": render_settings.sandbox_mode,
            "projectDocMaxBytes": render_settings.project_doc_max_bytes
        });

        // Build base config (no template needed - serialize directly)
        let mut config = json!({
            "version": 1,
            "model": model_config,
            "autoRun": {
                "enabled": true,
                "level": auto_level
            },
            "specificationMode": {
                "default": true
            },
            "execution": execution,
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)" ],
                "deny": []
            }
        });

        // Add tools config if URL is provided
        if !render_settings.tools_url.is_empty() {
            let mut tools_config = json!({
                "endpoint": render_settings.tools_url
            });
            if !remote_tools.is_empty() {
                tools_config["tools"] = json!(remote_tools);
            }
            config["tools"] = tools_config;

            // Also add MCP servers
            let mut mcp_tools_server = json!({
                "command": "tools",
                "args": ["--url", render_settings.tools_url],
                "env": {
                    "TOOLS_SERVER_URL": render_settings.tools_url
                }
            });
            if !remote_tools.is_empty() {
                mcp_tools_server["availableTools"] = json!(remote_tools);
            }
            config["mcp"] = json!({
                "servers": {
                    "tools": mcp_tools_server
                }
            });
        }

        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Factory CLI config: {e}"
            ))
        })
    }

    fn generate_factory_project_permissions() -> Result<String> {
        // Generate a minimal permissions config (no template needed)
        let config = json!({
            "version": 1,
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)" ],
                "deny": []
            }
        });
        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Factory project permissions: {e}"
            ))
        })
    }

    fn generate_container_script(code_run: &CodeRun, cli_config: &Value) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the `cli_execute` partial
        let cli_type = Self::determine_cli_type(code_run);
        Self::register_cli_invoke_partial(&mut handlebars, cli_type)?;

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
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "source_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "attempts": retry_count + 1,  // Current attempt number (1-indexed)
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
            "cli_config": cli_config,
            "enable_docker": code_run.spec.enable_docker,
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

        let template_path = Self::get_claude_memory_template(code_run);
        let template = Self::load_template(&template_path)?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template {template_path}: {e}"
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

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "tools": {
                "tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(
        code_run: &CodeRun,
        _config: &ControllerConfig,
        cli_config: &Value,
    ) -> Result<String> {
        // Check for agent-specific tool overrides
        let agent_tools_override = cli_config
            .get("permissions")
            .and_then(|p| p.get("allow"))
            .is_some();

        // Build permissions based on configuration
        let permissions = if agent_tools_override {
            json!({
                "allow": cli_config.get("permissions").and_then(|p| p.get("allow")).cloned().unwrap_or(json!([])),
                "deny": cli_config.get("permissions").and_then(|p| p.get("deny")).cloned().unwrap_or(json!([])),
                "defaultMode": "acceptEdits"
            })
        } else {
            // Default allowed tools for Claude Code CLI
            json!({
                "allow": [
                    "Bash", "Create", "Edit", "Read", "Write", "MultiEdit",
                    "Glob", "Grep", "LS", "Task", "ExitPlanMode", "ExitSpecMode",
                    "NotebookRead", "NotebookEdit", "WebFetch", "WebSearch",
                    "TodoRead", "TodoWrite", "GenerateDroid"
                ],
                "deny": [],
                "defaultMode": "acceptEdits"
            })
        };

        // Build environment variables
        let is_retry = cli_config
            .get("retry")
            .and_then(|r| r.get("is_retry"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let telemetry_enabled = cli_config
            .get("telemetry")
            .and_then(|t| t.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let mut env = json!({
            "NODE_ENV": "production",
            "DISABLE_AUTOUPDATER": "1",
            "DISABLE_COST_WARNINGS": "0",
            "DISABLE_NON_ESSENTIAL_MODEL_CALLS": "0",
            "CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR": "true",
            "CLAUDE_CODE_ENABLE_TELEMETRY": if telemetry_enabled { "1" } else { "0" }
        });

        if telemetry_enabled {
            env["OTEL_METRICS_EXPORTER"] = json!("otlp");
            env["OTEL_LOGS_EXPORTER"] = json!("otlp");
            env["OTEL_EXPORTER_OTLP_METRICS_ENDPOINT"] = json!(
                "otel-collector-opentelemetry-collector.observability.svc.cluster.local:4317"
            );
            env["OTEL_EXPORTER_OTLP_METRICS_PROTOCOL"] = json!("grpc");
            env["OTEL_EXPORTER_OTLP_LOGS_ENDPOINT"] = json!(
                "otel-collector-opentelemetry-collector.observability.svc.cluster.local:4317"
            );
            env["OTEL_EXPORTER_OTLP_LOGS_PROTOCOL"] = json!("grpc");
        }

        if is_retry {
            env["BASH_DEFAULT_TIMEOUT_MS"] = json!("30000");
            env["BASH_MAX_TIMEOUT_MS"] = json!("300000");
        }

        // Build complete settings config (no template needed - serialize directly)
        let settings = json!({
            "enableAllProjectMcpServers": true,
            "permissions": permissions,
            "env": env,
            "model": code_run.spec.model,
            "cleanupPeriodDays": 7,
            "includeCoAuthoredBy": false
        });

        serde_json::to_string_pretty(&settings).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize Claude settings.json: {e}"
            ))
        })
    }

    fn generate_mcp_config(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(CODE_MCP_CONFIG_TEMPLATE)?;

        handlebars
            .register_template_string("mcp_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register MCP config template: {e}"
                ))
            })?;

        // Get CLI config to extract tools URL and tools
        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        let render_settings = Self::build_cli_render_settings(code_run, &cli_config_value);

        // Generate client config and extract remote tools (same pattern as other functions)
        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        let context = json!({
            "tools_url": render_settings.tools_url,
            "tools_tools": remote_tools,
        });

        handlebars.render("mcp_config", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render MCP config template: {e}"
            ))
        })
    }

    fn generate_codex_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the cli_execute partial
        Self::register_cli_invoke_partial(&mut handlebars, CLIType::Codex)?;

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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let cli_model = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or_else(|| code_run.spec.model.clone(), |cfg| cfg.model.clone());

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "source_branch": code_run.spec.docs_branch,
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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

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

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "cli_config": cli_config,
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "tools": {
                "tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
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

            // If agent has frontend stack preference, inject it into cli_config
            // This is primarily used by Blaze agent for shadcn vs tanstack stack selection
            if let Some(frontend_stack) = &agent_config.frontend_stack {
                if enriched.get("frontendStack").is_none() {
                    enriched["frontendStack"] = json!(frontend_stack);
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

        let mut tools_url = settings
            .get("toolsUrl")
            .and_then(Value::as_str)
            .map_or_else(
                || {
                    std::env::var("TOOLS_SERVER_URL").unwrap_or_else(|_| {
                        "http://tools.cto.svc.cluster.local:3000/mcp".to_string()
                    })
                },
                std::string::ToString::to_string,
            );
        tools_url = tools_url.trim_end_matches('/').to_string();

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
            tools_url,
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

    #[allow(clippy::unnecessary_wraps)] // Keeping Result for consistency with other config generators
    fn generate_codex_config(
        code_run: &CodeRun,
        cli_config: &Value,
        _client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        use std::fmt::Write;
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        // Generate TOML directly (no template needed)
        let mut toml = String::from(
            "# Codex CLI Configuration\n\
             # Reference: https://developers.openai.com/codex/local-config#cli\n\
             # Generated by CTO controller\n\n\
             # Model configuration\n",
        );

        let _ = writeln!(toml, "model = \"{}\"", render_settings.model);

        if let Some(temp) = render_settings.temperature {
            let _ = writeln!(toml, "temperature = {temp}");
        }
        if let Some(max_tokens) = render_settings.max_output_tokens {
            let _ = writeln!(toml, "model_max_output_tokens = {max_tokens}");
        }
        if let Some(ref effort) = render_settings.reasoning_effort {
            let _ = writeln!(toml, "model_reasoning_effort = \"{effort}\"");
        }

        // Automation settings
        toml.push_str("\n# Automation settings\n");
        let _ = write!(
            toml,
            "# approval_policy: untrusted | on-failure | on-request | never\n\
             approval_policy = \"{}\"\n",
            render_settings.approval_policy
        );
        let _ = write!(
            toml,
            "\n# sandbox_mode: read-only | workspace-write | danger-full-access\n\
             sandbox_mode = \"{}\"\n",
            render_settings.sandbox_mode
        );
        let _ = write!(
            toml,
            "\n# Project documentation limits\n\
             project_doc_max_bytes = {}\n",
            render_settings.project_doc_max_bytes
        );

        // Tools MCP server
        if !remote_tools.is_empty() && !render_settings.tools_url.is_empty() {
            let tools_list: Vec<String> =
                remote_tools.iter().map(|t| format!("  \"{t}\"")).collect();
            let _ = write!(
                toml,
                "\n# Tools MCP server for remote tools\n\
                 [mcp_servers.tools]\n\
                 command = \"tools\"\n\
                 args = [\n  \"--url\",\n  \"{}\",\n  \"--working-dir\",\n  \"/workspace\"\n]\n\
                 env = {{ \"TOOLS_SERVER_URL\" = \"{}\" }}\n\
                 startup_timeout_sec = 30\n\
                 tool_timeout_sec = 120\n\
                 # Tool filtering: only expose the tools configured for this agent\n\
                 available_tools = [\n{}\n]\n",
                render_settings.tools_url,
                render_settings.tools_url,
                tools_list.join(",\n")
            );
        }

        // Model provider (only add if it's a non-empty object)
        if let Some(provider) = render_settings.model_provider.as_object() {
            if !provider.is_empty() {
                let name = provider
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("OpenAI");
                let base_url = provider
                    .get("base_url")
                    .and_then(Value::as_str)
                    .unwrap_or("https://api.openai.com/v1");
                let env_key = provider
                    .get("env_key")
                    .and_then(Value::as_str)
                    .unwrap_or("OPENAI_API_KEY");
                let wire_api = provider
                    .get("wire_api")
                    .and_then(Value::as_str)
                    .unwrap_or("chat");

                let _ = write!(
                    toml,
                    "\n[model_providers.openai]\n\
                     name = \"{name}\"\n\
                     base_url = \"{base_url}\"\n\
                     env_key = \"{env_key}\"\n\
                     wire_api = \"{wire_api}\"\n"
                );

                if let Some(max_retries) =
                    provider.get("request_max_retries").and_then(Value::as_u64)
                {
                    let _ = writeln!(toml, "request_max_retries = {max_retries}");
                }
                if let Some(max_retries) =
                    provider.get("stream_max_retries").and_then(Value::as_u64)
                {
                    let _ = writeln!(toml, "stream_max_retries = {max_retries}");
                }
            }
        }

        // Raw additional TOML
        if let Some(ref raw_toml) = render_settings.raw_additional_toml {
            let _ = write!(toml, "\n{raw_toml}\n");
        }

        Ok(toml)
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

        // Provide shared MCP configuration for Codex as well (Tools passthrough)
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    /// Generate templates for review tasks (Stitch PR Review)
    #[allow(clippy::too_many_lines)]
    fn generate_review_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        use crate::cli::types::CLIType;
        use crate::tasks::template_paths::{
            REVIEW_CLAUDE_AGENTS_TEMPLATE, REVIEW_CLAUDE_CONTAINER_TEMPLATE,
            REVIEW_FACTORY_AGENTS_TEMPLATE, REVIEW_FACTORY_CONTAINER_TEMPLATE,
            REVIEW_FACTORY_POST_REVIEW_TEMPLATE,
        };

        let mut templates = BTreeMap::new();
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Determine CLI type (default to Factory for review tasks)
        let cli_type = Self::determine_cli_type(code_run);
        let use_claude = matches!(cli_type, CLIType::Claude);

        // Extract PR number from labels or env
        let pr_number = code_run
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("pr-number"))
            .cloned()
            .or_else(|| code_run.spec.env.get("PR_NUMBER").cloned())
            .unwrap_or_default();

        // Extract head SHA from env
        let head_sha = code_run
            .spec
            .env
            .get("HEAD_SHA")
            .cloned()
            .unwrap_or_default();

        // Extract review mode from env or default
        let review_mode = code_run
            .spec
            .env
            .get("REVIEW_MODE")
            .cloned()
            .unwrap_or_else(|| "review".to_string());

        // Extract trigger from env
        let trigger = code_run
            .spec
            .env
            .get("TRIGGER")
            .cloned()
            .unwrap_or_else(|| "pull_request".to_string());

        // Extract repo slug from repository URL (shared helper)
        let repo_slug = Self::extract_repo_slug(&code_run.spec.repository_url);

        // Generate MCP client config and extract remote tools
        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        // Get tools URL from cli_config settings (using shared helper)
        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));
        let render_settings = Self::build_cli_render_settings(code_run, &cli_config_value);

        let context = json!({
            "github_app": code_run.spec.github_app.as_deref().unwrap_or("5DLabs-Stitch"),
            "pr_number": pr_number,
            "repository_url": code_run.spec.repository_url,
            "repo_slug": repo_slug,
            "head_sha": head_sha,
            "review_mode": review_mode,
            "trigger": trigger,
            "model": code_run.spec.model,
            "tools_url": render_settings.tools_url,
            "remote_tools": remote_tools,
        });

        // Generate client-config.json for MCP tools
        templates.insert("client-config.json".to_string(), client_config);

        // Select template paths based on CLI type
        let (container_path, agents_path) = if use_claude {
            (
                REVIEW_CLAUDE_CONTAINER_TEMPLATE,
                REVIEW_CLAUDE_AGENTS_TEMPLATE,
            )
        } else {
            (
                REVIEW_FACTORY_CONTAINER_TEMPLATE,
                REVIEW_FACTORY_AGENTS_TEMPLATE,
            )
        };

        // Load and render container script
        let container_template = Self::load_template(container_path)?;
        handlebars
            .register_template_string("review_container", container_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register review container template: {e}"
                ))
            })?;

        templates.insert(
            "container.sh".to_string(),
            handlebars
                .render("review_container", &context)
                .map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to render review container template: {e}"
                    ))
                })?,
        );

        // Load and render agents.md
        let agents_template = Self::load_template(agents_path)?;
        handlebars
            .register_template_string("review_agents", agents_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register review agents template: {e}"
                ))
            })?;

        templates.insert(
            "agents.md".to_string(),
            handlebars.render("review_agents", &context).map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render review agents template: {e}"
                ))
            })?,
        );

        // Load post_review.py helper script (Factory only, but include for both)
        let post_review_script = Self::load_template(REVIEW_FACTORY_POST_REVIEW_TEMPLATE)?;
        templates.insert("post_review.py".to_string(), post_review_script);

        // Generate MCP config
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        Ok(templates)
    }

    /// Generate templates for remediate tasks (Rex PR Remediation)
    #[allow(clippy::too_many_lines)]
    fn generate_remediate_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        use crate::cli::types::CLIType;
        use crate::tasks::template_paths::{
            REMEDIATE_CLAUDE_AGENTS_TEMPLATE, REMEDIATE_CLAUDE_CONTAINER_TEMPLATE,
            REMEDIATE_FACTORY_AGENTS_TEMPLATE, REMEDIATE_FACTORY_CONTAINER_TEMPLATE,
        };

        let mut templates = BTreeMap::new();
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Determine CLI type (default to Factory for remediate tasks)
        let cli_type = Self::determine_cli_type(code_run);
        let use_claude = matches!(cli_type, CLIType::Claude);

        // Extract PR number from labels or env
        let pr_number = code_run
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("pr-number"))
            .cloned()
            .or_else(|| code_run.spec.env.get("PR_NUMBER").cloned())
            .unwrap_or_default();

        // Extract head SHA from env
        let head_sha = code_run
            .spec
            .env
            .get("HEAD_SHA")
            .cloned()
            .unwrap_or_default();

        // Extract repo slug from repository URL (shared helper)
        let repo_slug = Self::extract_repo_slug(&code_run.spec.repository_url);

        // Extract review comment ID if triggered by review comment
        let review_comment_id = code_run
            .spec
            .env
            .get("REVIEW_COMMENT_ID")
            .cloned()
            .unwrap_or_default();

        // Extract findings JSON path if available
        let findings_path = code_run
            .spec
            .env
            .get("FINDINGS_PATH")
            .cloned()
            .unwrap_or_default();

        // Generate MCP client config and extract remote tools
        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        // Get tools URL from cli_config settings (using shared helper)
        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));
        let render_settings = Self::build_cli_render_settings(code_run, &cli_config_value);

        let context = json!({
            "github_app": code_run.spec.github_app.as_deref().unwrap_or("5DLabs-Rex"),
            "pr_number": pr_number,
            "repository_url": code_run.spec.repository_url,
            "repo_slug": repo_slug,
            "head_sha": head_sha,
            "model": code_run.spec.model,
            "review_comment_id": review_comment_id,
            "findings_path": findings_path,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "tools_url": render_settings.tools_url,
            "remote_tools": remote_tools,
        });

        // Generate client-config.json for MCP tools
        templates.insert("client-config.json".to_string(), client_config);

        // Select template paths based on CLI type
        let (container_path, agents_path) = if use_claude {
            (
                REMEDIATE_CLAUDE_CONTAINER_TEMPLATE,
                REMEDIATE_CLAUDE_AGENTS_TEMPLATE,
            )
        } else {
            (
                REMEDIATE_FACTORY_CONTAINER_TEMPLATE,
                REMEDIATE_FACTORY_AGENTS_TEMPLATE,
            )
        };

        // Load and render container script
        let container_template = Self::load_template(container_path)?;
        handlebars
            .register_template_string("remediate_container", container_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register remediate container template: {e}"
                ))
            })?;

        templates.insert(
            "container.sh".to_string(),
            handlebars
                .render("remediate_container", &context)
                .map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to render remediate container template: {e}"
                    ))
                })?,
        );

        // Load and render agents.md
        let agents_template = Self::load_template(agents_path)?;
        handlebars
            .register_template_string("remediate_agents", agents_template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register remediate agents template: {e}"
                ))
            })?;

        templates.insert(
            "agents.md".to_string(),
            handlebars
                .render("remediate_agents", &context)
                .map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to render remediate agents template: {e}"
                    ))
                })?,
        );

        // Generate MCP config
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
                                    canonical, "Normalized remote tool name using Tools catalog"
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
                    "Removed unknown remote tools; not present in Tools catalog"
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

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the cli_execute partial
        Self::register_cli_invoke_partial(&mut handlebars, CLIType::OpenCode)?;

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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

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

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "workflow_name": workflow_name,
            "tools": {
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
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
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
        _client_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let provider_obj = render_settings
            .model_provider
            .as_object()
            .cloned()
            .unwrap_or_default();
        let provider_env_key = provider_obj
            .get("env_key")
            .or_else(|| provider_obj.get("envKey"))
            .and_then(Value::as_str)
            .unwrap_or("OPENAI_API_KEY");
        let provider_base_url = provider_obj
            .get("base_url")
            .or_else(|| provider_obj.get("baseUrl"))
            .and_then(Value::as_str);

        // Build exec args
        let mut args = vec![
            "--task-repo-dir".to_string(),
            "/workspace".to_string(),
            "--model".to_string(),
            render_settings.model.clone(),
        ];
        let debug = cli_config
            .get("debug")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if debug {
            args.push("--debug".to_string());
        }

        // Build exec env
        let mut env = serde_json::Map::new();
        env.insert(provider_env_key.to_string(), json!(""));
        if let Some(base_url) = provider_base_url {
            env.insert("OPENCODE_BASE_URL".to_string(), json!(base_url));
        }
        if let Some(temp) = render_settings.temperature {
            env.insert("OPENCODE_TEMPERATURE".to_string(), json!(temp.to_string()));
        }
        if let Some(max_tokens) = render_settings.max_output_tokens {
            env.insert(
                "OPENCODE_MAX_TOKENS".to_string(),
                json!(max_tokens.to_string()),
            );
        }
        if debug {
            env.insert("OPENCODE_DEBUG".to_string(), json!("true"));
        }
        let continue_task = cli_config
            .get("continueTask")
            .or_else(|| cli_config.get("continue"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if continue_task {
            env.insert("OPENCODE_CONTINUE".to_string(), json!("true"));
        }

        // Build tools config
        let mut tools = json!({
            "filesystem": {
                "enabled": true,
                "sandbox": {
                    "root": "/workspace",
                    "allow_symlinks": false
                }
            },
            "shell": {
                "enabled": true,
                "sandbox": {
                    "working_dir": "/workspace",
                    "allowed_commands": ["git", "npm", "pnpm", "yarn", "cargo", "docker", "kubectl", "make"]
                }
            }
        });

        // Add remote tools if available
        if !render_settings.tools_url.is_empty() {
            let mut remote_config = json!({
                "enabled": true,
                "env": {
                    "TOOLS_SERVER_URL": render_settings.tools_url
                }
            });
            if !remote_tools.is_empty() {
                remote_config["availableTools"] = json!(remote_tools);
            }
            tools["remote"] = remote_config;
        }

        // Build complete config (no template needed - serialize directly)
        let config = json!({
            "mode": "exec",
            "exec": {
                "command": "opencode",
                "args": args,
                "env": Value::Object(env)
            },
            "instructions": ["AGENTS.md"],
            "tools": tools
        });

        serde_json::to_string_pretty(&config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize OpenCode config: {e}"
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

        // Register shared partials for CLI-agnostic building blocks
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke template as the cli_execute partial
        Self::register_cli_invoke_partial(&mut handlebars, CLIType::Gemini)?;

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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

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
            "task_id": code_run.spec.task_id.unwrap_or(0),
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

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

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

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
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
            "tools": {
                "tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
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
        let templates_path = get_templates_path();
        match std::fs::read_dir(&templates_path) {
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
                                                "task_id": code_run.spec.task_id.unwrap_or(0),
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

    /// Determine the job type from a CodeRun based on run_type, service, and template settings.
    /// Maps to the job subdirectory in agents/{agent}/{job}/
    fn determine_job_type(code_run: &CodeRun) -> &'static str {
        let run_type = code_run.spec.run_type.as_str();
        let service = code_run.spec.service.to_lowercase();

        // Check template setting for explicit job type
        let template_setting = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.settings.get("template"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        // Priority: explicit template setting > service name > run_type
        if template_setting.contains("heal") || service.contains("heal") {
            return "healer";
        }
        if template_setting.contains("watch") || service.contains("watch") {
            return "healer"; // Watch workflows use healer role
        }

        match run_type {
            "documentation" | "intake" => "intake",
            "quality" => "quality",
            "test" => "test",
            "deploy" => "deploy",
            "security" => "security",
            "review" => "review",
            "integration" => "integration",
            _ => "coder", // Default to coder for implementation runs
        }
    }

    /// Get the system prompt template path for an agent based on github_app and job type.
    /// Returns path in format: agents/{agent}/{job}/system-prompt.md.hbs
    fn get_agent_system_prompt_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let job_type = Self::determine_job_type(code_run);

        // Map GitHub app to agent name
        // Explicit patterns document known agents even if some share defaults
        let agent = match github_app {
            "5DLabs-Morgan" => "morgan",
            "5DLabs-Blaze" => "blaze",
            "5DLabs-Cipher" => "cipher",
            "5DLabs-Cleo" => "cleo",
            "5DLabs-Tess" => "tess",
            "5DLabs-Atlas" => "atlas",
            "5DLabs-Bolt" => "bolt",
            "5DLabs-Grizz" => "grizz",
            "5DLabs-Nova" => "nova",
            "5DLabs-Tap" => "tap",
            "5DLabs-Spark" => "spark",
            "5DLabs-Stitch" => "stitch",
            // Rex variants and unknown agents default to rex
            _ => "rex",
        };

        // Agent-specific job type defaults
        // Some agents only support specific job types
        let job = match (agent, job_type) {
            // Morgan: docs for coder, otherwise use the requested type
            ("morgan", "coder") => "docs",

            // Cleo: always quality (quality assurance specialist)
            ("cleo", _) => "quality",

            // Tess: always test (testing specialist)
            ("tess", _) => "test",

            // Atlas: always integration (integration specialist)
            ("atlas", _) => "integration",

            // Bolt: deploy by default (deployment specialist)
            ("bolt", "coder") => "deploy",

            // Cipher: security by default (security specialist)
            ("cipher", "coder") => "security",

            // Stitch: review by default (code review specialist)
            ("stitch", _) => "review",

            // All other agents/job combinations use the determined job type
            _ => job_type,
        };

        format!("agents/{agent}/{job}/system-prompt.md.hbs")
    }

    /// Select the container template for Claude CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_agent_container_template(code_run: &CodeRun) -> String {
        let run_type = code_run.spec.run_type.as_str();

        // Intake runs have a specialized container
        if run_type == "documentation" || run_type == "intake" {
            debug!("Using intake container template for run_type: {}", run_type);
            return "agents/morgan/intake/container.sh.hbs".to_string();
        }

        // All other runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Select the container template for Codex CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_codex_container_template(code_run: &CodeRun) -> String {
        let run_type = code_run.spec.run_type.as_str();

        // Intake runs have a specialized container
        if run_type == "documentation" || run_type == "intake" {
            return "agents/morgan/intake/container.sh.hbs".to_string();
        }

        // All other runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Get the memory/system-prompt template for Codex CLI.
    /// Uses the unified agent system prompt path.
    fn get_codex_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
    }

    /// Select the container template for OpenCode CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_opencode_container_template(_code_run: &CodeRun) -> String {
        // All runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Get the memory/system-prompt template for OpenCode CLI.
    /// Uses the unified agent system prompt path.
    fn get_opencode_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
    }

    /// Select the container template for Gemini CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_gemini_container_template(_code_run: &CodeRun) -> String {
        // All runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Select the container template for Cursor CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_cursor_container_template(code_run: &CodeRun) -> String {
        let run_type = code_run.spec.run_type.as_str();

        // Intake runs have a specialized container
        if run_type == "documentation" || run_type == "intake" {
            return "agents/morgan/intake/container.sh.hbs".to_string();
        }

        // All other runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Get the memory/system-prompt template for Cursor CLI.
    /// Uses the unified agent system prompt path.
    fn get_cursor_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
    }

    /// Get the memory/system-prompt template for Gemini CLI.
    /// Uses the unified agent system prompt path.
    fn get_gemini_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
    }

    /// Get the memory/system-prompt template for Claude CLI.
    /// Uses the unified agent system prompt path.
    fn get_claude_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
    }

    /// Select the container template for Factory CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_factory_container_template(_code_run: &CodeRun) -> String {
        // All runs use the shared container template
        "_shared/container.sh.hbs".to_string()
    }

    /// Get the memory/system-prompt template for Factory CLI.
    /// Uses the unified agent system prompt path.
    fn get_factory_memory_template(code_run: &CodeRun) -> String {
        Self::get_agent_system_prompt_template(code_run)
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

    /// Load a template file from the templates directory.
    ///
    /// Supports two modes:
    /// - **ConfigMap mode** (production): Files are flattened with `_` separators (e.g., `_shared_container.sh.hbs`)
    /// - **Directory mode** (testing): Files use original path structure (e.g., `_shared/container.sh.hbs`)
    fn load_template(relative_path: &str) -> Result<String> {
        let templates_path = get_templates_path();

        // First, try the original path structure (for local development/testing)
        let direct_path = Path::new(&templates_path).join(relative_path);
        if direct_path.exists() {
            debug!(
                "Loading code template from: {} (direct path)",
                direct_path.display()
            );
            return fs::read_to_string(&direct_path).map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to load code template {relative_path}: {e}"
                ))
            });
        }

        // Fall back to ConfigMap key format (path separators converted to underscores)
        let configmap_key = relative_path.replace('/', "_");
        let configmap_path = Path::new(&templates_path).join(&configmap_key);
        debug!(
            "Loading code template from: {} (configmap key: {})",
            configmap_path.display(),
            configmap_key
        );

        fs::read_to_string(&configmap_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load code template {relative_path} (tried: {}, {}): {e}",
                direct_path.display(),
                configmap_path.display()
            ))
        })
    }

    /// Extract repo slug (owner/repo) from a GitHub repository URL
    ///
    /// Handles various formats:
    /// - `https://github.com/owner/repo.git` -> `owner/repo`
    /// - `https://github.com/owner/repo` -> `owner/repo`
    /// - Falls back to the original URL if no prefix match
    fn extract_repo_slug(repository_url: &str) -> String {
        repository_url
            .strip_prefix("https://github.com/")
            .and_then(|s| s.strip_suffix(".git"))
            .or_else(|| repository_url.strip_prefix("https://github.com/"))
            .unwrap_or(repository_url)
            .to_string()
    }

    /// Register shared function and bootstrap partials
    /// These partials provide CLI-agnostic building blocks for container scripts
    fn register_shared_partials(handlebars: &mut Handlebars) -> Result<()> {
        use crate::tasks::template_paths::{
            // New templates partials
            PARTIAL_ACCEPTANCE_PROBE,
            PARTIAL_COMPLETION,
            PARTIAL_CONFIG,
            PARTIAL_EXPO_ENV,
            PARTIAL_FRONTEND_TOOLKITS,
            PARTIAL_GITHUB_AUTH,
            PARTIAL_GIT_SETUP,
            PARTIAL_GO_ENV,
            PARTIAL_HEADER,
            PARTIAL_NODE_ENV,
            PARTIAL_RETRY_LOOP,
            PARTIAL_RUST_ENV,
            PARTIAL_SHADCN_STACK,
            PARTIAL_TANSTACK_STACK,
            PARTIAL_TASK_FILES,
            PARTIAL_TOOLS_CONFIG,
            // Legacy partials
            SHARED_BOOTSTRAP_RUST_ENV,
            SHARED_CONTAINER_CORE,
            SHARED_FUNCTIONS_COMPLETION_MARKER,
            SHARED_FUNCTIONS_GITHUB_AUTH,
            SHARED_FUNCTIONS_GIT_OPERATIONS,
            SHARED_FUNCTIONS_QUALITY_GATES,
            SHARED_PROMPTS_CONTEXT7,
            SHARED_PROMPTS_DESIGN_SYSTEM,
        };

        // Map partial name (used in templates) -> template path
        // New templates partials (used by _shared/container.sh.hbs)
        let new_partials = vec![
            ("header", PARTIAL_HEADER),
            ("rust-env", PARTIAL_RUST_ENV),
            ("go-env", PARTIAL_GO_ENV),
            ("node-env", PARTIAL_NODE_ENV),
            ("expo-env", PARTIAL_EXPO_ENV),
            ("config", PARTIAL_CONFIG),
            ("github-auth", PARTIAL_GITHUB_AUTH),
            ("git-setup", PARTIAL_GIT_SETUP),
            ("task-files", PARTIAL_TASK_FILES),
            ("tools-config", PARTIAL_TOOLS_CONFIG),
            ("acceptance-probe", PARTIAL_ACCEPTANCE_PROBE),
            ("retry-loop", PARTIAL_RETRY_LOOP),
            ("completion", PARTIAL_COMPLETION),
            // Frontend stack partials (for Blaze/Morgan)
            ("frontend-toolkits", PARTIAL_FRONTEND_TOOLKITS),
            ("tanstack-stack", PARTIAL_TANSTACK_STACK),
            ("shadcn-stack", PARTIAL_SHADCN_STACK),
        ];

        // Legacy partials (for backwards compatibility)
        // Note: docker-sidecar and gh-cli removed - no longer exist as separate partials
        let legacy_partials = vec![
            ("shared/bootstrap/rust-env", SHARED_BOOTSTRAP_RUST_ENV),
            ("shared/functions/github-auth", SHARED_FUNCTIONS_GITHUB_AUTH),
            (
                "shared/functions/completion-marker",
                SHARED_FUNCTIONS_COMPLETION_MARKER,
            ),
            (
                "shared/functions/git-operations",
                SHARED_FUNCTIONS_GIT_OPERATIONS,
            ),
            (
                "shared/functions/quality-gates",
                SHARED_FUNCTIONS_QUALITY_GATES,
            ),
            ("shared/context7-instructions", SHARED_PROMPTS_CONTEXT7),
            ("shared/design-system", SHARED_PROMPTS_DESIGN_SYSTEM),
            ("shared/container-core", SHARED_CONTAINER_CORE),
        ];

        // Combine both sets
        let shared_partials: Vec<(&str, &str)> =
            new_partials.into_iter().chain(legacy_partials).collect();

        let mut failed_partials = Vec::new();

        for (partial_name, template_path) in shared_partials {
            match Self::load_template(template_path) {
                Ok(content) => {
                    handlebars
                        .register_partial(partial_name, content)
                        .map_err(|e| {
                            crate::tasks::types::Error::ConfigError(format!(
                                "Failed to register shared partial {partial_name}: {e}"
                            ))
                        })?;
                    debug!("Successfully registered shared partial: {}", partial_name);
                }
                Err(e) => {
                    // Warn but don't fail - the partial may not be needed for all templates
                    warn!(
                        "Failed to load shared partial {partial_name} from ConfigMap (path: {template_path}): {e}. \
                        Templates referencing this partial will fail to render."
                    );
                    failed_partials.push(partial_name);
                }
            }
        }

        if !failed_partials.is_empty() {
            warn!(
                "Shared partial registration incomplete. {} partials failed to load: {:?}. \
                Ensure the templates ConfigMaps are properly mounted at {}",
                failed_partials.len(),
                failed_partials,
                get_templates_path()
            );
        }

        Ok(())
    }

    /// Register shared agent system prompt partials
    /// These partials are used by agent-specific templates via {{> agents/partial-name}}
    fn register_agent_partials(handlebars: &mut Handlebars) -> Result<()> {
        use crate::tasks::template_paths::{
            PARTIAL_FRONTEND_TOOLKITS, PARTIAL_SHADCN_STACK, PARTIAL_TANSTACK_STACK,
        };

        // List of shared agent system prompt partials that need to be registered
        let agent_partials = vec![
            "agents/cipher-system-prompt",
            "agents/cleo-system-prompt",
            "agents/rex-system-prompt",
            "agents/tess-system-prompt",
            "agents/system-prompt",
        ];

        // Frontend stack partials used by Blaze and Morgan system prompts
        let frontend_stack_partials = vec![
            ("frontend-toolkits", PARTIAL_FRONTEND_TOOLKITS),
            ("tanstack-stack", PARTIAL_TANSTACK_STACK),
            ("shadcn-stack", PARTIAL_SHADCN_STACK),
        ];

        // Register frontend stack partials first
        for (partial_name, template_path) in frontend_stack_partials {
            match Self::load_template(template_path) {
                Ok(content) => {
                    handlebars
                        .register_partial(partial_name, content)
                        .map_err(|e| {
                            crate::tasks::types::Error::ConfigError(format!(
                                "Failed to register frontend stack partial {partial_name}: {e}"
                            ))
                        })?;
                    debug!(
                        "Successfully registered frontend stack partial: {}",
                        partial_name
                    );
                }
                Err(e) => {
                    // Warn but don't fail - the partial may not be needed for non-frontend agents
                    warn!(
                        "Failed to load frontend stack partial {partial_name} from ConfigMap (path: {template_path}): {e}. \
                        Blaze/Morgan templates referencing this partial will fail to render."
                    );
                }
            }
        }

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
                Ensure the templates ConfigMaps are properly mounted at {}",
                failed_partials.len(),
                failed_partials,
                get_templates_path()
            );
        }

        Ok(())
    }

    /// Register CLI-specific invoke template as the `cli_execute` partial.
    /// This allows the container.sh.hbs template to use {{> cli_execute}} to include
    /// the CLI-specific invocation logic.
    fn register_cli_invoke_partial(handlebars: &mut Handlebars, cli_type: CLIType) -> Result<()> {
        let cli_name = match cli_type {
            CLIType::Codex => "codex",
            CLIType::Cursor => "cursor",
            CLIType::Factory => "factory",
            CLIType::Gemini => "gemini",
            CLIType::OpenCode => "opencode",
            // Claude and types without dedicated invoke templates fall back to claude
            CLIType::Claude | CLIType::OpenHands | CLIType::Grok | CLIType::Qwen => "claude",
        };

        let invoke_template_path = format!("clis/{cli_name}/invoke.sh.hbs");

        match Self::load_template(&invoke_template_path) {
            Ok(content) => {
                handlebars
                    .register_partial("cli_execute", content)
                    .map_err(|e| {
                        crate::tasks::types::Error::ConfigError(format!(
                            "Failed to register CLI invoke partial for {cli_name}: {e}"
                        ))
                    })?;
                debug!(
                    "Successfully registered CLI invoke partial for {}",
                    cli_name
                );
                Ok(())
            }
            Err(e) => {
                // If the CLI-specific invoke template is not found, fall back to a simple echo
                warn!(
                    "CLI invoke template not found for {cli_name} (path: {invoke_template_path}): {e}. \
                    Registering a fallback placeholder."
                );

                // Register a fallback that just echoes the CLI type
                let fallback = format!(
                    r#"echo "⚠️ No invoke template found for {cli_name} CLI"
echo "CLI invocation should be handled by the adapter."
"#
                );
                handlebars
                    .register_partial("cli_execute", fallback)
                    .map_err(|e| {
                        crate::tasks::types::Error::ConfigError(format!(
                            "Failed to register fallback CLI invoke partial: {e}"
                        ))
                    })?;
                Ok(())
            }
        }
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
                run_type: "implementation".to_string(),
                cli_config: None,
                task_id: Some(1),
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
                linear_integration: None,
            },
            status: None,
        }
    }

    // ========================================================================
    // Container template selection tests
    // All agents now use the shared container template (_shared/container.sh.hbs)
    // CLI-specific behavior is injected via the cli_execute partial
    // ========================================================================

    #[test]
    fn test_container_template_uses_shared_for_all_agents() {
        // All agents should use the shared container template
        for github_app in [
            "5DLabs-Rex",
            "5DLabs-Cleo",
            "5DLabs-Tess",
            "5DLabs-Cipher",
            "5DLabs-Atlas",
            "5DLabs-Bolt",
            "5DLabs-Blaze",
        ] {
            let code_run = create_test_code_run(Some(github_app.to_string()));
            let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
            assert_eq!(
                template_path, "_shared/container.sh.hbs",
                "Agent {github_app} should use shared container template"
            );
        }
    }

    #[test]
    fn test_container_template_default_uses_shared() {
        let code_run = create_test_code_run(None);
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "_shared/container.sh.hbs");
    }

    #[test]
    fn test_container_template_intake_uses_morgan() {
        // Intake runs should use Morgan's intake container
        let mut code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        code_run.spec.run_type = "intake".to_string();
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(
            template_path, "agents/morgan/intake/container.sh.hbs",
            "Intake run should use Morgan intake container"
        );
    }

    #[test]
    fn test_container_template_documentation_uses_morgan() {
        // Documentation runs should also use Morgan's intake container
        let mut code_run = create_test_code_run(Some("5DLabs-Morgan".to_string()));
        code_run.spec.run_type = "documentation".to_string();
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(
            template_path, "agents/morgan/intake/container.sh.hbs",
            "Documentation run should use Morgan intake container"
        );
    }

    // ========================================================================
    // System prompt template selection tests
    // All agents use agents/{agent}/{job}/system-prompt.md.hbs
    // ========================================================================

    #[test]
    fn test_system_prompt_template_rex_coder() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/rex/coder/system-prompt.md.hbs");
    }

    #[test]
    fn test_system_prompt_template_cleo_quality() {
        // Cleo always uses quality job type (quality assurance specialist)
        let code_run = create_test_code_run(Some("5DLabs-Cleo".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/cleo/quality/system-prompt.md.hbs");
    }

    #[test]
    fn test_system_prompt_template_healer_service() {
        use crate::cli::types::CLIType;
        use crate::crds::coderun::CLIConfig;
        use serde_json::json;

        let mut code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let mut settings = HashMap::new();
        settings.insert("template".to_string(), json!("heal/claude"));
        code_run.spec.cli_config = Some(CLIConfig {
            cli_type: CLIType::Claude,
            model: "claude-opus-4-5-20251101".to_string(),
            settings,
            max_tokens: None,
            temperature: None,
            model_rotation: None,
        });
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path, "agents/rex/healer/system-prompt.md.hbs",
            "Heal template setting should map to healer job"
        );
    }

    #[test]
    fn test_system_prompt_template_morgan_intake() {
        let mut code_run = create_test_code_run(Some("5DLabs-Morgan".to_string()));
        code_run.spec.run_type = "intake".to_string();
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path, "agents/morgan/intake/system-prompt.md.hbs",
            "Morgan intake run should use intake prompt"
        );
    }

    #[test]
    fn test_system_prompt_template_atlas_integration() {
        let mut code_run = create_test_code_run(Some("5DLabs-Atlas".to_string()));
        code_run.spec.run_type = "integration".to_string();
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path,
            "agents/atlas/integration/system-prompt.md.hbs"
        );
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
                frontend_stack: None,
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
                frontend_stack: None,
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
    fn test_cursor_container_template_uses_shared() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_cursor_container_template(&code_run);
        assert_eq!(template_path, "_shared/container.sh.hbs");
    }

    #[test]
    fn test_cursor_memory_template_selection() {
        // Rex doing default (coder) work
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_cursor_memory_template(&code_run);
        assert_eq!(template_path, "agents/rex/coder/system-prompt.md.hbs");
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
                frontend_stack: None,
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
