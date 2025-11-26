//! Configuration Bridge
//!
//! Translates between universal configuration and CLI-specific formats.
//! This is the core component that enables CLI-agnostic operation.

use crate::cli::types::{CLIType, ConfigFile, TranslationResult, UniversalConfig};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashSet;
use std::env;
use std::fmt::Write;

/// Trait for CLI-specific configuration translators
#[async_trait]
pub trait CLIAdapter: Send + Sync {
    /// Convert universal config to CLI-specific format
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult>;

    /// Generate CLI-specific command from configuration
    async fn generate_command(&self, task: &str, config: &UniversalConfig) -> Result<Vec<String>>;

    /// Get required environment variables for this CLI
    fn required_env_vars(&self) -> Vec<String>;

    /// Get CLI type this adapter handles
    fn cli_type(&self) -> CLIType;
}

/// Markdown format CLI adapter (Claude, Cursor)
pub struct MarkdownCLIAdapter;

#[async_trait]
impl CLIAdapter for MarkdownCLIAdapter {
    #[allow(clippy::too_many_lines)]
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate CLAUDE.md content
        let mut claude_md = String::new();

        // Project context
        claude_md.push_str("# Project Context\n\n");
        let _ = writeln!(claude_md, "Project: {}", universal.context.project_name);
        let _ = writeln!(
            claude_md,
            "Description: {}\n",
            universal.context.project_description
        );

        // Architecture notes
        if !universal.context.architecture_notes.is_empty() {
            claude_md.push_str("# Architecture\n\n");
            claude_md.push_str(&universal.context.architecture_notes);
            claude_md.push_str("\n\n");
        }

        // Constraints
        if !universal.context.constraints.is_empty() {
            claude_md.push_str("# Constraints\n\n");
            for constraint in &universal.context.constraints {
                let _ = writeln!(claude_md, "- {constraint}");
            }
            claude_md.push('\n');
        }

        // Agent instructions
        claude_md.push_str("# Instructions\n\n");
        claude_md.push_str(&universal.agent.instructions);

