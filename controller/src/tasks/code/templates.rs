use crate::crds::CodeRun;
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

pub struct CodeTemplateGenerator;

impl CodeTemplateGenerator {
    /// Generate all template files for a code task
    pub fn generate_all_templates(
        code_run: &CodeRun,
        config: &ControllerConfig,
    ) -> Result<BTreeMap<String, String>> {
        let mut templates = BTreeMap::new();

        // Generate core code templates
        templates.insert(
            "container.sh".to_string(),
            Self::generate_container_script(code_run)?,
        );
        templates.insert(
            "CLAUDE.md".to_string(),
            Self::generate_claude_memory(code_run)?,
        );
        templates.insert(
            "settings.json".to_string(),
            Self::generate_claude_settings(code_run, config)?,
        );

        // Generate code-specific templates
        templates.insert(
            "mcp.json".to_string(),
            Self::generate_mcp_config(code_run, config)?,
        );

        // Generate agent-specific client-config.json
        templates.insert(
            "client-config.json".to_string(),
            Self::generate_client_config(code_run, config)?,
        );

        templates.insert(
            "coding-guidelines.md".to_string(),
            Self::generate_coding_guidelines(code_run)?,
        );
        templates.insert(
            "github-guidelines.md".to_string(),
            Self::generate_github_guidelines(code_run)?,
        );

        // Generate hook scripts
        let hook_scripts = Self::generate_hook_scripts(code_run)?;
        for (filename, content) in hook_scripts {
            // Use hooks- prefix to comply with ConfigMap key constraints
            templates.insert(format!("hooks-{filename}"), content);
        }

        Ok(templates)
    }

