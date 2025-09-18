use crate::crds::CodeRun;
use crate::tasks::config::ControllerConfig;
use crate::tasks::types::Result;
use handlebars::Handlebars;

use serde_json::{json, Value};
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

        // Derive allowed env var name lists for inclusion in CLAUDE.md
        // 1) Workflow-provided env names (keys only)
        let workflow_env_vars: Vec<String> = code_run
            .spec
            .env
            .keys()
            .cloned()
            .collect();

        // 2) From requirements.yaml (environment keys and mapped secret key names)
        let mut req_env_vars: Vec<String> = Vec::new();
        let mut req_secret_sources: Vec<String> = Vec::new();

        // Base64 decode helper import for task_requirements parsing
        use base64::{engine::general_purpose, Engine as _};

        if let Some(req_b64) = &code_run.spec.task_requirements {
            if !req_b64.trim().is_empty() {
                if let Ok(decoded) = general_purpose::STANDARD.decode(req_b64) {
                    if let Ok(req_yaml) = serde_yaml::from_slice::<serde_yaml::Value>(&decoded) {
                        if let Some(env_map) = req_yaml
                            .get("environment")
                            .and_then(|e| e.as_mapping())
                        {
                            for (k, _v) in env_map {
                                if let Some(key) = k.as_str() {
                                    req_env_vars.push(key.to_string());
                                }
                            }
                        }
                        if let Some(secrets) = req_yaml
                            .get("secrets")
                            .and_then(|s| s.as_sequence())
                        {
                            for secret in secrets {
                                if let Some(m) = secret.as_mapping() {
                                    if let Some(name) = m.get(serde_yaml::Value::from("name"))
                                        .and_then(|n| n.as_str())
                                    {
                                        req_secret_sources.push(name.to_string());
                                    }
                                    // If there are key mappings, surface the env var names (right-hand side)
                                    if let Some(keys) = m.get(serde_yaml::Value::from("keys"))
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
            "workflow_env_vars": workflow_env_vars,
            "requirements_env_vars": requirements_env_vars,
            "requirements_secret_sources": requirements_secret_sources,
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
                    for (k, val) in map.iter() {
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
            let remote_tools = tools_value.get("remote").cloned().unwrap_or_else(|| json!([]));
            let local_servers = sanitize_local_servers(tools_value.get("localServers").unwrap_or(&json!({})));
            json!({
                "remoteTools": remote_tools,
                "localServers": local_servers
            })
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
                        if !cfg.tools.is_empty() { obj.insert("tools".to_string(), json!(cfg.tools.clone())); }
                        if let Some(cmd) = &cfg.command { obj.insert("command".to_string(), json!(cmd)); }
                        if let Some(args) = &cfg.args { obj.insert("args".to_string(), json!(args)); }
                        if let Some(wd) = &cfg.working_directory { obj.insert("workingDirectory".to_string(), json!(wd)); }
                        local_servers_obj.insert(name.clone(), Value::Object(obj));
                    }
                }
            }

            Value::Object(
                vec![
                    ("remoteTools".to_string(), remote_tools),
                    ("localServers".to_string(), Value::Object(local_servers_obj)),
                ]
                .into_iter()
                .collect(),
            )
        };

        // Small helpers for merge logic
        let collect_string_array = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        };

        let merge_client_configs = |base: &Value, overlay: &Value| -> Value {
            // Merge remoteTools as a union preserving base order
            let mut merged_remote = collect_string_array(base.get("remoteTools").unwrap_or(&json!([])));
            let overlay_remote = collect_string_array(overlay.get("remoteTools").unwrap_or(&json!([])));
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
            if let Some(m) = base_ls { keys.extend(m.keys().cloned()); }
            if let Some(m) = overlay_ls { keys.extend(m.keys().cloned()); }

            for k in keys {
                let b = base_ls.and_then(|m| m.get(&k));
                let o = overlay_ls.and_then(|m| m.get(&k));

                let merged_val = match (b, o) {
                    (Some(Value::Object(bm)), Some(Value::Object(om))) => {
                        let mut out = bm.clone();
                        // tools union if present
                        let base_tools = collect_string_array(out.get("tools").unwrap_or(&json!([])));
                        let overlay_tools = collect_string_array(om.get("tools").unwrap_or(&json!([])));
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
                        for (ok, ov) in om.iter() {
                            if ok == "tools" { continue; }
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
        let minimal_client = json!({ "remoteTools": [], "localServers": {} });

        to_string_pretty(&minimal_client).map_err(|e| {
            crate::tasks::types::Error::ConfigError(format!(
                "Failed to serialize empty clientConfig: {e}"
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
    ) -> Result<crate::tasks::config::AgentTools> {
        use crate::tasks::config::AgentTools;

        // Try to get agent tools from controller config
        if let Some(agent_config) = config.agents.get(agent_name) {
            if let Some(tools) = &agent_config.tools {
                return Ok(AgentTools {
                    remote: tools.remote.clone(),
                    local_servers: tools.local_servers.clone(),
                });
            }
        }

        // Fallback to default configuration if agent not found or no tools configured
        debug!(
            "No agent-specific tools found for '{}', using defaults",
            agent_name
        );
        Ok(AgentTools { remote: vec![], local_servers: None })
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
        use crate::tasks::config::{AgentDefinition, AgentTools, ControllerConfig, LocalServerConfig};

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
                    "brave_web_search".to_string(),
                ],
                local_servers: Some(servers),
            }
        };

        config.agents.insert(
            "test-agent".to_string(),
            AgentDefinition {
                github_app: "Test-App".to_string(),
                tools: Some(agent_tools.clone()),
                client_config: None,
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
                tools: Some(agent_tools),
                client_config: Some(serde_json::json!({
                    "remoteTools": ["memory_create_entities", "brave_web_search"],
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
        assert!(remote_tools.contains(&serde_json::json!("brave_web_search")));

        // Verify local servers (generic server names)
        let local_servers = client_config["localServers"].as_object().unwrap();
        assert!(local_servers.contains_key("serverA"));
        assert!(!local_servers.contains_key("serverB"));

        let server_a = &local_servers["serverA"];
        assert_eq!(server_a["command"], "npx");
        assert!(server_a["tools"].is_array());
    }

    #[test]
    fn test_merge_client_config_overlay_on_helm_defaults() {
        use crate::tasks::config::{AgentDefinition, AgentTools, ControllerConfig, LocalServerConfig};

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
            AgentTools { remote: vec!["memory_create_entities".to_string()], local_servers: Some(servers) }
        };
        config.agents.insert(
            "rex".to_string(),
            AgentDefinition {
                github_app: "5DLabs-Rex".to_string(),
                tools: Some(helm_tools),
                client_config: None,
            },
        );

        // CodeRun with annotation overlay client config
        let mut code_run = create_test_code_run(Some("5DLabs-Rex".to_string()));
        let mut ann = BTreeMap::new();
        ann.insert(
            "agents.platform/tools-config".to_string(),
            serde_json::json!({
                "remoteTools": ["brave-search_brave_web_search"],
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
        assert!(remote.contains(&serde_json::json!("brave-search_brave_web_search")));

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
