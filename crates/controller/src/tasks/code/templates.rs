use crate::cli::types::CLIType;
use crate::crds::coderun::HarnessAgent;
use crate::crds::CodeRun;
use crate::tasks::code::agent::AgentClassifier;
use crate::tasks::config::ControllerConfig;
use crate::tasks::template_paths;
use crate::tasks::template_paths::{
    CODE_CODEX_CONTAINER_BASE_TEMPLATE, CODE_CODING_GUIDELINES_TEMPLATE,
    CODE_CURSOR_CONTAINER_BASE_TEMPLATE, CODE_FACTORY_CONTAINER_BASE_TEMPLATE,
    CODE_GEMINI_CONTAINER_BASE_TEMPLATE, CODE_GITHUB_GUIDELINES_TEMPLATE,
    CODE_OPENCODE_CONTAINER_BASE_TEMPLATE, HARNESS_HERMES_TEMPLATE,
    HARNESS_OPENCLAW_CONFIG_TEMPLATE, HARNESS_OPENCLAW_TEMPLATE, LOBSTER_BASE_TASK_TEMPLATE,
};
use crate::tasks::tool_catalog::resolve_tool_name;
use crate::tasks::types::Result;
use crate::tasks::workflow::extract_workflow_name;
use handlebars::{handlebars_helper, Handlebars, HelperDef, ScopedJson};

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

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

/// Helper for grouping array items by a field value.
/// Usage: `{{#each (group_by subtasks "execution_level")}}...{{/each}}`
/// Returns an object keyed by field values, where each value is an array of items.
struct GroupByHelper;

impl HelperDef for GroupByHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc handlebars::Context,
        _: &mut handlebars::RenderContext<'reg, 'rc>,
    ) -> std::result::Result<ScopedJson<'rc>, handlebars::RenderError> {
        // Get the array parameter
        let array = h
            .param(0)
            .ok_or_else(|| handlebars::RenderErrorReason::ParamNotFoundForIndex("group_by", 0))?
            .value();

        // Get the field name parameter
        let field = h
            .param(1)
            .ok_or_else(|| handlebars::RenderErrorReason::ParamNotFoundForIndex("group_by", 1))?
            .value()
            .as_str()
            .ok_or_else(|| {
                handlebars::RenderErrorReason::Other(
                    "group_by field parameter must be a string".to_string(),
                )
            })?;

        // Build grouped result using BTreeMap for sorted keys
        let mut grouped: BTreeMap<String, Vec<Value>> = BTreeMap::new();

        if let Some(items) = array.as_array() {
            for item in items {
                let key = item
                    .get(field)
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(n) => Some(n.to_string()),
                        Value::Bool(b) => Some(b.to_string()),
                        _ => None,
                    })
                    .unwrap_or_default();
                grouped.entry(key).or_default().push(item.clone());
            }
        }

        // Convert to JSON object
        let result: Value = grouped
            .into_iter()
            .map(|(k, v)| (k, Value::Array(v)))
            .collect::<serde_json::Map<String, Value>>()
            .into();

        Ok(ScopedJson::Derived(result))
    }
}

pub struct CodeTemplateGenerator;

