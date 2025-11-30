// Allow deprecated DocsRun usage for backwards compatibility
// This module provides templates for the deprecated DocsRun CRD
// Users should migrate to CodeRun with runType: "documentation"
#![allow(deprecated)]

use crate::crds::DocsRun;
use crate::tasks::config::ControllerConfig;
use crate::tasks::template_paths::{
    CODE_MCP_CONFIG_TEMPLATE, DOCS_CLAUDE_CONTAINER_TEMPLATE, DOCS_CLAUDE_MEMORY_TEMPLATE,
    DOCS_CLAUDE_PROMPT_TEMPLATE, DOCS_CLAUDE_SETTINGS_TEMPLATE,
};
use crate::tasks::tool_catalog::resolve_tool_name;
use crate::tasks::types::Result;
use handlebars::Handlebars;
use serde_json::{json, to_string_pretty, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

// Template base path (mounted from ConfigMap)
const AGENT_TEMPLATES_PATH: &str = "/agent-templates";

pub struct DocsTemplateGenerator;

impl DocsTemplateGenerator {
    /// Generate all template files for a docs task
    ///
    /// # Errors
    /// Returns error if template generation fails
    pub fn generate_all_templates(
        docs_run: &DocsRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        // All CLIs currently use Claude templates
        Self::generate_claude_templates(docs_run, config)
    }

    fn generate_claude_templates(
        docs_run: &DocsRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core docs templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(docs_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(docs_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(docs_run, config)?,
        );
        templates.insert(
            "prompt.md".to_string(),
            Self::generate_docs_prompt(docs_run)?,
        );

        // Add MCP servers config to enable Tools (reuse code/mcp.json.hbs)
        templates.insert("mcp.json".to_string(), Self::generate_mcp_config()?);

        // Agent-centric Tools config for docs: generate base client-config.json
        // (Repo-specific cto-config.json can append at runtime in the container script.)
        templates.insert(
            "client-config.json".to_string(),
            Self::generate_client_config(docs_run, config)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(docs_run);
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_container_script(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(DOCS_CLAUDE_CONTAINER_TEMPLATE)?;

        handlebars
            .register_template_string("container_script", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register container script template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "service_name": "docs-generator",
            "include_codebase": docs_run.spec.include_codebase.unwrap_or(false)
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(DOCS_CLAUDE_MEMORY_TEMPLATE)?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "model": docs_run.spec.model.as_deref().unwrap_or(""),
            "service_name": "docs-generator"
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(docs_run: &DocsRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(DOCS_CLAUDE_SETTINGS_TEMPLATE)?;

        handlebars
            .register_template_string("claude_settings", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register settings.json template: {e}"
                ))
            })?;

        // Debug logging to trace model value
        let model_value = docs_run.spec.model.as_deref().unwrap_or("");
        tracing::info!(
            "🐛 DEBUG: DocsRun template - model from spec: {:?}",
            docs_run.spec.model
        );
        tracing::info!(
            "🐛 DEBUG: DocsRun template - model value for template: {}",
            model_value
        );

        let context = json!({
            "model": model_value,
            "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
            "api_key_secret_name": config.secrets.api_key_secret_name,
            "api_key_secret_key": config.secrets.api_key_secret_key,
            "working_directory": &docs_run.spec.working_directory
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render settings.json: {e}"))
        })
    }

    fn generate_docs_prompt(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template(DOCS_CLAUDE_PROMPT_TEMPLATE)?;

        handlebars
            .register_template_string("docs_prompt", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register docs prompt template: {e}"
                ))
            })?;

        let context = json!({
            "repository_url": docs_run.spec.repository_url,
            "source_branch": docs_run.spec.source_branch,
            "working_directory": docs_run.spec.working_directory,
            "service_name": "docs-generator",
            "include_codebase": docs_run.spec.include_codebase.unwrap_or(false)
        });

        handlebars.render("docs_prompt", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render docs prompt: {e}"))
        })
    }

    fn generate_mcp_config() -> Result<String> {
        // Reuse the code template to avoid duplication in the templates ConfigMap
        Self::load_template(CODE_MCP_CONFIG_TEMPLATE)
    }

    /// Generate agent-centric client-config.json for `DocsRun`.
    /// Precedence:
    /// 1) agents.<agent>.clientConfig (verbatim pass-through)
    /// 2) agents.<agent>.tools (convert to client-config.json structure generically)
    /// 3) fallback to empty object {}
    ///
    /// # Errors
    /// Returns error if config generation fails
    #[allow(clippy::too_many_lines)]
    fn generate_client_config(docs_run: &DocsRun, config: &ControllerConfig) -> Result<String> {
        let github_app = docs_run.spec.github_app.as_deref().unwrap_or("");
        debug!(
            "docs: generating client-config.json for githubApp='{}'",
            github_app
        );

        // Helpful visibility: list known agents (names -> githubApp)
        if config.agents.is_empty() {
            debug!(
                "docs: config.agents is EMPTY; client-config will fall back to '{}'",
                "{}"
            );
        } else {
            for (k, a) in &config.agents {
                debug!(
                    "docs: available agent entry key='{}' githubApp='{}'",
                    k, a.github_app
                );
            }
        }

        if let Some(agent_cfg) = config.agents.values().find(|a| a.github_app == github_app) {
            debug!(
                "docs: matched agent config for githubApp='{}' (tools_present={}, clientConfig_present={})",
                github_app,
                agent_cfg.tools.is_some(),
                agent_cfg.client_config.is_some()
            );
            // 1) Verbatim clientConfig
            if let Some(client_cfg) = &agent_cfg.client_config {
                debug!("docs: using verbatim clientConfig for '{}'", github_app);
                return to_string_pretty(client_cfg).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to serialize clientConfig: {e}"
                    ))
                });
            }

            // 2) Convert tools → client-config.json
            if let Some(tools) = &agent_cfg.tools {
                debug!(
                    "docs: building clientConfig from tools for '{}' (remote_count={}, local_present={})",
                    github_app,
                    tools.remote.len(),
                    tools.local_servers.is_some()
                );
                // remoteTools - tools.remote is Vec<String>, not Option
                let remote_tools: Value = json!(tools.remote);

                // localServers (generic: include only servers marked enabled)
                let mut local_servers_obj = serde_json::Map::new();
                if let Some(ref servers) = tools.local_servers {
                    for (name, cfg) in servers {
                        if cfg.enabled {
                            let mut obj = serde_json::Map::new();
                            if !cfg.tools.is_empty() {
                                obj.insert("tools".to_string(), json!(cfg.tools));
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
                // 2b) Merge CRD-provided extras (remoteTools/localTools)
                if let Some(extras) = &docs_run.spec.remote_tools {
                    let mut base = client
                        .get("remoteTools")
                        .cloned()
                        .unwrap_or_else(|| json!([]));
                    if let Some(arr) = base.as_array_mut() {
                        for t in extras {
                            if !arr.iter().any(|v| v == t) {
                                arr.push(json!(t));
                            }
                        }
                    }
                    client["remoteTools"] = base;
                }
                if let Some(local_list) = &docs_run.spec.local_tools {
                    // If a selection is provided, restrict baseline localServers to that subset.
                    if let Some(ls_value) = client.get_mut("localServers") {
                        if let Some(ls_obj) = ls_value.as_object_mut() {
                            let requested: HashSet<String> = local_list.iter().cloned().collect();

                            // Build filtered map of only requested servers that exist in baseline
                            let mut filtered = serde_json::Map::new();
                            for (k, v) in ls_obj.iter() {
                                if requested.contains(k) {
                                    filtered.insert(k.clone(), v.clone());
                                }
                            }

                            // Log any requested servers that do not exist in baseline
                            for name in local_list {
                                if !ls_obj.contains_key(name) {
                                    debug!(
                                        "docs: requested localTool '{}' not present in agent baseline; skipping",
                                        name
                                    );
                                }
                            }

                            *ls_obj = filtered;
                        }
                    }
                }

                Self::normalize_remote_tools(&mut client);
                let rendered = to_string_pretty(&client).map_err(|e| {
                    crate::tasks::types::Error::ConfigError(format!(
                        "Failed to serialize tools-based clientConfig: {e}"
                    ))
                })?;
                debug!(
                    "docs: generated client-config.json ({} bytes) for '{}'",
                    rendered.len(),
                    github_app
                );
                return Ok(rendered);
            }
        }

        // 3) No clientConfig/tools provided → minimal JSON object (plus CRD extras if any)
        debug!(
            "docs: no matching agent or tools for githubApp='{}' → returning empty client-config",
            github_app
        );
        let mut client = json!({});
        if let Some(extras) = &docs_run.spec.remote_tools {
            client["remoteTools"] = json!(extras);
        }
        if let Some(local_list) = &docs_run.spec.local_tools {
            let mut ls = serde_json::Map::new();
            for name in local_list {
                ls.insert(name.clone(), json!({}));
            }
            client["localServers"] = Value::Object(ls);
        }
        Self::normalize_remote_tools(&mut client);
        to_string_pretty(&client).map_err(|e| {
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
                    None => {
                        dropped.push(name.to_string());
                    }
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

    /// # Errors
    /// Returns error if hook script generation fails
    fn generate_hook_scripts(docs_run: &DocsRun) -> BTreeMap<String, String> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefixes = vec![
            "docs_claude_hooks_".to_string(),
            "docs_shared_hooks_".to_string(),
            "docs_hooks_".to_string(),
        ];

        debug!(
            prefixes = ?hooks_prefixes,
            "Scanning for docs hook templates"
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(AGENT_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for docs
                            if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                                if let Some(prefix) = hooks_prefixes
                                    .iter()
                                    .find(|prefix| filename.starts_with(prefix.as_str()))
                                {
                                    // Extract just the hook filename (remove prefix)
                                    let hook_name =
                                        filename.strip_prefix(prefix).unwrap_or(filename);

                                    match std::fs::read_to_string(&path) {
                                        Ok(template_content) => {
                                            debug!(
                                                "Loaded docs hook template: {} (from {})",
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
                                                "repository_url": docs_run.spec.repository_url,
                                                "source_branch": docs_run.spec.source_branch,
                                                "working_directory": docs_run.spec.working_directory,
                                                "github_app": docs_run.spec.github_app.as_deref().unwrap_or(""),
                                                "service_name": "docs-generator"
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
                                                        "Failed to render docs hook script {}: {}",
                                                        hook_name, e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            debug!(
                                                "Failed to load docs hook template {}: {}",
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

    /// Load a template file from the mounted `ConfigMap`
    ///
    /// # Errors
    /// Returns error if template loading fails
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(AGENT_TEMPLATES_PATH).join(&configmap_key);
        debug!(
            "Loading docs template from: {} (key: {})",
            full_path.display(),
            configmap_key
        );

        fs::read_to_string(&full_path).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to load docs template {relative_path} (key: {configmap_key}): {e}"
            ))
        })
    }
}