        Ok(TranslationResult {
            content: claude_md.clone(),
            config_files: vec![ConfigFile {
                path: "/workspace/CLAUDE.md".to_string(),
                content: claude_md,
                permissions: Some("0644".to_string()),
            }],
            env_vars: vec![], // Claude doesn't require specific env vars
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["claude-code".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec![] // Claude uses its own authentication
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Claude
    }
}

/// TOML format CLI adapter (Codex)
pub struct TomlCLIAdapter;

#[async_trait]
impl CLIAdapter for TomlCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate TOML config for Codex
        let mut toml_config = String::new();

        // Basic settings
        let _ = writeln!(toml_config, "model = \"{}\"", universal.settings.model);
        let _ = writeln!(
            toml_config,
            "sandbox_mode = \"{}\"",
            universal.settings.sandbox_mode
        );

        // Model constraints
        if universal.settings.max_tokens > 0 {
            let _ = writeln!(
                toml_config,
                "model_max_output_tokens = {}",
                universal.settings.max_tokens
            );
        }

        // Approval policy based on sandbox mode
        toml_config.push_str("approval_policy = \"never\"\n");

        // Agent instructions as project doc
        if !universal.agent.instructions.is_empty() {
            let _ = writeln!(toml_config, "project_doc_max_bytes = {}", 32_768);
            // 32KB
        }

        // MCP server configuration
        if let Some(mcp_config) = &universal.mcp_config {
            toml_config.push('\n');
            toml_config.push_str("# MCP Server Configuration\n");

            for server in &mcp_config.servers {
                let _ = writeln!(toml_config, "[mcp_servers.{}]", server.name);
                let _ = writeln!(toml_config, "command = \"{}\"", server.command);

                if !server.args.is_empty() {
                    let _ = writeln!(toml_config, "args = {:?}", server.args);
                }

                if !server.env.is_empty() {
                    // Convert HashMap to TOML inline table format
                    let env_pairs: Vec<String> = server
                        .env
                        .iter()
                        .map(|(k, v)| format!("\"{k}\" = \"{v}\""))
                        .collect();
                    let _ = writeln!(toml_config, "env = {{{}}}", env_pairs.join(", "));
                }

                toml_config.push('\n');
            }
        }

        Ok(TranslationResult {
            content: toml_config.clone(),
            config_files: vec![
                ConfigFile {
                    path: "/home/node/.codex/config.toml".to_string(),
                    content: toml_config,
                    permissions: Some("0644".to_string()),
                },
                // Also create AGENTS.md for project instructions
                ConfigFile {
                    path: "/workspace/AGENTS.md".to_string(),
                    content: universal.agent.instructions.clone(),
                    permissions: Some("0644".to_string()),
                },
            ],
            env_vars: {
                let mut env_vars = vec!["OPENAI_API_KEY".to_string()];

                // Add MCP server environment variables
                if let Some(mcp_config) = &universal.mcp_config {
                    for server in &mcp_config.servers {
                        for env_var in server.env.keys() {
                            if !env_vars.contains(env_var) {
                                env_vars.push(env_var.clone());
                            }
                        }
                    }
                }

                env_vars
            },
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec![
            "codex".to_string(),
            "exec".to_string(),
            "--full-auto".to_string(),
            task.to_string(),
        ])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["OPENAI_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Codex
    }
}

/// JSON format CLI adapter (`OpenCode`)
pub struct JsonCLIAdapter;

#[async_trait]
impl CLIAdapter for JsonCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate JSON config for OpenCode
        let config = serde_json::json!({
            "model": universal.settings.model,
            "sandbox_mode": universal.settings.sandbox_mode,
            "max_tokens": universal.settings.max_tokens,
            "temperature": universal.settings.temperature,
            "project": {
                "name": universal.context.project_name,
                "description": universal.context.project_description
            },
            "agent": {
                "instructions": universal.agent.instructions
            }
        });

        let json_config = serde_json::to_string_pretty(&config)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        Ok(TranslationResult {
            content: json_config.clone(),
            config_files: vec![ConfigFile {
                path: "/home/node/.config/opencode/config.json".to_string(),
                content: json_config,
                permissions: Some("0644".to_string()),
            }],
            env_vars: vec!["OPENAI_API_KEY".to_string()],
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["opencode".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["OPENAI_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::OpenCode
    }
}

/// Cursor CLI adapter (skeleton)
pub struct CursorCLIAdapter;

#[async_trait]
impl CLIAdapter for CursorCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        let cursor_config = serde_json::json!({
            "version": 1,
            "editor": { "vimMode": false },
            "permissions": {
                "allow": [],
                "deny": [],
            },
            "notes": "TODO(cursor): populate CLI configuration",
        });

        let config_content = serde_json::to_string_pretty(&cursor_config)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        let mut config_files = vec![ConfigFile {
            path: "/workspace/.cursor/cli.json".to_string(),
            content: config_content.clone(),
            permissions: Some("0644".to_string()),
        }];

        config_files.push(ConfigFile {
            path: "/workspace/AGENTS.md".to_string(),
            content: universal.agent.instructions.clone(),
            permissions: Some("0644".to_string()),
        });

        Ok(TranslationResult {
            content: config_content,
            config_files,
            env_vars: vec!["CURSOR_API_KEY".to_string()],
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec![
            "cursor-agent".to_string(),
            "--print".to_string(),
            "--force".to_string(),
            task.to_string(),
        ])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["CURSOR_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Cursor
    }
}

/// Factory CLI adapter (placeholder)
pub struct FactoryCLIAdapter;

#[async_trait]
impl CLIAdapter for FactoryCLIAdapter {
    #[allow(clippy::too_many_lines)]
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        let sandbox_mode = universal.settings.sandbox_mode.clone();
        let auto_level = match sandbox_mode.as_str() {
            "danger-full-access" => "high",
            "workspace-write" => "medium",
            _ => "low",
        };

        let reasoning_effort = match auto_level {
            "high" => Some("high"),
            "medium" => Some("medium"),
            _ => None,
        };

        let tool_names: HashSet<String> = universal
            .tools
            .iter()
            .map(|tool| tool.name.clone())
            .collect();

        let mut mcp_servers_json = serde_json::Map::new();
        let mut tools_endpoint: Option<String> = None;

        if let Some(mcp_config) = &universal.mcp_config {
            for server in &mcp_config.servers {
                if server.name.eq_ignore_ascii_case("tools") {
                    tools_endpoint = server.env.get("TOOLS_SERVER_URL").cloned().or_else(|| {
                        server
                            .args
                            .iter()
                            .position(|arg| arg == "--url")
                            .and_then(|idx| server.args.get(idx + 1).cloned())
                    });
                }

                mcp_servers_json.insert(
                    server.name.clone(),
                    json!({
                        "command": server.command,
                        "args": server.args,
                        "env": server.env,
                    }),
                );
            }
        }

        let tools_url = tools_endpoint.unwrap_or_else(|| {
            env::var("TOOLS_SERVER_URL").unwrap_or_else(|_| {
                "http://tools.cto.svc.cluster.local:3000/mcp".to_string()
            })
        });
        let tools_url = tools_url.trim_end_matches('/').to_string();

        let tool_list: Vec<String> = {
            let mut list: Vec<String> = tool_names.into_iter().collect();
            list.sort();
            list
        };

        let mut model_json = json!({
            "default": universal.settings.model,
            "temperature": universal.settings.temperature,
            "maxOutputTokens": universal.settings.max_tokens,
        });
        if let Some(effort) = reasoning_effort {
            model_json
                .as_object_mut()
                .expect("model json is object")
                .insert("reasoningEffort".to_string(), json!(effort));
        }

        let factory_config = json!({
            "version": 1,
            "model": model_json,
            "autoRun": {
                "enabled": true,
                "level": auto_level,
            },
            "specificationMode": {
                "default": true,
            },
            "execution": {
                "approvalPolicy": "never",
                "sandboxMode": sandbox_mode,
                "projectDocMaxBytes": 65536,
            },
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)"],
                "deny": []
            },
            "tools": {
                "endpoint": tools_url,
                "tools": tool_list,
            },
            "mcp": {
                "servers": serde_json::Value::Object(mcp_servers_json)
            }
        });

        let config_content = serde_json::to_string_pretty(&factory_config)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        let project_permissions = json!({
            "permissions": {
                "allow": ["Shell(*)", "Read(**/*)", "Write(**/*)"],
                "deny": []
            }
        });

        let project_permissions_content = serde_json::to_string_pretty(&project_permissions)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        let config_files = vec![
            ConfigFile {
                path: "/home/node/.factory/cli-config.json".to_string(),
                content: config_content.clone(),
                permissions: Some("0644".to_string()),
            },
            ConfigFile {
                path: "/workspace/.factory/cli.json".to_string(),
                content: project_permissions_content,
                permissions: Some("0644".to_string()),
            },
            ConfigFile {
                path: "/workspace/AGENTS.md".to_string(),
                content: universal.agent.instructions.clone(),
                permissions: Some("0644".to_string()),
            },
        ];

        Ok(TranslationResult {
            content: config_content,
            config_files,
            env_vars: vec!["FACTORY_API_KEY".to_string()],
        })
    }

    async fn generate_command(&self, task: &str, config: &UniversalConfig) -> Result<Vec<String>> {
        let auto_level = match config.settings.sandbox_mode.as_str() {
            "danger-full-access" => "high",
            "workspace-write" => "medium",
            _ => "low",
        };

        Ok(vec![
            "droid".to_string(),
            "exec".to_string(),
            "--output-format".to_string(),
            "json".to_string(),
            "--auto".to_string(),
            auto_level.to_string(),
            task.to_string(),
        ])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["FACTORY_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Factory
    }
}

/// Gemini CLI adapter
pub struct GeminiCLIAdapter;

#[async_trait]
impl CLIAdapter for GeminiCLIAdapter {
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        let json_obj = json!({
            "model": universal.settings.model,
            "temperature": universal.settings.temperature,
            "max_tokens": universal.settings.max_tokens,
            "tools": universal.tools,
            "memory_file": "GEMINI.md",
            "api_key_env": "GOOGLE_API_KEY",
            "base_url": "https://generativelanguage.googleapis.com/v1beta"
        });

        let json_config = serde_json::to_string_pretty(&json_obj)
            .map_err(|e| BridgeError::ConfigSerializationError(e.to_string()))?;

        Ok(TranslationResult {
            content: json_config.clone(),
            config_files: vec![ConfigFile {
                path: "/workspace/.gemini/config.json".to_string(),
                content: json_config,
                permissions: Some("0644".to_string()),
            }],
            env_vars: vec!["GOOGLE_API_KEY".to_string()],
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["gemini-cli".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec!["GOOGLE_API_KEY".to_string()]
    }

    fn cli_type(&self) -> CLIType {
        CLIType::Gemini
    }
}

/// Main configuration bridge
pub struct ConfigurationBridge {
    adapters: std::collections::HashMap<CLIType, Box<dyn CLIAdapter>>,
}

impl ConfigurationBridge {
    /// Create a new configuration bridge with all supported adapters
    #[must_use]
    pub fn new() -> Self {
        let mut adapters = std::collections::HashMap::new();

        adapters.insert(
            CLIType::Claude,
            Box::new(MarkdownCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::Codex,
            Box::new(TomlCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::OpenCode,
            Box::new(JsonCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::Cursor,
            Box::new(CursorCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::Factory,
            Box::new(FactoryCLIAdapter) as Box<dyn CLIAdapter>,
        );
        adapters.insert(
            CLIType::Gemini,
            Box::new(GeminiCLIAdapter) as Box<dyn CLIAdapter>,
        );

        Self { adapters }
    }

    /// Translate universal config to CLI-specific format
    pub async fn translate(
        &self,
        universal: &UniversalConfig,
        cli_type: CLIType,
    ) -> Result<TranslationResult> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        adapter.to_cli_config(universal).await
    }

    /// Generate CLI-specific command
    pub async fn generate_command(
        &self,
        task: &str,
        config: &UniversalConfig,
        cli_type: CLIType,
    ) -> Result<Vec<String>> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        adapter.generate_command(task, config).await
    }

    /// Get required environment variables for a CLI
    pub fn required_env_vars(&self, cli_type: CLIType) -> Result<Vec<String>> {
        let adapter = self
            .adapters
            .get(&cli_type)
            .ok_or(BridgeError::UnsupportedCLI(cli_type))?;

        Ok(adapter.required_env_vars())
    }

    /// Get all supported CLI types
    #[must_use]
    pub fn supported_clis(&self) -> Vec<CLIType> {
        self.adapters.keys().copied().collect()
    }

    /// Check if a CLI type is supported
    #[must_use]
    pub fn supports_cli(&self, cli_type: CLIType) -> bool {
        self.adapters.contains_key(&cli_type)
    }
}

impl Default for ConfigurationBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge-specific errors
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Unsupported CLI type: {0:?}")]
    UnsupportedCLI(CLIType),

    #[error("Configuration serialization failed: {0}")]
    ConfigSerializationError(String),

    #[error("Command generation failed: {0}")]
    CommandGenerationError(String),

    #[error("File system error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BridgeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::{
        AgentConfig, ContextConfig, MCPServer, SettingsConfig, ToolDefinition, UniversalMCPConfig,
    };
    use serde_json::Value;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_claude_adapter() {
        let adapter = MarkdownCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: "Simple architecture".to_string(),
                constraints: vec!["No external APIs".to_string()],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "claude-3-5-sonnet".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "read-only".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean, well-documented code.".to_string(),
            },
            mcp_config: None,
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result.content.contains("# Project Context"));
        assert!(result.content.contains("Test Project"));
        assert!(result.config_files.len() == 1);
        assert!(result.config_files[0].path == "/workspace/CLAUDE.md");
    }

    #[tokio::test]
    async fn test_codex_adapter() {
        let adapter = TomlCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean code.".to_string(),
            },
            mcp_config: None,
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result.content.contains("model = \"gpt-4\""));
        assert!(result
            .content
            .contains("sandbox_mode = \"workspace-write\""));
        assert!(result.config_files.len() == 2); // config.toml + AGENTS.md
        assert!(result.env_vars.contains(&"OPENAI_API_KEY".to_string()));
    }