    fn generate_container_script(code_run: &CodeRun) -> Result<String> {
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
                Self::load_template("code/container.sh.hbs")?
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

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "continue_session": Self::get_continue_session(code_run),
            "overwrite_memory": code_run.spec.overwrite_memory,
            "docs_project_directory": code_run.spec.docs_project_directory.as_deref().unwrap_or(""),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
        });

        handlebars
            .render("container_script", &context)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to render container script: {e}"
                ))
            })
    }

    fn generate_claude_memory(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/claude.md.hbs")?;

        handlebars
            .register_template_string("claude_memory", template)
            .map_err(|e| {
                crate::tasks::types::Error::ConfigError(format!(
                    "Failed to register CLAUDE.md template: {e}"
                ))
            })?;

        let context = json!({
            "task_id": code_run.spec.task_id,
            "service": code_run.spec.service,
            "repository_url": code_run.spec.repository_url,
            "docs_repository_url": code_run.spec.docs_repository_url,
            "docs_branch": code_run.spec.docs_branch,
            "working_directory": Self::get_working_directory(code_run),
            "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
            "model": code_run.spec.model,
            "context_version": code_run.spec.context_version,
        });

        handlebars.render("claude_memory", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render CLAUDE.md: {e}"))
        })
    }

    fn generate_claude_settings(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/settings.json.hbs")?;

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
            "working_directory": code_run.spec.working_directory.as_deref().unwrap_or(".")
        });

        handlebars.render("claude_settings", &context).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!("Failed to render settings.json: {e}"))
        })
    }

    fn generate_mcp_config(_code_run: &CodeRun, _config: &ControllerConfig) -> Result<String> {
        // MCP config is currently static, so just load and return the template content
        Self::load_template("code/mcp.json.hbs")
    }

    fn generate_client_config(code_run: &CodeRun, config: &ControllerConfig) -> Result<String> {
        use serde_json::{json, Value};

        // Extract agent name from GitHub app
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");
        let agent_name = Self::extract_agent_name_from_github_app(github_app)?;

        // Get agent tool configuration from controller config
        let agent_tools = Self::get_agent_tools(&agent_name, config)?;

        // Build the client-config.json structure
        let mut client_config = json!({
            "remoteTools": agent_tools.remote,
            "localServers": {}
        });

        // Add local servers based on agent configuration
        if let Some(local_servers) = agent_tools.local_servers {
            let mut local_servers_obj = serde_json::Map::new();

            // Add filesystem server if enabled
            if local_servers.filesystem.enabled {
                let filesystem_server = json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
                    "tools": local_servers.filesystem.tools,
                    "workingDirectory": "project_root"
                });
                local_servers_obj.insert("filesystem".to_string(), filesystem_server);
            }

            // Add git server if enabled
            if local_servers.git.enabled {
                let git_server = json!({
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-git", "/workspace"],
                    "tools": local_servers.git.tools,
                    "workingDirectory": "project_root"
                });
                local_servers_obj.insert("git".to_string(), git_server);
            }

            client_config["localServers"] = Value::Object(local_servers_obj);
        }

        // Serialize to pretty-printed JSON
        serde_json::to_string_pretty(&client_config).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize client-config.json: {e}"
            ))
        })
    }

    fn generate_coding_guidelines(code_run: &CodeRun) -> Result<String> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        let template = Self::load_template("code/coding-guidelines.md.hbs")?;

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

        let template = Self::load_template("code/github-guidelines.md.hbs")?;

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

    fn generate_hook_scripts(code_run: &CodeRun) -> Result<BTreeMap<String, String>> {
        let mut hook_scripts = BTreeMap::new();
        let hooks_prefix = "code_hooks_";

        debug!(
            "Scanning for code hook templates with prefix: {}",
            hooks_prefix
        );

        // Read the ConfigMap directory and find files with the hook prefix
        match std::fs::read_dir(CLAUDE_TEMPLATES_PATH) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if this is a hook template for code
                            if filename.starts_with(hooks_prefix) && filename.ends_with(".hbs") {
                                // Extract just the hook filename (remove prefix)
                                let hook_name =
                                    filename.strip_prefix(hooks_prefix).unwrap_or(filename);

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
            Err(e) => {
                debug!("Failed to read templates directory: {}", e);
            }
        }

        Ok(hook_scripts)
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

    /// Select the appropriate container template based on the github_app field
    fn get_agent_container_template(code_run: &CodeRun) -> String {
        let github_app = code_run.spec.github_app.as_deref().unwrap_or("");

        // Map GitHub App to agent-specific container template
        let template_name = match github_app {
            "5DLabs-Rex" | "5DLabs-Blaze" | "5DLabs-Morgan" => "container-rex.sh.hbs",
            "5DLabs-Cleo" => "container-cleo.sh.hbs",
            "5DLabs-Tess" => "container-tess.sh.hbs",
            _ => {
                // Default to the generic container template for unknown agents
                debug!(
                    "No agent-specific template for '{}', using default container.sh.hbs",
                    github_app
                );
                "container.sh.hbs"
            }
        };

        format!("code/{template_name}")
    }

    /// Extract agent name from GitHub app identifier
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
            _ => {
                return Err(crate::tasks::types::Error::ConfigError(format!(
                    "Unknown GitHub app '{github_app}' - no corresponding agent found"
                )));
            }
        };

        Ok(agent_name.to_string())
    }

    /// Get agent tool configuration from controller config
    fn get_agent_tools(
        agent_name: &str,
        config: &ControllerConfig,
    ) -> Result<crate::tasks::config::AgentTools> {
        use crate::tasks::config::{AgentTools, LocalServerConfig, LocalServerConfigs};

        // Try to get agent tools from controller config
        if let Some(agent_config) = config.agents.get(agent_name) {
            if let Some(tools) = &agent_config.tools {
                let remote_tools = tools.remote.clone();
                let local_servers =
                    tools
                        .local_servers
                        .as_ref()
                        .map(|local_servers_config| LocalServerConfigs {
                            filesystem: LocalServerConfig {
                                enabled: local_servers_config.filesystem.enabled,
                                tools: local_servers_config.filesystem.tools.clone(),
                            },
                            git: LocalServerConfig {
                                enabled: local_servers_config.git.enabled,
                                tools: local_servers_config.git.tools.clone(),
                            },
                        });

                return Ok(AgentTools {
                    remote: remote_tools,
                    local_servers,
                });
            }
        }

        // Fallback to default configuration if agent not found or no tools configured
        debug!(
            "No agent-specific tools found for '{}', using defaults",
            agent_name
        );
        Ok(AgentTools {
            remote: vec![
                "memory_create_entities".to_string(),
                "memory_add_observations".to_string(),
            ],
            local_servers: Some(LocalServerConfigs {
                filesystem: LocalServerConfig {
                    enabled: true,
                    tools: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "list_directory".to_string(),
                        "search_files".to_string(),
                        "directory_tree".to_string(),
                    ],
                },
                git: LocalServerConfig {
                    enabled: true,
                    tools: vec![
                        "git_status".to_string(),
                        "git_diff".to_string(),
                        "git_log".to_string(),
                        "git_show".to_string(),
                    ],
                },
            }),
        })
    }

    /// Load a template file from the mounted ConfigMap
    fn load_template(relative_path: &str) -> Result<String> {
        // Convert path separators to underscores for ConfigMap key lookup
        let configmap_key = relative_path.replace('/', "_");
        let full_path = Path::new(CLAUDE_TEMPLATES_PATH).join(&configmap_key);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::{CodeRun, CodeRunSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::HashMap;

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
                enable_docker: None,
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
        assert_eq!(template_path, "code/container-rex.sh.hbs");
    }

    #[test]
    fn test_cleo_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Cleo".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/container-cleo.sh.hbs");
    }

    #[test]
    fn test_tess_agent_template_selection() {
        let code_run = create_test_code_run(Some("5DLabs-Tess".to_string()));
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/container-tess.sh.hbs");
    }

    #[test]
    fn test_default_template_selection() {
        let code_run = create_test_code_run(None);
        let template_path = CodeTemplateGenerator::get_agent_container_template(&code_run);
        assert_eq!(template_path, "code/container.sh.hbs");
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
            AgentDefinition, AgentTools, ControllerConfig, LocalServerConfig, LocalServerConfigs,
        };

        let mut config = ControllerConfig::default();
        let agent_tools = AgentTools {
            remote: vec![
                "memory_create_entities".to_string(),
                "brave_web_search".to_string(),
            ],
            local_servers: Some(LocalServerConfigs {
                filesystem: LocalServerConfig {
                    enabled: true,
                    tools: vec!["read_file".to_string(), "write_file".to_string()],
                },
                git: LocalServerConfig {
                    enabled: false,
                    tools: vec![],
                },
            }),
        };

        config.agents.insert(
            "test-agent".to_string(),
            AgentDefinition {
                github_app: "Test-App".to_string(),
                tools: Some(agent_tools.clone()),
            },
        );

        let code_run = create_test_code_run(Some("Test-App".to_string()));
        let _result = CodeTemplateGenerator::generate_client_config(&code_run, &config);

        // This will fail because the extract_agent_name_from_github_app doesn't know about Test-App
        // But we can test the logic by mocking or by using a known agent
        let known_code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));

        // Add rex agent to config
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                tools: Some(agent_tools),
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
        assert!(remote_tools.contains(&serde_json::json!("brave_web_search")));

        // Verify local servers
        let local_servers = client_config["localServers"].as_object().unwrap();
        assert!(local_servers.contains_key("filesystem"));
        assert!(!local_servers.contains_key("git")); // git should be disabled

        let filesystem = &local_servers["filesystem"];
        assert_eq!(filesystem["command"], "npx");
        assert!(filesystem["tools"].is_array());
    }
}
