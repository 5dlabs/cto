//! Configuration Bridge
//!
//! Translates between universal configuration and CLI-specific formats.
//! This is the core component that enables CLI-agnostic operation.

use crate::cli::types::*;
use async_trait::async_trait;

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
    async fn to_cli_config(&self, universal: &UniversalConfig) -> Result<TranslationResult> {
        // Generate CLAUDE.md content
        let mut claude_md = String::new();

        // Project context
        claude_md.push_str("# Project Context\n\n");
        claude_md.push_str(&format!("Project: {}\n", universal.context.project_name));
        claude_md.push_str(&format!(
            "Description: {}\n\n",
            universal.context.project_description
        ));

        // Architecture notes
        if !universal.context.architecture_notes.is_empty() {
            claude_md.push_str("# Architecture\n\n");
            claude_md.push_str(&format!("{}\n\n", universal.context.architecture_notes));
        }

        // Constraints
        if !universal.context.constraints.is_empty() {
            claude_md.push_str("# Constraints\n\n");
            for constraint in &universal.context.constraints {
                claude_md.push_str(&format!("- {constraint}\n"));
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
        toml_config.push_str(&format!("model = \"{}\"\n", universal.settings.model));
        toml_config.push_str(&format!(
            "sandbox_mode = \"{}\"\n",
            universal.settings.sandbox_mode
        ));

        // Model constraints
        if universal.settings.max_tokens > 0 {
            toml_config.push_str(&format!(
                "model_max_output_tokens = {}\n",
                universal.settings.max_tokens
            ));
        }

        // Approval policy based on sandbox mode
        let approval_policy = match universal.settings.sandbox_mode.as_str() {
            "read-only" => "untrusted",
            "workspace-write" => "on-failure",
            _ => "never",
        };
        toml_config.push_str(&format!("approval_policy = \"{approval_policy}\"\n"));

        // Agent instructions as project doc
        if !universal.agent.instructions.is_empty() {
            toml_config.push_str(&format!("project_doc_max_bytes = {}\n", 32768));
            // 32KB
        }

        // MCP server configuration
        if let Some(mcp_config) = &universal.mcp_config {
            toml_config.push('\n');
            toml_config.push_str("# MCP Server Configuration\n");

            for server in &mcp_config.servers {
                toml_config.push_str(&format!("[mcp_servers.{}]\n", server.name));
                toml_config.push_str(&format!("command = \"{}\"\n", server.command));

                if !server.args.is_empty() {
                    toml_config.push_str(&format!("args = {:?}\n", server.args));
                }

                if !server.env.is_empty() {
                    // Convert HashMap to TOML inline table format
                    let env_pairs: Vec<String> = server
                        .env
                        .iter()
                        .map(|(k, v)| format!("\"{k}\" = \"{v}\""))
                        .collect();
                    toml_config.push_str(&format!("env = {{{}}}\n", env_pairs.join(", ")));
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

/// JSON format CLI adapter (OpenCode)
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
            env_vars: vec![], // OpenCode uses its own auth
        })
    }

    async fn generate_command(&self, task: &str, _config: &UniversalConfig) -> Result<Vec<String>> {
        Ok(vec!["opencode".to_string(), task.to_string()])
    }

    fn required_env_vars(&self) -> Vec<String> {
        vec![] // OpenCode handles auth internally
    }

    fn cli_type(&self) -> CLIType {
        CLIType::OpenCode
    }
}

/// Main configuration bridge
pub struct ConfigurationBridge {
    adapters: std::collections::HashMap<CLIType, Box<dyn CLIAdapter>>,
}

impl ConfigurationBridge {
    /// Create a new configuration bridge with all supported adapters
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
    pub fn supported_clis(&self) -> Vec<CLIType> {
        self.adapters.keys().cloned().collect()
    }

    /// Check if a CLI type is supported
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
                architecture_notes: "".to_string(),
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
        let mut toolman_env = std::collections::HashMap::new();
        toolman_env.insert(
            "TOOLMAN_SERVER_URL".to_string(),
            "http://toolman.agent-platform.svc.cluster.local:3000/mcp".to_string(),
        );

        let universal = UniversalConfig {
            context: ContextConfig {
                project_name: "Test Project".to_string(),
                project_description: "A test project".to_string(),
                architecture_notes: "".to_string(),
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
                    name: "toolman".to_string(),
                    command: "toolman".to_string(),
                    args: vec!["--working-dir".to_string(), "/workspace".to_string()],
                    env: toolman_env,
                }],
            }),
        };

        let result = adapter.to_cli_config(&universal).await.unwrap();

        // Check that MCP server configuration is included in TOML format
        assert!(result.content.contains("[mcp_servers.toolman]"));
        assert!(result.content.contains("command = \"toolman\""));
        assert!(result
            .content
            .contains("args = [\"--working-dir\", \"/workspace\"]"));
        assert!(result.content.contains("TOOLMAN_SERVER_URL"));

        // Check that MCP server environment variables are included
        assert!(result.env_vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(result.env_vars.contains(&"TOOLMAN_SERVER_URL".to_string()));
    }
}