    #[tokio::test]
    async fn test_codex_adapter_with_mcp_servers() {
        let adapter = TomlCLIAdapter;

        // Create test MCP server configuration
        let mut tools_env = std::collections::HashMap::new();
        tools_env.insert(
            "TOOLS_SERVER_URL".to_string(),
            "http://tools.cto.svc.cluster.local:3000/mcp".to_string(),
        );

        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 4000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec!["coding".to_string()],
                instructions: "Write clean code.".to_string(),
            },
            mcp_config: Some(UniversalMCPConfig {
                servers: vec![MCPServer {
                    name: "tools".to_string(),
                    command: "tools".to_string(),
                    args: vec!["--working-dir".to_string(), "/workspace".to_string()],
                    env: tools_env,
                }],
            }),
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();

        // Check that MCP server configuration is included in TOML format
        assert!(result.content.contains("[mcp_servers.tools]"));
        assert!(result.content.contains("command = \"tools\""));
        assert!(result
            .content
            .contains("args = [\"--working-dir\", \"/workspace\"]"));
        assert!(result.content.contains("TOOLS_SERVER_URL"));

        // Check that MCP server environment variables are included
        assert!(result.env_vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(result.env_vars.contains(&"TOOLS_SERVER_URL".to_string()));
    }

    #[tokio::test]
    async fn test_cursor_adapter_placeholder() {
        let adapter = CursorCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Cursor Project".to_string(),
                project_description: "Testing cursor adapter".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-5-cursor".to_string(),
                temperature: 0.2,
                max_tokens: 2000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec![],
                instructions: "Test instructions".to_string(),
            },
            mcp_config: None,
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result
            .config_files
            .iter()
            .any(|f| f.path.ends_with(".cursor/cli.json")));
        assert!(result.env_vars.contains(&"CURSOR_API_KEY".to_string()));

        let command = adapter
            .generate_command("implement feature", &universal)
            .await
            .unwrap();
        assert_eq!(command[0], "cursor-agent");
        assert!(command.contains(&"--print".to_string()));
    }

    #[tokio::test]
    async fn test_factory_adapter_generates_config() {
        let adapter = FactoryCLIAdapter;
        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Factory Project".to_string(),
                project_description: "Testing factory adapter".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![ToolDefinition {
                name: "memory_create_entities".to_string(),
                description: "Create memory entities".to_string(),
                parameters: json!({}),
                implementations: HashMap::new(),
            }],
            settings: SettingsConfig {
                model: "gpt-5-factory-high".to_string(),
                temperature: 0.42,
                max_tokens: 64000,
                timeout: 300,
                sandbox_mode: "workspace-write".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec![],
                instructions: "Factory instructions".to_string(),
            },
            mcp_config: Some(UniversalMCPConfig {
                servers: vec![MCPServer {
                    name: "tools".to_string(),
                    command: "tools".to_string(),
                    args: vec!["--url".to_string(), "http://localhost:3000/mcp".to_string()],
                    env: HashMap::from([(
                        String::from("TOOLS_SERVER_URL"),
                        String::from("http://localhost:3000/mcp"),
                    )]),
                }],
            }),
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();
        assert!(result
            .config_files
            .iter()
            .any(|f| f.path == "/home/node/.factory/cli-config.json"));
        assert!(result
            .config_files
            .iter()
            .any(|f| f.path == "/workspace/.factory/cli.json"));
        assert!(result
            .config_files
            .iter()
            .any(|f| f.path == "/workspace/AGENTS.md"));
        assert!(result.env_vars.contains(&"FACTORY_API_KEY".to_string()));

        let parsed: Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(
            parsed
                .get("model")
                .and_then(|model| model.get("default"))
                .and_then(Value::as_str)
                .unwrap(),
            "gpt-5-factory-high"
        );
        assert_eq!(
            parsed
                .get("autoRun")
                .and_then(|auto| auto.get("level"))
                .and_then(Value::as_str)
                .unwrap(),
            "medium"
        );
        let tool_list = parsed
            .get("tools")
            .and_then(|tools| tools.get("tools"))
            .and_then(Value::as_array)
            .unwrap();
        assert!(tool_list.contains(&Value::String("memory_create_entities".to_string())));

        let command = adapter
            .generate_command("implement feature", &universal)
            .await
            .unwrap();
        assert_eq!(command[0], "droid");
        assert!(command.contains(&"--auto".to_string()));
        assert!(command.contains(&"medium".to_string()));
        assert!(command.contains(&"implement feature".to_string()));
    }
}