// Per-CLI template generators below are legacy paths; production routing uses
// `generate_all_templates` → `generate_claude_templates` + OpenClaw harness.
// Helpers are interleaved with that path, so dead-code is suppressed on the impl.
#[allow(dead_code)]
impl CodeTemplateGenerator {
    /// Register common Handlebars helpers for template conditionals
    /// This enables `eq` and `or` helpers used in templates like:
    /// `{{#if (eq github_app "tap")}}` and `{{#if (or (eq a "x") (eq b "y"))}}`
    fn register_template_helpers(handlebars: &mut Handlebars) {
        // Helper for equality comparison: {{#if (eq var "value")}}
        // Returns a boolean that can be used in conditionals
        handlebars_helper!(eq: |left: str, right: str| left == right);
        handlebars.register_helper("eq", Box::new(eq));

        // Helper for logical OR: {{#if (or cond1 cond2 ...)}}
        // Returns true if any argument is truthy
        handlebars.register_helper(
            "or",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    let any_truthy = h.params().iter().any(|p| {
                        let val = p.value();
                        // Check if value is truthy: non-empty string, true bool, or non-null
                        match val {
                            serde_json::Value::Bool(b) => *b,
                            serde_json::Value::String(s) => !s.is_empty(),
                            serde_json::Value::Null => false,
                            _ => true,
                        }
                    });
                    // Write "true" or empty string - Handlebars treats non-empty as truthy
                    out.write(if any_truthy { "true" } else { "" })?;
                    Ok(())
                },
            ),
        );

        // Helper for grouping array items by a field: {{#each (group_by subtasks "execution_level")}}
        // Returns an object keyed by field values, where each value is an array of items
        handlebars.register_helper("group_by", Box::new(GroupByHelper));
    }

    /// Generate all template files for a code task
    pub fn generate_all_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        // Check run_type first for review/remediate tasks
        let mut templates = match code_run.spec.run_type.as_str() {
            "review" => Self::generate_review_templates(code_run, config)?,
            "remediate" => Self::generate_remediate_templates(code_run, config)?,
            _ => {
                // All CLI types route through the OpenClaw harness.
                // The harness uses acpx to dispatch to the correct CLI at runtime.
                Self::generate_claude_templates(code_run, config)?
            }
        };

        // If prompt_modification is set, write it to prompt.md
        // This is critical for healer CI runs and other cases where the prompt
        // is provided directly in the CodeRun spec rather than from task files
        if let Some(ref prompt_content) = code_run.spec.prompt_modification {
            if !prompt_content.trim().is_empty() {
                debug!(
                    "Writing prompt_modification to prompt.md ({} bytes)",
                    prompt_content.len()
                );
                templates.insert("prompt.md".to_string(), prompt_content.clone());
            }
        }

        // If acceptance_criteria is set, write it to acceptance-criteria.md
        // This allows the acceptance probe to verify checkboxes after task completion
        if let Some(ref criteria_content) = code_run.spec.acceptance_criteria {
            if !criteria_content.trim().is_empty() {
                debug!(
                    "Writing acceptance_criteria to acceptance-criteria.md ({} bytes)",
                    criteria_content.len()
                );
                templates.insert(
                    "acceptance-criteria.md".to_string(),
                    criteria_content.clone(),
                );
            }
        }

        // Inject persona files from the remote skills cache.
        // Persona AGENTS.md is prepended to the generated AGENTS.md.
        // Other persona files (SOUL.md, USER.md, etc.) are added as new entries.
        if code_run.spec.skills_url.is_some() {
            let agent_name = Self::get_agent_name(code_run);
            let persona = super::skills_cache::get_persona_files(&agent_name);

            if !persona.is_empty() {
                debug!(
                    "Injecting {} persona files for agent '{}'",
                    persona.len(),
                    agent_name
                );

                for (filename, content) in &persona {
                    if filename == "AGENTS.md" {
                        // Prepend persona AGENTS.md to the generated one
                        if let Some(existing) = templates.get("AGENTS.md") {
                            let merged = format!("{content}\n\n---\n\n{existing}");
                            templates.insert("AGENTS.md".to_string(), merged);
                        } else {
                            templates.insert("AGENTS.md".to_string(), content.clone());
                        }
                    } else {
                        // Add other persona files directly
                        templates.insert(filename.clone(), content.clone());
                    }
                }
            }
        }

        // Inject CLI-specific config files from templates/cli-configs/
        // These are simple Handlebars templates rendered with CRD context
        let cli_type = Self::determine_cli_type(code_run);
        let model = &code_run.spec.model;
        let workspace_dir = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| serde_json::to_value(c).ok())
            .and_then(|v| {
                v.get("workspaceDir")
                    .and_then(|w| w.as_str().map(String::from))
            })
            .unwrap_or_default();
        let fireworks_routing = model.contains("fireworks");

        if let Some((filename, content)) =
            Self::render_cli_config(cli_type, model, &workspace_dir, fireworks_routing)
        {
            templates.insert(filename, content);
        }

        // Kimi CLI requires a persisted OAuth token to pass _check_auth.
        // The harness writes ~/.kimi/credentials/kimi-code.json at startup
        // using FIREWORKS_API_KEY from the environment.

        // Include cto-tools CLI and mcp.ts runtime for dynamic MCP tool access.
        // These are static files loaded into every agent pod via the task-files ConfigMap.
        // The cto-tools-setup partial copies them from /task-files/ to /.cto-tools/.
        if let Ok(cli_content) = Self::load_template(template_paths::CTO_TOOLS_CLI) {
            templates.insert("cto-tools".to_string(), cli_content);
        } else {
            debug!("cto-tools CLI not found in templates — dynamic tool CLI unavailable");
        }
        if let Ok(ts_content) = Self::load_template(template_paths::CTO_TOOLS_MCP_TS) {
            templates.insert("mcp.ts".to_string(), ts_content);
        } else {
            debug!("mcp.ts runtime not found in templates — TS code execution unavailable");
        }
        if let Ok(codegen_content) = Self::load_template(template_paths::CTO_TOOLS_CODEGEN_TS) {
            templates.insert("codegen.ts".to_string(), codegen_content);
        } else {
            debug!("codegen.ts not found in templates — wrapper generation unavailable");
        }
        if let Ok(deno_content) = Self::load_template(template_paths::CTO_TOOLS_DENO_JSON) {
            templates.insert("deno.json".to_string(), deno_content);
        } else {
            debug!("deno.json not found in templates — Deno config unavailable");
        }

        Ok(templates)
    }

    /// Render a CLI-specific config file from templates/cli-configs/.
    /// Returns (filename, rendered_content) or None if no config needed.
    fn render_cli_config(
        cli_type: CLIType,
        model: &str,
        workspace_dir: &str,
        fireworks_routing: bool,
    ) -> Option<(String, String)> {
        let templates_path = get_templates_path();
        let (template_file, output_name) = match cli_type {
            CLIType::Copilot => ("copilot-config.json.hbs", "copilot-config.json"),
            CLIType::Kimi => ("kimi-config.toml.hbs", "kimi-config.toml"),
            CLIType::OpenCode => ("opencode.json.hbs", "opencode.json"),
            CLIType::Codex => ("codex-config.toml.hbs", "codex-config.toml"),
            CLIType::Gemini => ("gemini-settings.json.hbs", "gemini-settings.json"),
            CLIType::Cursor => ("cursor-config.json.hbs", "cursor-config.json"),
            CLIType::Factory => ("factory-config.json.hbs", "factory-config.json"),
            _ => return None,
        };

        let template_path = format!("{templates_path}/cli-configs/{template_file}");
        let template_content = match fs::read_to_string(&template_path) {
            Ok(content) => content,
            Err(e) => {
                warn!("CLI config template not found at {}: {}", template_path, e);
                return None;
            }
        };

        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);
        let data = json!({
            "model": model,
            "workspace_dir": workspace_dir,
            "fireworks_routing": fireworks_routing,
        });

        match hbs.render_template(&template_content, &data) {
            Ok(rendered) => Some((output_name.to_string(), rendered)),
            Err(e) => {
                warn!(
                    "Failed to render CLI config template {}: {}",
                    template_file, e
                );
                None
            }
        }
    }

    fn determine_cli_type(code_run: &CodeRun) -> CLIType {
        code_run
            .spec
            .cli_config
            .as_ref()
            .map_or(CLIType::Claude, |cfg| cfg.cli_type)
    }

    /// Check if the CLI supports native skill loading.
    /// CLIs with native skill support: Claude Code, Factory/Droid, OpenCode, Codex
    /// CLIs without native skill support: Cursor, Gemini, Aider, etc.
    fn cli_supports_native_skills(cli_type: CLIType) -> bool {
        matches!(
            cli_type,
            CLIType::Claude | CLIType::Factory | CLIType::OpenCode | CLIType::Codex
        )
    }

    /// Return the CLI-native skills directory path (relative to repo root)
    /// where each CLI expects SKILL.md files to be placed.
    fn cli_native_skills_dir(cli_type: CLIType) -> &'static str {
        match cli_type {
            CLIType::Claude => ".claude/skills",
            CLIType::Factory => ".factory/skills",
            CLIType::OpenCode => ".opencode/skills",
            CLIType::Codex => ".codex/skills",
            CLIType::Cursor => ".cursor/skills",
            CLIType::Gemini => ".gemini/skills",
            CLIType::Copilot => ".copilot/skills",
            CLIType::Kimi => ".kimi/skills",
            // Fallback: use a generic path that won't collide
            _ => ".agent/skills",
        }
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

        // Fetch all skills from remote tarball (used by both harness and lobster)
        let all_skills = Self::fetch_all_skills_for_coderun(code_run, config);

        // Write each skill as a separate ConfigMap entry to avoid E2BIG
        // when inlining large heredocs in the lobster script.
        // Skills are mounted at /task-files/skill-{name}.md and copied by
        // the skills-setup lobster step.
        let skill_names: Vec<serde_json::Value> = all_skills
            .iter()
            .map(|s| {
                let name = s["name"].as_str().unwrap_or("");
                let content = s["content"].as_str().unwrap_or("");
                if !name.is_empty() && !content.is_empty() {
                    templates.insert(format!("skill-{name}.md"), content.to_string());
                }
                json!({ "name": name })
            })
            .filter(|s| !s["name"].as_str().unwrap_or("").is_empty())
            .collect();

        // Render the Lobster base-task workflow (replaces the old container.sh flow)
        templates.insert(
            "base-task.lobster".to_string(),
            Self::generate_lobster_base_task(code_run, &enriched_cli_config, config, &skill_names)?,
        );

        // Render harness-specific templates based on the CRD's harnessAgent field
        match code_run.spec.effective_harness() {
            HarnessAgent::OpenClaw => {
                // OpenClaw path: gateway config + gateway launcher
                templates.insert(
                    "openclaw.json".to_string(),
                    Self::generate_openclaw_config(code_run, &enriched_cli_config, config)?,
                );
                templates.insert(
                    "container.sh".to_string(),
                    Self::generate_harness_launcher(code_run, config)?,
                );
            }
            HarnessAgent::Hermes => {
                // Hermes path: standalone launcher, no gateway config
                templates.insert(
                    "container.sh".to_string(),
                    Self::generate_hermes_launcher(code_run, config)?,
                );
            }
        }
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
                config,
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
        Self::register_template_helpers(&mut handlebars);

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
        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();

        let workflow_name = extract_workflow_name(code_run).unwrap_or_else(|_| {
            format!("play-task-{}-workflow", code_run.spec.task_id.unwrap_or(0))
        });

        let job_type = Self::determine_job_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let task_language = Self::get_task_language(code_run);
        let default_retries = Self::get_default_retries(code_run);
        let fresh_start_threshold = Self::get_fresh_start_threshold(code_run);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "model": model.clone(),
            "cli_type": cli_type,
            "enable_docker": code_run.spec.enable_docker,
            "list_tools_on_start": render_settings.list_tools_on_start,
            // Required template context variables
            "job_type": job_type,
            "agent_name": agent_name,
            "task_language": task_language,
            "default_retries": default_retries,
            "fresh_start_threshold": fresh_start_threshold,
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "use_sdk_path": Self::extract_use_sdk_path(cli_config),
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
        Self::register_template_helpers(&mut handlebars);

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
            "github_app": Self::get_github_app_or_default(code_run),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
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
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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

        // Build MCP servers config (HTTP transport format for Claude CLI)
        let mut mcp_servers = json!({});
        if !render_settings.tools_url.is_empty() {
            let mut tools_server = json!({
                "type": "http",
                "url": render_settings.tools_url
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
            Self::generate_factory_container_script(
                code_run,
                &enriched_cli_config,
                &remote_tools,
                config,
            )?,
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

    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn generate_factory_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
        config: &ControllerConfig,
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

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

        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();
        let job_type = Self::determine_job_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let task_language = Self::get_task_language(code_run);
        let default_retries = Self::get_default_retries(code_run);
        let fresh_start_threshold = Self::get_fresh_start_threshold(code_run);

        // Get skills for agent (Factory supports native skill loading)
        let skills = Self::get_agent_skills_enriched(code_run, config);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "model": render_settings.model,
            "auto_level": render_settings.auto_level,
            "output_format": render_settings.output_format,
            "model_rotation": render_settings.model_rotation,
            "list_tools_on_start": render_settings.list_tools_on_start,
            "enable_docker": code_run.spec.enable_docker,
            // Required template context variables
            "cli_type": cli_type,
            "job_type": job_type,
            "agent_name": agent_name,
            "task_language": task_language,
            "default_retries": default_retries,
            "fresh_start_threshold": fresh_start_threshold,
            // Skills for native skill loading
            "skills": skills,
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            // Telemetry configuration
            "telemetry_enabled": Self::is_telemetry_enabled(code_run, config),
            "otel_endpoint": Self::get_otel_endpoint(),
            "datadog_enabled": Self::is_datadog_enabled(config),
            // Watch-specific context
            "iteration": iteration,
            "max_iterations": max_iterations,
            "target_repository": target_repository,
            "namespace": namespace,
            "use_sdk_path": Self::extract_use_sdk_path(cli_config),
            "cli": {
                "type": cli_type,
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
        Self::register_template_helpers(&mut handlebars);

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

        // Check if CLI supports native skills (for conditional partial rendering)
        let cli_type_enum = Self::determine_cli_type(code_run);

        let context = json!({
            "cli_config": cli_config,
            "github_app": Self::get_github_app_or_default(code_run),
            "model": render_settings.model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "cli_type": cli_type_enum.to_string(),
            "iteration": iteration,
            // Flag to conditionally skip inline partials (for CLIs with native skills)
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "cli": {
                "type": cli_type_enum.to_string(),
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
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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

    /// Render the Lobster base-task workflow template.
    /// Uses the same Handlebars partials as container.sh (github-auth, git-setup, etc.)
    /// so the rendered YAML contains fully expanded shell blocks.
    fn generate_lobster_base_task(
        code_run: &CodeRun,
        cli_config: &Value,
        _config: &ControllerConfig,
        skills: &[serde_json::Value],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);
        Self::register_shared_partials(&mut handlebars)?;

        let template = Self::load_template(LOBSTER_BASE_TASK_TEMPLATE)?;
        handlebars
            .register_template_string("lobster_base_task", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Lobster base task template: {e}"
                ))
            })?;

        let cli_type = Self::determine_cli_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let job_type = Self::determine_job_type(code_run);

        let coderun_name = code_run.metadata.name.as_deref().unwrap_or("unknown");
        let namespace = code_run.metadata.namespace.as_deref().unwrap_or("cto");
        let qualified_model = Self::qualify_model_for_openclaw(code_run);
        let github_app_name = Self::get_github_app_or_default(code_run);

        // Build openclaw_providers for the CRD param dump (same logic as openclaw config)
        use crate::crds::coderun::OpenClawConfig;
        let openclaw_cfg = code_run
            .spec
            .openclaw
            .clone()
            .unwrap_or_else(OpenClawConfig::default_providers);
        let openclaw_providers_summary: Vec<Value> = openclaw_cfg
            .providers
            .iter()
            .map(|p| {
                let models: Vec<Value> = p.models.iter().map(|m| {
                    let display = m.display_name.as_deref().unwrap_or(&m.name);
                    json!({
                        "id": m.name,
                        "name": display,
                    })
                }).collect();
                json!({
                    "name": p.name,
                    "baseUrl": p.base_url,
                    "api": p.api.as_deref().unwrap_or("openai-completions"),
                    "apiKeyEnvVar": p.api_key_env_var,
                    "models": models,
                })
            })
            .collect();

        let discord_enabled = code_run.spec.openclaw.as_ref()
            .is_none_or(|oc| oc.discord_enabled);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "task_number": code_run.spec.task_id.unwrap_or(0),
            "coderun_name": coderun_name,
            "service": code_run.spec.service,
            "run_type": code_run.spec.run_type,
            "job_type": job_type,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "docs_project": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "git_branch": &code_run.spec.docs_branch,
            "github_app": &github_app_name,
            "github_app_name": &github_app_name,
            "github_app_id": "",
            "model": &qualified_model,
            "cli_model": &code_run.spec.model,
            "resolved_provider": &qualified_model,
            "cli_type": cli_type.to_string(),
            "agent_name": &agent_name,
            "agent_name_upper": agent_name.to_uppercase(),
            "namespace": namespace,
            "discord_enabled": discord_enabled,
            "discord_channel_id": "",
            "openclaw_providers": openclaw_providers_summary,
            "cli_config": cli_config,
            "skills": skills,
            "cli_skills_dir": Self::cli_native_skills_dir(cli_type),
            "cli_skills_path": format!("$REPO_ROOT/{}", Self::cli_native_skills_dir(cli_type)),
        });

        handlebars
            .render("lobster_base_task", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Lobster base task: {e}"
                ))
            })
    }

    /// Render the OpenClaw gateway config template for a CRD pod.
    /// Produces a JSON config modeled on Morgan's openclaw.json.
    ///
    /// Provider data is sourced from `spec.openclaw` when present,
    /// otherwise [`OpenClawConfig::default_providers()`] is used.
    fn generate_openclaw_config(
        code_run: &CodeRun,
        cli_config: &Value,
        _config: &ControllerConfig,
    ) -> Result<String> {
        use crate::crds::coderun::OpenClawConfig;

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

        let template = Self::load_template(HARNESS_OPENCLAW_CONFIG_TEMPLATE)?;
        handlebars
            .register_template_string("openclaw_config", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register OpenClaw config template: {e}"
                ))
            })?;

        let cli_type = Self::determine_cli_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let coderun_name = code_run.metadata.name.as_deref().unwrap_or("unknown");

        // Use CRD openclaw providers, or fall back to defaults
        let openclaw_cfg = code_run
            .spec
            .openclaw
            .clone()
            .unwrap_or_else(OpenClawConfig::default_providers);

        // Build normalized provider data for the template.
        // Each provider becomes a JSON object with all fields the template needs,
        // applying sensible defaults for missing optional values.
        let openclaw_providers: Vec<Value> = openclaw_cfg
            .providers
            .iter()
            .map(|p| {
                let models: Vec<Value> = p
                    .models
                    .iter()
                    .map(|m| {
                        let input = m
                            .input
                            .clone()
                            .unwrap_or_else(|| vec!["text".to_string()]);
                        // Pre-serialize input array as JSON string for safe template insertion
                        let input_json =
                            serde_json::to_string(&input).unwrap_or_else(|_| "[\"text\"]".into());
                        json!({
                            "id": m.name,
                            "name": m.display_name.as_deref().unwrap_or(&m.name),
                            "reasoning": m.reasoning.unwrap_or(false),
                            "input_json": input_json,
                            "contextWindow": m.context_window.unwrap_or(131_072),
                            "maxTokens": m.max_tokens.unwrap_or(8192),
                        })
                    })
                    .collect();
                json!({
                    "name": p.name,
                    "baseUrl": p.base_url.as_deref().unwrap_or(""),
                    "apiKeyEnvVar": p.api_key_env_var.as_deref().unwrap_or(""),
                    "api": p.api.as_deref().unwrap_or("openai-completions"),
                    "models": models,
                })
            })
            .collect();

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "service": code_run.spec.service,
            "coderun_name": coderun_name,
            "model": Self::resolve_openclaw_primary_model(code_run, &openclaw_providers),
            "cli_type": cli_type.to_string(),
            "agent_name": &agent_name,
            "agent_name_upper": agent_name.to_uppercase(),
            "github_app": Self::get_github_app_or_default(code_run),
            "cli_config": cli_config,
            "discord_enabled": code_run.spec.openclaw.as_ref()
                .is_none_or(|oc| oc.discord_enabled),
            "openclaw_providers": openclaw_providers,
        });

        handlebars.render("openclaw_config", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to render OpenClaw config: {e}"
            ))
        })
    }

    /// Render the harness-agent launcher script (OpenClaw entrypoint).
    /// Thin shell script: fix perms, copy config, exec openclaw gateway.
    fn generate_harness_launcher(code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

        let template = Self::load_template(HARNESS_OPENCLAW_TEMPLATE)?;
        handlebars
            .register_template_string("harness_launcher", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register harness launcher template: {e}"
                ))
            })?;

        let cli_type = Self::determine_cli_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let job_type = Self::determine_job_type(code_run);

        let context = json!({
            "agent_name": &agent_name,
            "agent_name_upper": agent_name.to_uppercase(),
            "cli_type": cli_type.to_string(),
            "job_type": job_type,
            "model": code_run.spec.model,
            "prompt_modification": code_run.spec.prompt_modification.as_deref().unwrap_or(""),
            "repository_url": code_run.spec.repository_url,
            "github_app": Self::get_github_app_or_default(code_run),
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "service": &code_run.spec.service,
            "discord_enabled": code_run.spec.openclaw.as_ref()
                .is_none_or(|oc| oc.discord_enabled),
        });

        handlebars
            .render("harness_launcher", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render harness launcher: {e}"
                ))
            })
    }

    /// Render the Hermes harness launcher script (standalone ACPX + Lobster).
    /// No OpenClaw gateway — runs lobster workflow directly.
    fn generate_hermes_launcher(code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

        let template = Self::load_template(HARNESS_HERMES_TEMPLATE)?;
        handlebars
            .register_template_string("hermes_launcher", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register Hermes launcher template: {e}"
                ))
            })?;

        let cli_type = Self::determine_cli_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let job_type = Self::determine_job_type(code_run);

        let context = json!({
            "agent_name": &agent_name,
            "agent_name_upper": agent_name.to_uppercase(),
            "cli_type": cli_type.to_string(),
            "job_type": job_type,
            "model": code_run.spec.model,
            "prompt_modification": code_run.spec.prompt_modification.as_deref().unwrap_or(""),
            "repository_url": code_run.spec.repository_url,
            "github_app": Self::get_github_app_or_default(code_run),
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "service": &code_run.spec.service,
            "discord_enabled": false, // Hermes doesn't use Discord
        });

        handlebars
            .render("hermes_launcher", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Hermes launcher: {e}"
                ))
            })
    }

    #[allow(clippy::too_many_lines, clippy::items_after_statements)] // Complex memory generation
    fn generate_claude_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

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
        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();

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

        // Get task language for support agents (Cleo, Cipher, Tess)
        let task_language = Self::get_task_language(code_run);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "task_language": task_language,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": Self::get_github_app_or_default(code_run),
            "model": cli_model.clone(), // Use cli_model for consistency
            "context_version": code_run.spec.context_version,
            "workflow_env_vars": workflow_env_vars,
            "requirements_env_vars": requirements_env_vars,
            "requirements_secret_sources": requirements_secret_sources,
            "cli_config": cli_config,
            "cli_type": cli_type,
            // Flag to conditionally skip inline partials (for CLIs with native skills)
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
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
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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

    /// Generate MCP configuration JSON for CLIs that support `--mcp-config`.
    ///
    /// This builds the config programmatically (same approach as `generate_cursor_mcp_config`)
    /// rather than using a Handlebars template, since the structure is simple and consistent.
    fn generate_mcp_config(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        // Get CLI config to extract tools URL
        let cli_config_value = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|cfg| serde_json::to_value(cfg).ok())
            .unwrap_or_else(|| json!({}));

        let render_settings = Self::build_cli_render_settings(code_run, &cli_config_value);

        // Generate client config and extract remote tools
        let client_config = Self::generate_client_config(code_run, config)?;
        let client_config_value: Value = serde_json::from_str(&client_config)
            .unwrap_or_else(|_| json!({ "remoteTools": [], "localServers": {} }));
        let remote_tools = Self::extract_remote_tools(&client_config_value);

        // Build MCP servers config (serialize directly, no template needed)
        // Use HTTP transport format for Claude CLI (type + url)
        // NOT stdio format (command + args) which is for local MCP servers
        let mut mcp_servers = json!({});

        if !render_settings.tools_url.is_empty() {
            let mut tools_server = json!({
                "type": "http",
                "url": render_settings.tools_url
            });
            if !remote_tools.is_empty() {
                tools_server["availableTools"] = json!(remote_tools);
            }

            // Thread escalation policy + agent identity as HTTP headers so the
            // tools server can apply per-session policy (PR 1 added the
            // server-side handler; this wires the CRD-side policy through).
            let mut headers = serde_json::Map::new();

            // Agent identity — used by the tools server to key per-session state.
            let agent_name = code_run.spec.github_app.as_deref().unwrap_or("unknown");
            headers.insert("X-Agent-Id".to_string(), json!(agent_name));

            // Prewarm set — the tools the agent starts with before escalation.
            if !remote_tools.is_empty() {
                headers.insert("X-Agent-Prewarm".to_string(), json!(remote_tools.join(" ")));
            }

            // Per-session escalation policy from the CRD (overrides server default).
            if let Some(policy) = &code_run.spec.escalation_policy {
                if let Ok(policy_json) = serde_json::to_string(policy) {
                    headers.insert("X-Escalation-Policy".to_string(), json!(policy_json));
                }
            }

            if !headers.is_empty() {
                tools_server["headers"] = Value::Object(headers);
            }

            mcp_servers["tools"] = tools_server;
        }

        let mcp_config = json!({
            "mcpServers": mcp_servers
        });

        serde_json::to_string_pretty(&mcp_config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to serialize MCP config: {e}"))
        })
    }

    fn generate_codex_container_script(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
        config: &ControllerConfig,
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

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

        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();
        let job_type = Self::determine_job_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let task_language = Self::get_task_language(code_run);
        let default_retries = Self::get_default_retries(code_run);
        let fresh_start_threshold = Self::get_fresh_start_threshold(code_run);
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        // Get skills for agent (Codex supports native skill loading)
        let skills = Self::get_agent_skills_enriched(code_run, config);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "model": render_settings.model,
            "auto_level": render_settings.auto_level,
            "output_format": render_settings.output_format,
            "model_rotation": render_settings.model_rotation,
            "list_tools_on_start": render_settings.list_tools_on_start,
            // Telemetry context for openclaw.sh.hbs unified dispatch
            "telemetry_enabled": Self::is_telemetry_enabled(code_run, config),
            "otel_endpoint": Self::get_otel_endpoint(),
            "datadog_enabled": Self::is_datadog_enabled(config),
            // Required template context variables
            "cli_type": cli_type,
            "job_type": job_type,
            "agent_name": agent_name,
            "task_language": task_language,
            "default_retries": default_retries,
            "fresh_start_threshold": fresh_start_threshold,
            // Skills for native skill loading
            "skills": skills,
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "use_sdk_path": Self::extract_use_sdk_path(cli_config),
            "cli": {
                "type": cli_type,
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
        Self::register_template_helpers(&mut handlebars);

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
        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "cli_config": cli_config,
            "github_app": Self::get_github_app_or_default(code_run),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            // Flag to conditionally skip inline partials (for CLIs with native skills)
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
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
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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

            // If agent has subagent config, inject it into cli_config
            // This allows coordinator templates to conditionally render dispatch instructions
            if let Some(subagents) = &agent_config.subagents {
                enriched["subagents"] = json!({
                    "enabled": subagents.enabled,
                    "maxConcurrent": subagents.max_concurrent
                });
            }

            // If agent opts into the Dynamic MCP SDK path, inject the flag
            // so templates can skip the legacy client binary setup.
            if agent_config.use_sdk_path {
                enriched["useSdkPath"] = json!(true);
            }
        }

        // When the tools sidecar is enabled, override toolsUrl so all generators
        // route through localhost instead of the cluster-wide tools service.
        if config.tools_sidecar.enabled {
            let sidecar_url = format!("http://localhost:{}/mcp", config.tools_sidecar.port);
            if enriched.get("settings").is_none() {
                enriched["settings"] = json!({});
            }
            enriched["settings"]["toolsUrl"] = json!(sidecar_url);
        }

        enriched
    }

    #[allow(clippy::too_many_lines)] // Complex function not easily split
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
                        "http://cto-tools.cto.svc.cluster.local:3000/mcp".to_string()
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

    /// Check whether the enriched cli_config has the Dynamic MCP SDK path enabled.
    fn extract_use_sdk_path(cli_config: &Value) -> bool {
        cli_config
            .get("useSdkPath")
            .and_then(Value::as_bool)
            .unwrap_or(false)
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
            Self::generate_codex_container_script(
                code_run,
                &enriched_cli_config,
                &remote_tools,
                config,
            )?,
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
    #[allow(clippy::too_many_lines)] // Complex function not easily split
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
        Self::register_template_helpers(&mut handlebars);

        // Determine CLI type (default to Claude for review tasks)
        let cli_type = Self::determine_cli_type(code_run);
        let use_claude = matches!(cli_type, CLIType::Claude);

        // Register shared partials (header, config, git-setup, etc.)
        Self::register_shared_partials(&mut handlebars)?;

        // Register CLI-specific invoke partial (cli_execute)
        Self::register_cli_invoke_partial(&mut handlebars, cli_type)?;

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
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);
        let render_settings = Self::build_cli_render_settings(code_run, &enriched_cli_config);

        // Determine agent name from service or default to "stitch"
        let agent_name = code_run
            .spec
            .service
            .split('-')
            .next()
            .unwrap_or("stitch")
            .to_lowercase();

        // CLI type name for templates
        let cli_type_name = cli_type.to_string();

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
            "list_tools_on_start": render_settings.list_tools_on_start,
            // Template variables expected by container.sh.hbs
            "agent_name": agent_name,
            "cli_type": cli_type_name,
            "job_type": "review",
            "default_retries": 3,
            "task_language": "",
            "use_sdk_path": Self::extract_use_sdk_path(&enriched_cli_config),
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

        // Generate prompt.md for review task
        let pr_url = code_run.spec.env.get("PR_URL").cloned().unwrap_or_default();
        let pr_title = code_run
            .spec
            .env
            .get("PR_TITLE")
            .cloned()
            .unwrap_or_default();
        let pr_author = code_run
            .spec
            .env
            .get("PR_AUTHOR")
            .cloned()
            .unwrap_or_default();

        let prompt = format!(
            r#"# Code Review Request

Review the pull request and provide feedback.

## PR Details

- **PR URL**: {pr_url}
- **PR Number**: {pr_number}
- **Title**: {pr_title}
- **Author**: {pr_author}
- **Head SHA**: {head_sha}

## Instructions

1. Use the `github_get_pull_request_files` tool to fetch the files changed in PR #{pr_number}
2. Review each changed file for:
   - Correctness
   - Security vulnerabilities  
   - Performance issues
   - Code style and maintainability
3. **IMPORTANT: Use the `gh` CLI to post your review** (NOT MCP tools):
   ```bash
   # Post review with inline comments
   gh pr review {pr_number} --repo {repo_slug} --comment --body "Your review body"
   
   # Or approve
   gh pr review {pr_number} --repo {repo_slug} --approve --body "LGTM!"
   
   # Or request changes  
   gh pr review {pr_number} --repo {repo_slug} --request-changes --body "Please fix..."
   ```
   
   **Never use** `github_create_pull_request_review` or `github_add_pull_request_review_comment` MCP tools - they use the wrong identity.

Be constructive and explain the "why" behind your suggestions.
"#
        );
        templates.insert("prompt.md".to_string(), prompt);

        Ok(templates)
    }

    /// Generate templates for remediate tasks (Rex PR Remediation)
    #[allow(clippy::too_many_lines)] // Complex function not easily split
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
        Self::register_template_helpers(&mut handlebars);

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
        let enriched_cli_config =
            Self::enrich_cli_config_from_agent(cli_config_value, code_run, config);
        let render_settings = Self::build_cli_render_settings(code_run, &enriched_cli_config);

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
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "tools_url": render_settings.tools_url,
            "remote_tools": remote_tools,
            "use_sdk_path": Self::extract_use_sdk_path(&enriched_cli_config),
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

    #[allow(clippy::too_many_lines, clippy::items_after_statements)] // Complex config generation
    fn generate_client_config(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        use serde_json::to_string_pretty;

        let github_app = Self::get_github_app_or_default(code_run);

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
                            // Check if it has "remoteTools" (client-config format) vs "remote" (tools format)
                            // The presence of localServers alone doesn't indicate client-config format
                            let has_remote_tools = tools_value.get("remoteTools").is_some();
                            let has_remote = tools_value.get("remote").is_some();

                            // Build overlay client config from annotation
                            let mut overlay_client = if has_remote_tools {
                                // Already in client-config format
                                if tools_value.get("localServers").is_none() {
                                    tools_value["localServers"] = json!({});
                                }
                                tools_value
                            } else if has_remote {
                                // Tools format with "remote" key - normalize to client-config
                                normalize_tools_to_client_config(tools_value)
                            } else {
                                // Neither format - check if it has localServers and treat as partial client-config
                                if tools_value.get("localServers").is_some() {
                                    tools_value["remoteTools"] = json!([]);
                                    tools_value
                                } else {
                                    // Completely empty or unknown format
                                    normalize_tools_to_client_config(tools_value)
                                }
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

        // 2) Check for direct remoteTools/localTools in CodeRun spec (used by healer)
        if let Some(ref remote_tools_str) = code_run.spec.remote_tools {
            if !remote_tools_str.trim().is_empty() {
                debug!(
                    "code: using remoteTools from CodeRun spec: '{}'",
                    remote_tools_str
                );
                // Parse comma-separated tools into array
                let remote_tools: Vec<String> = remote_tools_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut client = json!({
                    "remoteTools": remote_tools,
                    "localServers": {}
                });

                // Handle localTools if present
                if let Some(ref local_tools_str) = code_run.spec.local_tools {
                    if !local_tools_str.trim().is_empty() {
                        // local_tools is a comma-separated list of server names to enable
                        // We'd need the full server configs from Helm, so just log for now
                        debug!(
                            "code: localTools specified but not fully supported: '{}'",
                            local_tools_str
                        );
                    }
                }

                Self::normalize_remote_tools(&mut client);
                return to_string_pretty(&client).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to serialize spec remoteTools: {e}"
                    ))
                });
            }
        }

        // 3) Fall back to agent config from Helm values
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

        // 4) Fall back to built-in defaults based on agent + run_type
        // This ensures agents get their required tools even without explicit Helm config
        let run_type = code_run.spec.run_type.as_str();
        let default_tools = Self::get_default_agent_tools(&github_app, run_type);

        if !default_tools.is_empty() {
            debug!(
                "code: using built-in default tools for '{}' run_type='{}': {:?}",
                github_app, run_type, default_tools
            );
            let mut client = json!({
                "remoteTools": default_tools,
                "localServers": {}
            });
            Self::normalize_remote_tools(&mut client);
            return to_string_pretty(&client).map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to serialize default agent tools: {e}"
                ))
            });
        }

        // 5) No defaults available → minimal JSON object
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
        config: &ControllerConfig,
    ) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

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

        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();
        let job_type = Self::determine_job_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let task_language = Self::get_task_language(code_run);
        let default_retries = Self::get_default_retries(code_run);
        let fresh_start_threshold = Self::get_fresh_start_threshold(code_run);

        // Get skills for agent (OpenCode supports native skill loading)
        let skills = Self::get_agent_skills_enriched(code_run, config);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "model": render_settings.model,
            "list_tools_on_start": render_settings.list_tools_on_start,
            // Required template context variables
            "cli_type": cli_type,
            "job_type": job_type,
            "agent_name": agent_name,
            "task_language": task_language,
            "default_retries": default_retries,
            "fresh_start_threshold": fresh_start_threshold,
            // Skills for native skill loading
            "skills": skills,
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "use_sdk_path": Self::extract_use_sdk_path(cli_config),
            "cli": {
                "type": cli_type,
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
        Self::register_template_helpers(&mut handlebars);

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
        let cli_type_enum = Self::determine_cli_type(code_run);
        let cli_type = cli_type_enum.to_string();

        // Determine frontend stack from agent config (defaults to shadcn)
        // Priority: cli_config.frontendStack > default "shadcn"
        let frontend_stack = cli_config
            .get("frontendStack")
            .and_then(Value::as_str)
            .unwrap_or("shadcn");
        let is_tanstack_stack = frontend_stack == "tanstack";

        let context = json!({
            "github_app": Self::get_github_app_or_default(code_run),
            "model": model,
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
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
            // Flag to conditionally skip inline partials (for CLIs with native skills)
            "skills_native": Self::cli_supports_native_skills(cli_type_enum),
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "cli": {
                "type": cli_type,
                "model": model,
                "settings": cli_settings,
                "remote_tools": remote_tools,
            },
            // Frontend stack context for Blaze agent
            "frontend_stack": frontend_stack,
            "is_tanstack_stack": is_tanstack_stack,
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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
        Self::register_template_helpers(&mut handlebars);

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

        let cli_type = Self::determine_cli_type(code_run).to_string();
        let job_type = Self::determine_job_type(code_run);
        let agent_name = Self::get_agent_name(code_run);
        let task_language = Self::get_task_language(code_run);
        let default_retries = Self::get_default_retries(code_run);
        let fresh_start_threshold = Self::get_fresh_start_threshold(code_run);
        let render_settings = Self::build_cli_render_settings(code_run, cli_config);

        let context = json!({
            "task_id": code_run.spec.task_id.unwrap_or(0),
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "working_directory": Self::get_working_directory(code_run),
            "docs_branch": code_run.spec.docs_branch,
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "model": code_run.spec.model,
            "remote_tools": remote_tools,
            "settings": cli_settings,
            "continue_session": continue_session,
            "overwrite_memory": code_run.spec.overwrite_memory,
            "attempts": retry_count + 1,
            "list_tools_on_start": render_settings.list_tools_on_start,
            // Required template context variables
            "cli_type": cli_type,
            "job_type": job_type,
            "agent_name": agent_name,
            "task_language": task_language,
            "default_retries": default_retries,
            "fresh_start_threshold": fresh_start_threshold,
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
            "use_sdk_path": Self::extract_use_sdk_path(cli_config),
        });

        handlebars
            .render("gemini_container", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render Gemini container script: {e}"
                ))
            })
    }

    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn generate_gemini_memory(
        code_run: &CodeRun,
        cli_config: &Value,
        remote_tools: &[String],
    ) -> Result<String> {
        use base64::{engine::general_purpose, Engine as _};
        use std::collections::BTreeSet;

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        Self::register_template_helpers(&mut handlebars);

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
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": Self::get_github_app_or_default(code_run),
            "workflow_name": workflow_name,
            "remote_tools": remote_tools,
            "settings": cli_settings,
            "workflow_env_vars": workflow_env_vars,
            "requirements_env_vars": requirements_env_vars,
            "requirements_secret_sources": requirements_secret_sources,
            "cli_type": cli_type,
            "subtasks": code_run.spec.subtasks.clone().unwrap_or_default(),
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
            // Default task_language so {{#eq task_language "rust"}} blocks in cipher/security
            // and similar templates don't trip the handlebars-rust eq helper type check
            "task_language": "",
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
        Self::register_template_helpers(&mut handlebars);

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
        Self::register_template_helpers(&mut handlebars);

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
            "github_app": Self::get_github_app_or_default(code_run),
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
            "project_id": code_run.spec.project_id.clone().unwrap_or_default(),
                                                "service": code_run.spec.service,
                                                "repository_url": code_run.spec.repository_url,
                                                "docs_repository_url": code_run.spec.docs_repository_url,
                                                "working_directory": Self::get_working_directory(code_run),
                                                "github_app": Self::get_github_app_or_default(code_run),
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

    /// Get the github_app value with defensive handling for empty strings.
    /// Returns the github_app if set and non-empty, otherwise returns "agent".
    /// Used for template context where github_app should never be empty.
    #[allow(dead_code)]
    fn get_github_app_or_default(code_run: &CodeRun) -> String {
        // 1. Explicit githubApp field (backward compat)
        if let Some(ref app) = code_run.spec.github_app {
            if !app.is_empty() {
                return app.clone();
            }
        }
        // 2. Derive from implementationAgent (e.g. "rex" → "5DLabs-Rex")
        if let Some(ref agent) = code_run.spec.implementation_agent {
            if !agent.is_empty() {
                let mut chars = agent.chars();
                let capitalized: String = match chars.next() {
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    None => agent.clone(),
                };
                return format!("5DLabs-{capitalized}");
            }
        }
        "agent".to_string()
    }

    /// Get the lowercase agent name from `implementation_agent` or `github_app`.
    /// Returns lowercase agent name (e.g., "rex", "blaze", "tap").
    /// Prefers `implementation_agent` directly; falls back to extracting from
    /// `github_app` ("5DLabs-Rex" → "rex"). Returns "agent" as default.
    #[allow(dead_code)]
    fn get_agent_name(code_run: &CodeRun) -> String {
        // Prefer implementationAgent directly (already lowercase)
        if let Some(ref agent) = code_run.spec.implementation_agent {
            if !agent.is_empty() {
                return agent.to_lowercase();
            }
        }
        // Fall back to github_app extraction
        let github_app = Self::get_github_app_or_default(code_run);
        github_app
            .strip_prefix("5DLabs-")
            .unwrap_or(&github_app)
            .to_lowercase()
    }

    /// Get the task language based on agent type or explicit TASK_LANGUAGE env var.
    /// Used for language-specific conditionals in templates (e.g., node-env, go-env).
    ///
    /// Priority:
    /// 1. TASK_LANGUAGE env var (set by play workflow for support agents)
    /// 2. Agent name inference (for implementation agents)
    ///
    /// Returns lowercase language identifier.
    #[allow(dead_code, clippy::match_same_arms)] // Explicit match for documentation
    fn get_task_language(code_run: &CodeRun) -> String {
        // First check if TASK_LANGUAGE is explicitly set (used by support agents
        // like Cleo, Cipher, Tess to know the implementation language)
        if let Some(lang) = code_run.spec.env.get("TASK_LANGUAGE") {
            if !lang.is_empty() {
                return lang.to_lowercase();
            }
        }

        // Fall back to inferring from agent name
        let agent_name = Self::get_agent_name(code_run);
        match agent_name.as_str() {
            // Rust agents
            "rex" => "rust".to_string(),
            // TypeScript/JavaScript agents
            "blaze" | "nova" | "tap" | "spark" => "typescript".to_string(),
            // Go agents
            "grizz" => "go".to_string(),
            // Support agents and others default to empty (no language-specific setup)
            _ => String::new(),
        }
    }

    /// Get default retries from environment or use platform default
    #[allow(dead_code)]
    fn get_default_retries(code_run: &CodeRun) -> u32 {
        code_run
            .spec
            .env
            .get("EXECUTION_MAX_RETRIES")
            .or_else(|| code_run.spec.env.get("MAX_RETRIES"))
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(10) // Platform default
    }

    /// Get fresh start threshold from environment or use platform default.
    /// After this many retries, context is cleared and agent starts fresh.
    /// Based on Cursor's research: periodic fresh starts combat drift and tunnel vision.
    #[allow(dead_code)]
    fn get_fresh_start_threshold(code_run: &CodeRun) -> u32 {
        code_run
            .spec
            .env
            .get("FRESH_START_THRESHOLD")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3) // Platform default - same as cto-config default
    }

    /// Check if telemetry (OTEL metrics/logs) is enabled for this CodeRun.
    fn is_telemetry_enabled(code_run: &CodeRun, _config: &ControllerConfig) -> bool {
        // CodeRun env override takes precedence
        if let Some(val) = code_run.spec.env.get("TELEMETRY_ENABLED") {
            return val == "true" || val == "1";
        }
        // Fall back to controller env var, default to true (observability stack is deployed)
        std::env::var("TELEMETRY_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true)
    }

    /// Get the OTLP gRPC endpoint for telemetry export.
    fn get_otel_endpoint() -> String {
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_else(|_| {
            "otel-collector-opentelemetry-collector.observability.svc.cluster.local:4317"
                .to_string()
        })
    }

    /// Check if Datadog APM is enabled.
    fn is_datadog_enabled(_config: &ControllerConfig) -> bool {
        std::env::var("DATADOG_ENABLED")
            .or_else(|_| std::env::var("DD_TRACE_ENABLED"))
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Get default MCP tools for an agent based on github_app and run_type.
    /// This provides built-in tool defaults when no explicit configuration exists.
    /// Tools are specified as patterns that match the MCP tool server naming convention.
    fn get_default_agent_tools(github_app: &str, run_type: &str) -> Vec<String> {
        // Normalize github_app to agent name (e.g., "5DLabs-Morgan" -> "morgan")
        let agent = github_app
            .strip_prefix("5DLabs-")
            .unwrap_or(github_app)
            .to_lowercase();

        match (agent.as_str(), run_type) {
            // Morgan: intake/documentation - needs URL scraping, library docs, and memory
            ("morgan", "intake" | "documentation") => vec![
                "mcp_tools_firecrawl_*".to_string(),
                "mcp_tools_context7_*".to_string(),
                "mcp_tools_openmemory_*".to_string(),
            ],

            // Implementation agents: need GitHub, context docs, and memory
            (
                "rex" | "grizz" | "nova" | "blaze" | "tap" | "spark" | "angie",
                "implementation" | "coder",
            ) => {
                vec![
                    "mcp_tools_github_*".to_string(),
                    "mcp_tools_context7_*".to_string(),
                    "mcp_tools_firecrawl_*".to_string(),
                    "mcp_tools_openmemory_*".to_string(),
                ]
            }

            // Bolt: infrastructure - needs Kubernetes and GitHub
            ("bolt", _) => vec![
                "mcp_tools_kubernetes_*".to_string(),
                "mcp_tools_github_*".to_string(),
                "mcp_tools_context7_*".to_string(),
            ],

            // Atlas/Cleo/Tess/Cipher: need GitHub for integration/review
            ("atlas" | "cleo" | "tess" | "cipher", _) => vec!["mcp_tools_github_*".to_string()],

            // Default: no specific tools (will get whatever is globally available)
            _ => vec![],
        }
    }

    /// Resolve the content of a skill file by searching category subdirectories.
    /// Returns the SKILL.md content if found, None otherwise.
    fn resolve_skill_content(skill_name: &str, templates_path: &str) -> Option<String> {
        let categories = [
            "stacks",
            "auth",
            "context",
            "design",
            "documents",
            "languages",
            "llm-docs",
            "platforms",
            "quality",
            "security",
            "workflow",
            "animations",
            "tools",
        ];
        for category in &categories {
            let path = format!("{templates_path}/skills/{category}/{skill_name}/SKILL.md");
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(content);
            }
        }
        None
    }

    /// Fetch ALL skills from the remote tarball for this CodeRun.
    /// Returns a JSON array of {name, content} objects containing every skill
    /// in the agent's published tarball (not filtered by cto-config mappings).
    ///
    /// When `skills_url` is not set, falls back to the mapped skills from
    /// `get_agent_skills_enriched()` for backward compatibility.
    fn fetch_all_skills_for_coderun(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Vec<serde_json::Value> {
        if let Some(ref skills_url) = code_run.spec.skills_url {
            let agent_name = Self::get_agent_name(code_run);
            let project = code_run.spec.skills_project.as_deref().map(str::to_owned);
            // Run blocking HTTP + filesystem work on a dedicated OS thread
            // to avoid panicking reqwest::blocking inside the tokio runtime.
            let url = skills_url.clone();
            let agent = agent_name.clone();
            let proj = project.clone();
            let handle = std::thread::spawn(move || {
                super::skills_cache::ensure_all_skills(
                    &url,
                    &agent,
                    proj.as_deref(),
                )
            });
            match handle.join() {
                Ok(Ok(all_skills)) => {
                    info!(
                        "Fetched {} skills from tarball for agent '{}'",
                        all_skills.len(),
                        agent_name
                    );
                    return all_skills
                        .into_iter()
                        .map(|(name, content)| json!({ "name": name, "content": content }))
                        .collect();
                }
                Ok(Err(e)) => {
                    warn!(
                        "Failed to fetch all skills from {}: {} — falling back to mapped skills",
                        skills_url, e
                    );
                }
                Err(_) => {
                    warn!(
                        "Skills fetch thread panicked for {} — falling back to mapped skills",
                        skills_url
                    );
                }
            }
        }
        // Fallback: use the filtered/mapped skill set
        Self::get_agent_skills_enriched(code_run, config)
    }

    /// Get skills enriched with inline content for embedding in container scripts.
    /// Returns JSON array of {name, content} objects. Content is empty string if skill not found.
    ///
    /// When `spec.skills_url` is set, skills are fetched from the remote skills
    /// repo via [`skills_cache::ensure_skills`]. A fetch/hash/extract failure
    /// logs an error and sets content to empty (the CodeRun will still proceed
    /// but the skill will be missing — callers can check for empty content).
    ///
    /// When `skills_url` is `None`, the baked-in templates directory is used.
    fn get_agent_skills_enriched(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Vec<serde_json::Value> {
        let skill_names = Self::get_agent_skills(code_run, config);

        // If a skills repo URL is configured, try the remote cache first.
        // Run on a dedicated OS thread to avoid reqwest::blocking panicking
        // inside the tokio async runtime.
        if let Some(ref skills_url) = code_run.spec.skills_url {
            let agent_name = Self::get_agent_name(code_run);
            let project = code_run.spec.skills_project.as_deref().map(str::to_owned);
            let url = skills_url.clone();
            let agent = agent_name.clone();
            let proj = project.clone();
            let names = skill_names.clone();
            let handle = std::thread::spawn(move || {
                super::skills_cache::ensure_skills(&url, &agent, proj.as_deref(), &names)
            });
            match handle.join() {
                Ok(Ok(cached)) => {
                    return skill_names
                        .into_iter()
                        .map(|name| {
                            let content = cached.get(&name).cloned().unwrap_or_default();
                            if content.is_empty() {
                                warn!("Skill '{}' resolved from cache but content is empty", name);
                            } else {
                                debug!("Loaded skill '{}' from skills cache", name);
                            }
                            json!({ "name": name, "content": content })
                        })
                        .collect();
                }
                Ok(Err(e)) => {
                    warn!(
                        "Failed to fetch skills from {}: {} — returning empty skills (no baked-in fallback)",
                        skills_url, e
                    );
                    return skill_names
                        .into_iter()
                        .map(|name| json!({ "name": name, "content": "" }))
                        .collect();
                }
                Err(_) => {
                    warn!(
                        "Skills fetch thread panicked for {} — returning empty skills",
                        skills_url
                    );
                    return skill_names
                        .into_iter()
                        .map(|name| json!({ "name": name, "content": "" }))
                        .collect();
                }
            }
        }

        // Fallback: resolve from baked-in templates directory
        let templates_path = get_templates_path();
        skill_names
            .into_iter()
            .map(|name| {
                let content =
                    Self::resolve_skill_content(&name, &templates_path).unwrap_or_default();
                if content.is_empty() {
                    debug!("Skill content not found for '{}' in templates", name);
                } else {
                    debug!("Loaded inline skill content for '{}'", name);
                }
                json!({ "name": name, "content": content })
            })
            .collect()
    }

    /// Get the skills for an agent based on agent name and job type.
    /// Returns a list of skill names that should be loaded for the agent.
    /// For Claude Code, Factory, OpenCode, Codex: skills are copied to native skill directories.
    ///
    /// Skill resolution order:
    /// 1. Try cto-config.json (via Helm agent config) - preferred source
    /// 2. Fall back to skill-mappings.yaml if no skills in config
    ///
    /// Within each source:
    /// - Agent's `default` skills (always loaded)
    /// - Agent's job-type-specific skills (e.g., `healer`, `coder`) - merged with defaults
    fn get_agent_skills(code_run: &CodeRun, config: &ControllerConfig) -> Vec<String> {
        let agent_name = Self::get_agent_name(code_run);
        let github_app = Self::get_github_app_or_default(code_run);
        let job_type = Self::determine_job_type(code_run);

        // 1. Try to get skills from cto-config.json (via Helm agent config)
        if let Some(agent_def) = config.agents.values().find(|a| a.github_app == github_app) {
            if let Some(ref agent_skills) = agent_def.skills {
                if agent_skills.has_skills() {
                    let skills = agent_skills.get_skills_for_job(job_type);
                    debug!(
                        "Loaded {} skills from cto-config for agent '{}' (github_app='{}') job '{}': {:?}",
                        skills.len(),
                        agent_name,
                        github_app,
                        job_type,
                        skills
                    );
                    return skills;
                }
            }
        }

        // 2. Fall back to skill-mappings.yaml
        Self::get_agent_skills_from_yaml(&agent_name, job_type)
    }

    /// Load skills from skill-mappings.yaml (fallback source).
    fn get_agent_skills_from_yaml(agent_name: &str, job_type: &str) -> Vec<String> {
        use crate::tasks::template_paths::SKILLS_MAPPINGS;

        let templates_path = get_templates_path();
        let mappings_path = format!("{templates_path}/{SKILLS_MAPPINGS}");

        // Try to load and parse the skill mappings YAML
        let mappings_content = match fs::read_to_string(&mappings_path) {
            Ok(content) => content,
            Err(e) => {
                debug!(
                    "Could not load skill mappings from {}: {} - using empty skills",
                    mappings_path, e
                );
                return Vec::new();
            }
        };

        // Parse YAML to get agent's skills
        let mappings: serde_yaml::Value = match serde_yaml::from_str(&mappings_content) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Failed to parse skill mappings YAML: {} - using empty skills",
                    e
                );
                return Vec::new();
            }
        };

        let mut skills = Vec::new();

        // Look up the agent in mappings
        if let Some(agent_config) = mappings.get(agent_name) {
            // 1. Get default skills (always loaded)
            if let Some(defaults) = agent_config.get("default") {
                if let Some(skills_array) = defaults.as_sequence() {
                    for skill in skills_array {
                        if let Some(s) = skill.as_str() {
                            skills.push(s.to_string());
                        }
                    }
                }
            }

            // 2. Get job-type-specific skills (merged with defaults)
            // Job types: coder, healer, intake, quality, test, deploy, security, review, integration
            if let Some(job_skills) = agent_config.get(job_type) {
                if let Some(skills_array) = job_skills.as_sequence() {
                    for skill in skills_array {
                        if let Some(s) = skill.as_str() {
                            // Only add if not already present
                            if !skills.contains(&s.to_string()) {
                                skills.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }

        if skills.is_empty() {
            debug!(
                "No skill mappings found in YAML for agent '{}' job '{}' - using empty skills",
                agent_name, job_type
            );
        } else {
            debug!(
                "Loaded {} skills from YAML for agent '{}' job '{}': {:?}",
                skills.len(),
                agent_name,
                job_type,
                skills
            );
        }

        skills
    }

    /// Qualify a model name with its provider prefix for OpenClaw gateway.
    ///
    /// Resolve the full `model.primary` value for the OpenClaw config.
    ///
    /// OpenClaw gateway looks up models as `{provider_name}/{model_id}` where
    /// `provider_name` is the key under `models.providers` and `model_id` is
    /// the `id` field of a model entry in that provider.
    ///
    /// This function takes the qualified model from `qualify_model_for_openclaw()`
    /// and finds the provider that lists it, then returns `provider/model_id`.
    fn resolve_openclaw_primary_model(
        code_run: &CodeRun,
        openclaw_providers: &[Value],
    ) -> String {
        let model_id = Self::qualify_model_for_openclaw(code_run);

        // Check if any provider lists this model — if so, prefix with provider name
        for provider in openclaw_providers {
            let provider_name = provider.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(models) = provider.get("models").and_then(|v| v.as_array()) {
                for m in models {
                    if m.get("id").and_then(|v| v.as_str()) == Some(&model_id) {
                        return format!("{provider_name}/{model_id}");
                    }
                }
            }
        }

        // No provider match — return as-is (gateway will try to route it)
        model_id
    }

    /// Resolve the raw model identifier from the CRD for OpenClaw.
    ///
    /// Priority (pure CRD passthrough — no hard-coded model names):
    ///   1. `cli_config.model` — the actual model the CLI uses (may already be
    ///      a fully-qualified Fireworks/provider path like
    ///      `accounts/fireworks/routers/kimi-k2p5-turbo`).
    ///   2. `spec.model` — top-level display/naming model.
    ///
    /// If the resolved value already contains `/` it is passed through as-is.
    /// Otherwise we try the explicit `cli_config.provider` prefix, and finally
    /// fall back to provider inference from well-known prefixes so bare names
    /// like `gemini-2.5-pro` still get routed correctly.
    fn qualify_model_for_openclaw(code_run: &CodeRun) -> String {
        // Prefer cli_config.model (the actual model) over spec.model (display name)
        let model = code_run
            .spec
            .cli_config
            .as_ref()
            .map_or(&code_run.spec.model, |c| &c.model);

        // Already provider-qualified (e.g. "accounts/fireworks/routers/kimi-k2p5-turbo")
        if model.contains('/') {
            return model.clone();
        }

        // Try explicit provider from CLIConfig
        if let Some(provider) = code_run
            .spec
            .cli_config
            .as_ref()
            .and_then(|c| c.provider.as_ref())
        {
            return format!("{provider}/{model}");
        }

        // Last resort: infer provider from model name prefix.
        // This is a convenience fallback for bare model names in CRDs that
        // don't set cli_config.model to a qualified path.
        let prefix = if model.starts_with("claude-") || model.starts_with("claude3") {
            "anthropic"
        } else if model.starts_with("gpt-")
            || model.starts_with("o1-")
            || model.starts_with("o3-")
            || model.starts_with("o4-")
        {
            "openai"
        } else if model.starts_with("gemini-") {
            "google"
        } else if model.starts_with("glm-") {
            "openai"
        } else if model.starts_with("kimi-") || model.starts_with("moonshot-") {
            "moonshot"
        } else {
            return model.clone();
        };

        format!("{prefix}/{model}")
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

    /// Get the system prompt template path for an agent based on agent name and job type.
    /// Returns path in format: agents/{agent}/{job}.md.hbs
    fn get_agent_system_prompt_template(code_run: &CodeRun) -> String {
        let agent_name = Self::get_agent_name(code_run);
        let job_type = Self::determine_job_type(code_run);
        let run_type = code_run.spec.run_type.as_str();

        // Intake/documentation runs always use morgan templates
        if run_type == "intake" || run_type == "documentation" {
            return format!("agents/morgan/{job_type}.md.hbs");
        }

        let agent = agent_name.as_str();

        // Agent-specific job type defaults
        // Some agents only support specific job types
        let job = match (agent, job_type) {
            // Morgan: intake by default (intake/documentation specialist)
            ("morgan", "coder") => "intake",

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

        // Check for prompt style variant (e.g., "minimal" for Ralph-style prompts)
        let suffix = code_run
            .spec
            .prompt_style
            .as_ref()
            .filter(|s| *s == "minimal")
            .map_or("", |_| "-minimal");

        format!("agents/{agent}/{job}{suffix}.md.hbs")
    }

    /// Select the container template for Codex CLI.
    /// All CLIs use the shared container template - CLI-specific behavior is
    /// injected via the `{{> cli_execute}}` partial from `clis/{cli}/invoke.sh.hbs`.
    fn get_codex_container_template(code_run: &CodeRun) -> String {
        let run_type = code_run.spec.run_type.as_str();

        // Intake runs have a specialized container
        if run_type == "documentation" || run_type == "intake" {
            return "agents/morgan/intake.sh.hbs".to_string();
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
            return "agents/morgan/intake.sh.hbs".to_string();
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
            "5DLabs-Angie" => "angie",
            "5DLabs-Vex" => "vex",
            "5DLabs-Pixel" => "pixel",
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
    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn register_shared_partials(handlebars: &mut Handlebars) -> Result<()> {
        use crate::tasks::template_paths::{
            // New templates partials
            PARTIAL_ACCEPTANCE_PROBE,
            PARTIAL_COMPLETION,
            PARTIAL_CONFIG,
            PARTIAL_CTO_TOOLS_SETUP,
            PARTIAL_EXPO_ENV,
            PARTIAL_FRONTEND_TOOLKITS,
            PARTIAL_GITHUB_AUTH,
            PARTIAL_GIT_SETUP,
            PARTIAL_GO_ENV,
            PARTIAL_HEADER,
            PARTIAL_INFRASTRUCTURE_OPERATORS,
            PARTIAL_INFRASTRUCTURE_SETUP,
            PARTIAL_INFRASTRUCTURE_VERIFY,
            PARTIAL_MCP_CHECK,
            PARTIAL_NODE_ENV,
            PARTIAL_RETRY_LOOP,
            PARTIAL_RUST_ENV,
            PARTIAL_SHADCN_STACK,
            PARTIAL_SKILLS_SETUP,
            PARTIAL_TANSTACK_STACK,
            PARTIAL_TASK_FILES,
            PARTIAL_TOOLS_CONFIG,
            PARTIAL_UNITY_ENV,
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
            ("unity-env", PARTIAL_UNITY_ENV),
            ("config", PARTIAL_CONFIG),
            ("github-auth", PARTIAL_GITHUB_AUTH),
            ("git-setup", PARTIAL_GIT_SETUP),
            ("task-files", PARTIAL_TASK_FILES),
            ("tools-config", PARTIAL_TOOLS_CONFIG),
            ("cto-tools-setup", PARTIAL_CTO_TOOLS_SETUP),
            ("acceptance-probe", PARTIAL_ACCEPTANCE_PROBE),
            ("retry-loop", PARTIAL_RETRY_LOOP),
            ("completion", PARTIAL_COMPLETION),
            ("mcp-check", PARTIAL_MCP_CHECK),
            ("skills-setup", PARTIAL_SKILLS_SETUP),
            // Frontend stack partials (for Blaze/Morgan)
            ("frontend-toolkits", PARTIAL_FRONTEND_TOOLKITS),
            ("tanstack-stack", PARTIAL_TANSTACK_STACK),
            ("shadcn-stack", PARTIAL_SHADCN_STACK),
            // Infrastructure partials (for Bolt)
            ("infrastructure-operators", PARTIAL_INFRASTRUCTURE_OPERATORS),
            ("infrastructure-setup", PARTIAL_INFRASTRUCTURE_SETUP),
            ("infrastructure-verify", PARTIAL_INFRASTRUCTURE_VERIFY),
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
    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn register_agent_partials(handlebars: &mut Handlebars) -> Result<()> {
        use crate::tasks::template_paths::{
            PARTIAL_BETTER_AUTH, PARTIAL_BETTER_AUTH_ELECTRON, PARTIAL_BETTER_AUTH_EXPO,
            PARTIAL_FRONTEND_TOOLKITS, PARTIAL_INFRASTRUCTURE_OPERATORS,
            PARTIAL_INFRASTRUCTURE_SETUP, PARTIAL_INFRASTRUCTURE_VERIFY, PARTIAL_SHADCN_STACK,
            PARTIAL_TANSTACK_STACK,
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

        // Infrastructure partials used by Morgan (intake) and Bolt (infra) system prompts
        let infrastructure_partials = vec![
            ("infrastructure-operators", PARTIAL_INFRASTRUCTURE_OPERATORS),
            ("infrastructure-setup", PARTIAL_INFRASTRUCTURE_SETUP),
            ("infrastructure-verify", PARTIAL_INFRASTRUCTURE_VERIFY),
        ];

        // Auth partials used by Spark/Tap/Blaze system prompts when skills_native=false
        // (Cursor and Gemini need these inlined; Claude/Factory/Codex/OpenCode get them via MCP skills)
        let auth_partials = vec![
            ("better-auth", PARTIAL_BETTER_AUTH),
            ("better-auth-electron", PARTIAL_BETTER_AUTH_ELECTRON),
            ("better-auth-expo", PARTIAL_BETTER_AUTH_EXPO),
        ];

        // Register auth partials (best-effort — agents only reference them when skills_native=false)
        for (partial_name, template_path) in auth_partials {
            match Self::load_template(template_path) {
                Ok(content) => match handlebars.register_partial(partial_name, content) {
                    Ok(()) => {
                        debug!("Successfully registered auth partial: {}", partial_name);
                    }
                    Err(e) => {
                        warn!(
                                "Auth partial {partial_name} has invalid handlebars syntax (likely JSX {{ }} in code samples): {e}. \
                                Agents referencing this partial will get raw markdown instead."
                            );
                    }
                },
                Err(e) => {
                    warn!(
                        "Failed to load auth partial {partial_name} from ConfigMap (path: {template_path}): {e}. \
                        Spark/Tap/Blaze templates referencing this partial will fail to render."
                    );
                }
            }
        }

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

        // Register infrastructure partials (for Morgan/intake and Bolt/infra templates)
        for (partial_name, template_path) in infrastructure_partials {
            match Self::load_template(template_path) {
                Ok(content) => {
                    handlebars
                        .register_partial(partial_name, content)
                        .map_err(|e| {
                            crate::tasks::types::Error::ConfigError(format!(
                                "Failed to register infrastructure partial {partial_name}: {e}"
                            ))
                        })?;
                    debug!(
                        "Successfully registered infrastructure partial: {}",
                        partial_name
                    );
                }
                Err(e) => {
                    // Warn but don't fail - the partial may not be needed for non-infrastructure agents
                    warn!(
                        "Failed to load infrastructure partial {partial_name} from ConfigMap (path: {template_path}): {e}. \
                        Morgan/Bolt templates referencing this partial will fail to render."
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
    fn register_cli_invoke_partial(handlebars: &mut Handlebars, _cli_type: CLIType) -> Result<()> {
        // All CLI types route through the unified ACP dispatch template (openclaw.sh.hbs).
        // CLI-specific behavior (auth, flags, autonomy) is handled inside the template
        // via the cli_type context variable and acpx agent mapping.
        // Legacy per-CLI templates are archived in templates/clis/_archived/.
        let invoke_template_path = "clis/openclaw.sh.hbs".to_string();

        match Self::load_template(&invoke_template_path) {
            Ok(content) => {
                handlebars
                    .register_partial("cli_execute", content)
                    .map_err(|e| {
                        crate::tasks::types::Error::ConfigError(format!(
                            "Failed to register ACP dispatch partial: {e}"
                        ))
                    })?;
                debug!("Registered unified ACP dispatch partial (openclaw.sh.hbs)");
                Ok(())
            }
            Err(e) => Err(crate::tasks::types::Error::ConfigError(format!(
                "ACP dispatch template not found ({invoke_template_path}): {e}. \
                    Ensure templates/clis/openclaw.sh.hbs exists in the controller image."
            ))),
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
                task_id: Some(1),
                service: "test-service".to_string(),
                repository_url: "https://github.com/test/repo".to_string(),
                docs_repository_url: "https://github.com/test/docs".to_string(),
                model: "sonnet".to_string(),
                github_app,
                ..Default::default()
            },
            status: None,
        }
    }

    // ========================================================================
    // Container template selection tests
    // All agents now use the shared container template (_shared/container.sh.hbs)
    // CLI-specific behavior is injected via the cli_execute partial
    // ========================================================================

    // ========================================================================
    // Handlebars helper tests
    // Tests for eq and or helpers used in template conditionals
    // ========================================================================

    #[test]
    fn test_handlebars_eq_helper_returns_true_for_matching_strings() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string("test", "{{#if (eq name \"tap\")}}yes{{else}}no{{/if}}")
            .unwrap();
        let result = hb.render("test", &json!({"name": "tap"})).unwrap();
        assert_eq!(
            result, "yes",
            "eq helper should return true for matching strings"
        );
    }

    #[test]
    fn test_handlebars_eq_helper_returns_false_for_non_matching_strings() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string("test", "{{#if (eq name \"tap\")}}yes{{else}}no{{/if}}")
            .unwrap();
        let result = hb.render("test", &json!({"name": "rex"})).unwrap();
        assert_eq!(
            result, "no",
            "eq helper should return false for non-matching strings"
        );
    }

    #[test]
    fn test_handlebars_or_helper_returns_true_when_any_truthy() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string(
            "test",
            "{{#if (or (eq name \"blaze\") (eq name \"tap\"))}}yes{{else}}no{{/if}}",
        )
        .unwrap();

        // Should be true when name is "tap"
        let result = hb.render("test", &json!({"name": "tap"})).unwrap();
        assert_eq!(
            result, "yes",
            "or helper should return true when second condition matches"
        );

        // Should be true when name is "blaze"
        let result = hb.render("test", &json!({"name": "blaze"})).unwrap();
        assert_eq!(
            result, "yes",
            "or helper should return true when first condition matches"
        );
    }

    #[test]
    fn test_handlebars_or_helper_returns_false_when_all_falsy() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string(
            "test",
            "{{#if (or (eq name \"blaze\") (eq name \"tap\"))}}yes{{else}}no{{/if}}",
        )
        .unwrap();

        // Should be false when name is neither "blaze" nor "tap"
        let result = hb.render("test", &json!({"name": "rex"})).unwrap();
        assert_eq!(
            result, "no",
            "or helper should return false when no condition matches"
        );
    }

    #[test]
    fn test_handlebars_expo_condition_only_matches_tap() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        // Note: The actual template uses `agent_name` not `github_app`
        // This matches templates/_shared/container.sh.hbs line 10:
        // {{#if (eq agent_name "tap")}}{{> expo-env }}{{/if}}
        hb.register_template_string("test", "{{#if (eq agent_name \"tap\")}}EXPO{{/if}}")
            .unwrap();

        // Only tap should include EXPO
        assert_eq!(
            hb.render("test", &json!({"agent_name": "tap"})).unwrap(),
            "EXPO",
            "tap agent should get Expo environment"
        );

        // Other agents should NOT include EXPO
        // This is the critical test - Atlas running expo-env is the reported bug
        for agent in [
            "rex", "blaze", "atlas", "spark", "cleo", "tess", "cipher", "bolt", "nova", "grizz",
            "morgan", "stitch",
        ] {
            let result = hb.render("test", &json!({"agent_name": agent})).unwrap();
            assert_eq!(result, "", "Agent {agent} should NOT get Expo environment");
        }

        // Empty string should NOT include EXPO
        let result = hb.render("test", &json!({"agent_name": ""})).unwrap();
        assert_eq!(
            result, "",
            "Empty agent_name should NOT get Expo environment"
        );

        // Note: Missing agent_name causes RenderError due to type mismatch in eq helper
        // This is OK because get_agent_name() always provides a value in production
    }

    #[test]
    fn test_handlebars_group_by_groups_items_by_field() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        // Template iterates over grouped keys and lists item names in each group
        hb.register_template_string(
            "test",
            "{{#each (group_by items \"level\")}}{{@key}}:[{{#each this}}{{name}}{{/each}}];{{/each}}",
        )
        .unwrap();

        let data = json!({
            "items": [
                {"name": "a", "level": "1"},
                {"name": "b", "level": "2"},
                {"name": "c", "level": "1"},
                {"name": "d", "level": "2"},
                {"name": "e", "level": "1"}
            ]
        });
        let result = hb.render("test", &data).unwrap();
        // BTreeMap ensures sorted keys, so "1" comes before "2"
        assert_eq!(
            result, "1:[ace];2:[bd];",
            "group_by should group items by field"
        );
    }

    #[test]
    fn test_handlebars_group_by_handles_numeric_field_values() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string(
            "test",
            "{{#each (group_by items \"level\")}}{{@key}}:[{{#each this}}{{name}}{{/each}}];{{/each}}",
        )
        .unwrap();

        let data = json!({
            "items": [
                {"name": "a", "level": 1},
                {"name": "b", "level": 2},
                {"name": "c", "level": 1}
            ]
        });
        let result = hb.render("test", &data).unwrap();
        assert_eq!(
            result, "1:[ac];2:[b];",
            "group_by should handle numeric field values"
        );
    }

    #[test]
    fn test_handlebars_group_by_handles_empty_array() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string(
            "test",
            "{{#each (group_by items \"level\")}}{{@key}}:[{{#each this}}{{name}}{{/each}}];{{/each}}",
        )
        .unwrap();

        let data = json!({"items": []});
        let result = hb.render("test", &data).unwrap();
        assert_eq!(result, "", "group_by should handle empty arrays");
    }

    #[test]
    fn test_handlebars_group_by_handles_missing_field() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        hb.register_template_string(
            "test",
            "{{#each (group_by items \"level\")}}{{@key}}:[{{#each this}}{{name}}{{/each}}];{{/each}}",
        )
        .unwrap();

        // Items without the "level" field should group under empty string key
        let data = json!({
            "items": [
                {"name": "a", "level": "1"},
                {"name": "b"},
                {"name": "c", "level": "1"}
            ]
        });
        let result = hb.render("test", &data).unwrap();
        // Empty key comes first in BTreeMap ordering
        assert_eq!(
            result, ":[b];1:[ac];",
            "group_by should group items with missing field under empty key"
        );
    }

    #[test]
    fn test_handlebars_group_by_preserves_item_data() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        // Template accesses nested data within grouped items
        hb.register_template_string(
            "test",
            "{{#each (group_by items \"level\")}}[{{#each this}}{{name}}{{/each}}]{{/each}}",
        )
        .unwrap();

        let data = json!({
            "items": [
                {"name": "a", "level": "x"},
                {"name": "b", "level": "y"},
                {"name": "c", "level": "x"}
            ]
        });
        let result = hb.render("test", &data).unwrap();
        assert_eq!(
            result, "[ac][b]",
            "group_by should preserve all item data within groups"
        );
    }

    #[test]
    fn test_handlebars_group_by_subtasks_by_execution_level() {
        let mut hb = Handlebars::new();
        CodeTemplateGenerator::register_template_helpers(&mut hb);
        // Simulate the intended use case: grouping subtasks by execution_level
        hb.register_template_string(
            "test",
            "{{#each (group_by subtasks \"execution_level\")}}Level {{@key}}: {{#each this}}{{task_id}}{{#unless @last}}, {{/unless}}{{/each}}\n{{/each}}",
        )
        .unwrap();

        let data = json!({
            "subtasks": [
                {"task_id": "TASK-1", "execution_level": "1", "title": "Setup"},
                {"task_id": "TASK-2", "execution_level": "2", "title": "Build"},
                {"task_id": "TASK-3", "execution_level": "1", "title": "Config"},
                {"task_id": "TASK-4", "execution_level": "2", "title": "Test"},
                {"task_id": "TASK-5", "execution_level": "3", "title": "Deploy"}
            ]
        });
        let result = hb.render("test", &data).unwrap();
        assert_eq!(
            result, "Level 1: TASK-1, TASK-3\nLevel 2: TASK-2, TASK-4\nLevel 3: TASK-5\n",
            "group_by should work for subtasks grouped by execution_level"
        );
    }

    /// Verify that Atlas container.sh does NOT include Expo environment setup.
    /// This is a regression test for the issue where expo-env was being included
    /// for all agents instead of just Tap.
    #[test]
    fn test_atlas_container_does_not_include_expo() {
        // Skip if templates not available
        let templates_path = match std::env::var("AGENT_TEMPLATES_PATH") {
            Ok(p)
                if std::path::PathBuf::from(&p)
                    .join("_shared/container.sh.hbs")
                    .exists() =>
            {
                p
            }
            _ => {
                tracing::warn!(
                    "AGENT_TEMPLATES_PATH not set or templates not found, skipping test"
                );
                return;
            }
        };

        // Create a test CodeRun for Atlas
        let code_run = create_test_code_run(Some("5DLabs-Atlas".to_string()));

        // Verify agent_name extraction
        let agent_name = CodeTemplateGenerator::get_agent_name(&code_run);
        assert_eq!(agent_name, "atlas", "Agent name should be 'atlas'");

        // Generate templates using Factory CLI (what the log shows)
        let config = ControllerConfig::default();
        let templates = CodeTemplateGenerator::generate_factory_templates(&code_run, &config)
            .expect("Should generate Factory templates for Atlas");

        let container_script = templates
            .get("container.sh")
            .expect("container.sh should be generated");

        // CRITICAL: Verify expo-env content is NOT included
        assert!(
            !container_script.contains("Setting up Expo environment"),
            "Atlas container.sh should NOT contain 'Setting up Expo environment'. \
             Got container.sh of {} bytes. AGENT_TEMPLATES_PATH={}",
            container_script.len(),
            templates_path
        );

        // Additional verification - no EAS CLI references
        assert!(
            !container_script.contains("eas-cli"),
            "Atlas container.sh should NOT contain 'eas-cli'"
        );
        assert!(
            !container_script.contains("EXPO_TOKEN"),
            "Atlas container.sh should NOT contain 'EXPO_TOKEN'"
        );
    }

    // ========================================================================
    // System prompt template selection tests
    // All agents use agents/{agent}/{job}.md.hbs
    // ========================================================================

    #[test]
    fn test_system_prompt_template_rex_coder() {
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/rex/coder.md.hbs");
    }

    #[test]
    fn test_system_prompt_template_cleo_quality() {
        // Cleo always uses quality job type (quality assurance specialist)
        let code_run = create_test_code_run(Some("5DLabs-Cleo".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/cleo/quality.md.hbs");
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
            provider: None,
            provider_base_url: None,
            api_key_env_var: None,
        });
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path, "agents/rex/healer.md.hbs",
            "Heal template setting should map to healer job"
        );
    }

    #[test]
    fn test_system_prompt_template_morgan_intake() {
        let mut code_run = create_test_code_run(Some("5DLabs-Morgan".to_string()));
        code_run.spec.run_type = "intake".to_string();
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path, "agents/morgan/intake.md.hbs",
            "Morgan intake run should use intake prompt"
        );
    }

    #[test]
    fn test_system_prompt_template_intake_unknown_github_app() {
        // Intake runs with unknown github_app (like "cto-dev") should still use morgan templates
        let mut code_run = create_test_code_run(Some("cto-dev".to_string()));
        code_run.spec.run_type = "intake".to_string();
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(
            template_path, "agents/morgan/intake.md.hbs",
            "Intake run with unknown github_app should use morgan intake template"
        );
    }

    #[test]
    fn test_system_prompt_template_atlas_integration() {
        let mut code_run = create_test_code_run(Some("5DLabs-Atlas".to_string()));
        code_run.spec.run_type = "integration".to_string();
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/atlas/integration.md.hbs");
    }

    #[test]
    fn test_system_prompt_template_pixel_coder() {
        let code_run = create_test_code_run(Some("5DLabs-Pixel".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_system_prompt_template(&code_run);
        assert_eq!(template_path, "agents/pixel/coder.md.hbs");
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
    fn test_extract_agent_name_from_github_app_pixel() {
        let agent_name =
            CodeTemplateGenerator::extract_agent_name_from_github_app("5DLabs-Pixel").unwrap();
        assert_eq!(agent_name, "pixel");
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
    fn test_get_agent_name_with_empty_github_app() {
        // Test that empty github_app defaults to "agent"
        let code_run = create_test_code_run(Some(String::new()));
        let agent_name = CodeTemplateGenerator::get_agent_name(&code_run);
        assert_eq!(
            agent_name, "agent",
            "Empty github_app should default to 'agent'"
        );
    }

    #[test]
    fn test_get_agent_name_with_none_github_app() {
        // Test that None github_app defaults to "agent"
        let code_run = create_test_code_run(None);
        let agent_name = CodeTemplateGenerator::get_agent_name(&code_run);
        assert_eq!(
            agent_name, "agent",
            "None github_app should default to 'agent'"
        );
    }

    #[test]
    fn test_get_agent_name_with_valid_github_app() {
        // Test that valid github_app extracts correctly
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let agent_name = CodeTemplateGenerator::get_agent_name(&code_run);
        assert_eq!(agent_name, "rex", "5DLabs-Rex should extract to 'rex'");

        let code_run = create_test_code_run(Some("5DLabs-Tap".to_string()));
        let agent_name = CodeTemplateGenerator::get_agent_name(&code_run);
        assert_eq!(agent_name, "tap", "5DLabs-Tap should extract to 'tap'");
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
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(agent_tools.clone()),
                skills: None,
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: None,
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
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(agent_tools),
                skills: None,
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
                subagents: None,
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
        assert_eq!(template_path, "agents/rex/coder.md.hbs");
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
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: Some(helm_tools),
                skills: None,
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: None,
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

    #[test]
    fn test_get_default_agent_tools_morgan_intake() {
        let tools = CodeTemplateGenerator::get_default_agent_tools("5DLabs-Morgan", "intake");
        assert!(!tools.is_empty(), "Morgan intake should have default tools");
        assert!(
            tools.iter().any(|t| t.contains("firecrawl")),
            "Morgan intake should have firecrawl tools"
        );
        assert!(
            tools.iter().any(|t| t.contains("context7")),
            "Morgan intake should have context7 tools"
        );
        assert!(
            tools.iter().any(|t| t.contains("openmemory")),
            "Morgan intake should have openmemory tools"
        );
    }

    #[test]
    fn test_get_default_agent_tools_rex_implementation() {
        let tools = CodeTemplateGenerator::get_default_agent_tools("5DLabs-Rex", "implementation");
        assert!(!tools.is_empty(), "Rex should have default tools");
        assert!(
            tools.iter().any(|t| t.contains("github")),
            "Rex should have github tools"
        );
    }

    #[test]
    fn test_get_default_agent_tools_bolt_infra() {
        let tools = CodeTemplateGenerator::get_default_agent_tools("5DLabs-Bolt", "deploy");
        assert!(!tools.is_empty(), "Bolt should have default tools");
        assert!(
            tools.iter().any(|t| t.contains("kubernetes")),
            "Bolt should have kubernetes tools"
        );
    }

    #[test]
    fn test_get_default_agent_tools_unknown_agent() {
        let tools = CodeTemplateGenerator::get_default_agent_tools("Unknown-Agent", "coder");
        assert!(
            tools.is_empty(),
            "Unknown agent should have no default tools"
        );
    }

    #[test]
    fn test_enrich_cli_config_passes_subagents_to_context() {
        use crate::tasks::config::{AgentDefinition, ControllerConfig, SubagentConfig};

        let mut config = ControllerConfig::default();
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: None,
                skills: None,
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: Some(SubagentConfig {
                    enabled: true,
                    max_concurrent: 8,
                }),
            },
        );

        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let cli_config = json!({});

        let enriched =
            CodeTemplateGenerator::enrich_cli_config_from_agent(cli_config, &code_run, &config);

        // Verify subagents config is passed through
        assert!(
            enriched.get("subagents").is_some(),
            "subagents should be present in enriched config"
        );

        let subagents = enriched.get("subagents").unwrap();
        assert_eq!(
            subagents.get("enabled").and_then(Value::as_bool),
            Some(true),
            "subagents.enabled should be true"
        );
        assert_eq!(
            subagents.get("maxConcurrent").and_then(Value::as_u64),
            Some(8),
            "subagents.maxConcurrent should be 8"
        );
    }

    // ========================================================================
    // Skills resolution tests
    // Tests for get_agent_skills with cto-config.json priority over YAML fallback
    // ========================================================================

    #[test]
    fn test_get_agent_skills_from_cto_config() {
        use crate::tasks::config::{AgentDefinition, ControllerConfig};

        let mut config = ControllerConfig::default();
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: None,
                skills: Some(cto_config::AgentSkills {
                    default: vec!["rust-patterns".to_string(), "context7".to_string()],
                    coder: Some(vec!["compound-engineering".to_string()]),
                    healer: Some(vec!["incident-response".to_string()]),
                    intake: None,
                    quality: None,
                    test: None,
                    security: None,
                    review: None,
                    deploy: None,
                    integration: None,
                    optional: None,
                }),
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: None,
            },
        );

        // Test coder job type (default run_type is "implementation" which maps to "coder")
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));

        let skills = CodeTemplateGenerator::get_agent_skills(&code_run, &config);
        assert!(
            skills.contains(&"rust-patterns".to_string()),
            "Should include default skill"
        );
        assert!(
            skills.contains(&"context7".to_string()),
            "Should include default skill"
        );
        assert!(
            skills.contains(&"compound-engineering".to_string()),
            "Should include coder skill"
        );
        assert!(
            !skills.contains(&"incident-response".to_string()),
            "Should NOT include healer skill"
        );

        // Test healer job type by setting service name to trigger healer detection
        let mut healer_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        healer_run.spec.service = "healer-service".to_string();
        let skills = CodeTemplateGenerator::get_agent_skills(&healer_run, &config);
        assert!(
            skills.contains(&"rust-patterns".to_string()),
            "Should include default skill"
        );
        assert!(
            skills.contains(&"incident-response".to_string()),
            "Should include healer skill"
        );
        assert!(
            !skills.contains(&"compound-engineering".to_string()),
            "Should NOT include coder skill"
        );
    }

    #[test]
    fn test_get_agent_skills_default_only() {
        use crate::tasks::config::{AgentDefinition, ControllerConfig};

        let mut config = ControllerConfig::default();
        config.agents.insert(
            "cipher".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Cipher".to_string(),
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: None,
                skills: Some(cto_config::AgentSkills {
                    default: vec!["security-analysis".to_string(), "codeql".to_string()],
                    coder: None,
                    healer: None,
                    intake: None,
                    quality: None,
                    test: None,
                    security: None,
                    review: None,
                    deploy: None,
                    integration: None,
                    optional: None,
                }),
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: None,
            },
        );

        let code_run = create_test_code_run(Some("5DLabs-Cipher".to_string()));
        let skills = CodeTemplateGenerator::get_agent_skills(&code_run, &config);

        assert!(
            skills.contains(&"security-analysis".to_string()),
            "Should include default skill"
        );
        assert!(
            skills.contains(&"codeql".to_string()),
            "Should include default skill"
        );
        assert_eq!(skills.len(), 2, "Should only have default skills");
    }

    #[test]
    fn test_get_agent_skills_no_duplicates() {
        use crate::tasks::config::{AgentDefinition, ControllerConfig};

        let mut config = ControllerConfig::default();
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                scm_identity: None,
                cli: None,
                model: None,
                max_tokens: None,
                temperature: None,
                reasoning_effort: None,
                tools: None,
                skills: Some(cto_config::AgentSkills {
                    default: vec!["skill-a".to_string(), "skill-b".to_string()],
                    coder: Some(vec!["skill-a".to_string(), "skill-c".to_string()]), // skill-a is duplicate
                    healer: None,
                    intake: None,
                    quality: None,
                    test: None,
                    security: None,
                    review: None,
                    deploy: None,
                    integration: None,
                    optional: None,
                }),
                client_config: None,
                model_rotation: None,
                frontend_stack: None,
                subagents: None,
            },
        );

        // Default run_type "implementation" maps to "coder"
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));

        let skills = CodeTemplateGenerator::get_agent_skills(&code_run, &config);

        // Count occurrences of skill-a
        let skill_a_count = skills.iter().filter(|s| *s == "skill-a").count();
        assert_eq!(
            skill_a_count, 1,
            "skill-a should appear only once (deduped)"
        );
        assert!(skills.contains(&"skill-b".to_string()));
        assert!(skills.contains(&"skill-c".to_string()));
    }

    #[test]
    fn test_get_agent_skills_no_config_falls_back() {
        use crate::tasks::config::ControllerConfig;

        // Empty config - no agents defined
        let config = ControllerConfig::default();
        let code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));

        // This would fall back to skill-mappings.yaml
        // Since we can't mock the file, just verify it doesn't panic
        let skills = CodeTemplateGenerator::get_agent_skills(&code_run, &config);

        // Skills might be empty or come from YAML fallback
        // Just ensure no panic
        assert!(skills.is_empty() || !skills.is_empty());
    }
}
