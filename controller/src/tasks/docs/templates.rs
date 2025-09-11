use crate::crds::DocsRun;
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tracing::debug;

// Template base path (mounted from ConfigMap)
const CLAUDE_TEMPLATES_PATH: &str = "/claude-templates";

pub struct DocsTemplateGenerator;

impl DocsTemplateGenerator {
    /// Generate all template files for a docs task
    pub fn generate_all_templates(
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

        // Agent-centric ToolMan config for docs: generate base client-config.json
        // (Repo-specific cto-config.json can append at runtime in the container script.)
        templates.insert(
            "client-config.json".to_string(),
            Self::generate_client_config(docs_run, config)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(docs_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_container_script(docs_run: &DocsRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("docs/container.sh.hbs")?;

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

        let template = Self::load_template("docs/claude.md.hbs")?;

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

        let template = Self::load_template("docs/settings.json.hbs")?;

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

        let template = Self::load_template("docs/prompt.md.hbs")?;

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

    /// Generate agent-centric client-config.json for DocsRun based on Helm (base) config.
    fn generate_client_config(docs_run: &DocsRun, config: &ControllerConfig) -> Result<String> {
        use serde_json::{json, Value};

        // Find the matching agent definition by github_app (e.g., 5DLabs-Morgan)
        let github_app = docs_run.spec.github_app.as_deref().unwrap_or("");
        let mut selected_tools: Option<crate::tasks::config::AgentTools> = None;

        for (_name, def) in &config.agents {
            if def.github_app == github_app {
                if let Some(tools) = &def.tools {
                    selected_tools = Some(crate::tasks::config::AgentTools {
                        remote: tools.remote.clone(),
                        local_servers: tools.local_servers.clone(),
                    });
                }
                break;
            }
        }

        // Fallback defaults if not configured
        let agent_tools = selected_tools.unwrap_or_else(|| crate::tasks::config::AgentTools {
            remote: vec![
                "memory_create_entities".to_string(),
                "memory_add_observations".to_string(),
            ],
            local_servers: Some(crate::tasks::config::LocalServerConfigs {
                filesystem: crate::tasks::config::LocalServerConfig {
                    enabled: true,
                    tools: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "list_directory".to_string(),
                        "search_files".to_string(),
                        "directory_tree".to_string(),
                    ],
                },
                git: crate::tasks::config::LocalServerConfig {
                    enabled: true,
                    tools: vec![
                        "git_status".to_string(),
                        "git_diff".to_string(),
                        "git_log".to_string(),
                        "git_show".to_string(),
                    ],
                },
            }),
        });

        // Build the client-config.json structure
        let mut client_config = json!({
            "remoteTools": agent_tools.remote,
            "localServers": {}
        });

        if let Some(local_servers) = agent_tools.local_servers {
            let mut local_servers_obj = serde_json::Map::new();

            if local_servers.filesystem.enabled {
                let fs_cmd = local_servers
                    .filesystem
                    .command
                    .clone()
                    .unwrap_or_else(|| "npx".to_string());
                let fs_args = local_servers
                    .filesystem
                    .args
                    .clone()
                    .unwrap_or_else(|| vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/workspace".to_string(),
                    ]);
                let fs_workdir = local_servers
                    .filesystem
                    .working_directory
                    .clone()
                    .unwrap_or_else(|| "project_root".to_string());
                let filesystem_server = json!({
                    "command": fs_cmd,
                    "args": fs_args,
                    "tools": local_servers.filesystem.tools,
                    "workingDirectory": fs_workdir
                });
                local_servers_obj.insert("filesystem".to_string(), filesystem_server);
            }

            if local_servers.git.enabled {
                let git_cmd = local_servers.git.command.clone().unwrap_or_else(|| "npx".to_string());
                let git_args = local_servers.git.args.clone().unwrap_or_else(|| vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-git".to_string(),
                    "/workspace".to_string(),
                ]);
                let git_workdir = local_servers
                    .git
                    .working_directory
                    .clone()
                    .unwrap_or_else(|| "project_root".to_string());
                let git_server = json!({
                    "command": git_cmd,
                    "args": git_args,
                    "tools": local_servers.git.tools,
                    "workingDirectory": git_workdir
                });
                local_servers_obj.insert("git".to_string(), git_server);
            }

            client_config["localServers"] = Value::Object(local_servers_obj);
        }

        serde_json::to_string_pretty(&client_config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize client-config.json: {e}"
            ))
        })
    }

    fn generate_hook_scripts(docs_run: &DocsRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "docs_hooks_";

        debug!(
            "Scanning for docs hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for docs
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

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
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        Ok(hook_scripts)
    }

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
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
