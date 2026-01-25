//! Tooling module for calling MCP tools to install skills and MCP servers.
//!
//! This module provides HTTP client functionality to call the CTO tools service
//! for automatic installation of detected skills and MCP servers from research items.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// Default URL for the CTO tools MCP service.
const DEFAULT_MCP_SERVER_URL: &str = "http://cto-tools.cto.svc.cluster.local:3000/mcp";

/// Default agents to assign new skills to.
const DEFAULT_SKILL_AGENTS: &[&str] = &["rex", "blaze", "nova"];

/// Configuration for the tooling client.
#[derive(Debug, Clone)]
pub struct ToolingConfig {
    /// URL of the MCP server.
    pub mcp_server_url: String,
    /// Default agents to assign skills to.
    pub default_agents: Vec<String>,
    /// Whether to skip auto-merge for PRs.
    pub skip_merge: bool,
    /// HTTP request timeout in seconds.
    pub timeout_secs: u64,
    /// Whether auto-install is enabled.
    pub enabled: bool,
}

impl Default for ToolingConfig {
    fn default() -> Self {
        Self {
            mcp_server_url: std::env::var("MCP_SERVER_URL")
                .unwrap_or_else(|_| DEFAULT_MCP_SERVER_URL.to_string()),
            default_agents: DEFAULT_SKILL_AGENTS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            skip_merge: false,
            timeout_secs: 30,
            enabled: true,
        }
    }
}

/// JSON-RPC request structure.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u32,
    method: &'static str,
    params: JsonRpcParams,
}

/// Parameters for tools/call method.
#[derive(Debug, Serialize)]
struct JsonRpcParams {
    name: String,
    arguments: Value,
}

/// JSON-RPC response structure.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u32>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Client for calling MCP tools.
pub struct ToolingClient {
    config: ToolingConfig,
    client: reqwest::Client,
}

impl ToolingClient {
    /// Create a new tooling client with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(ToolingConfig::default())
    }

    /// Create a new tooling client with custom configuration.
    pub fn with_config(config: ToolingConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Create a tooling client from environment variables.
    pub fn from_env() -> Result<Self> {
        let config = ToolingConfig {
            mcp_server_url: std::env::var("MCP_SERVER_URL")
                .unwrap_or_else(|_| DEFAULT_MCP_SERVER_URL.to_string()),
            default_agents: std::env::var("SKILL_DEFAULT_AGENTS").map_or_else(
                |_| {
                    DEFAULT_SKILL_AGENTS
                        .iter()
                        .map(|s| (*s).to_string())
                        .collect()
                },
                |s| s.split(',').map(str::trim).map(String::from).collect(),
            ),
            skip_merge: std::env::var("AUTO_INSTALL_SKIP_MERGE")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
            timeout_secs: std::env::var("MCP_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            enabled: std::env::var("AUTO_INSTALL_ENABLED")
                .map(|s| s != "false" && s != "0")
                .unwrap_or(true),
        };

        Self::with_config(config)
    }

    /// Check if auto-install is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Install a skill from a GitHub repository.
    pub async fn install_skill(&self, github_url: &str) -> Result<InstallResult> {
        self.install_skill_with_agents(github_url, &self.config.default_agents)
            .await
    }

    /// Install a skill with specific agent assignment.
    pub async fn install_skill_with_agents(
        &self,
        github_url: &str,
        agents: &[String],
    ) -> Result<InstallResult> {
        if !self.config.enabled {
            return Ok(InstallResult {
                success: false,
                message: "Auto-install is disabled".to_string(),
                coderun_name: None,
            });
        }

        tracing::info!(url = %github_url, agents = ?agents, "Installing skill from research");

        let arguments = json!({
            "github_url": github_url,
            "agents": agents,
            "skip_merge": self.config.skip_merge
        });

        self.call_tool("add_skills", arguments).await
    }

    /// Install an MCP server from a GitHub repository.
    pub async fn install_mcp_server(&self, github_url: &str) -> Result<InstallResult> {
        if !self.config.enabled {
            return Ok(InstallResult {
                success: false,
                message: "Auto-install is disabled".to_string(),
                coderun_name: None,
            });
        }

        tracing::info!(url = %github_url, "Installing MCP server from research");

        let arguments = json!({
            "github_url": github_url,
            "skip_merge": self.config.skip_merge
        });

        self.call_tool("add_mcp_server", arguments).await
    }

    /// Call an MCP tool via JSON-RPC.
    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<InstallResult> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "tools/call",
            params: JsonRpcParams {
                name: tool_name.to_string(),
                arguments,
            },
        };

        tracing::debug!(
            tool = tool_name,
            url = %self.config.mcp_server_url,
            "Calling MCP tool"
        );

        let response = self
            .client
            .post(&self.config.mcp_server_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to call MCP tool '{tool_name}'"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "MCP server returned error status {status}: {body}"
            ));
        }

        let rpc_response: JsonRpcResponse = response
            .json()
            .await
            .context("Failed to parse MCP response")?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!(
                "MCP tool '{}' failed with code {}: {}",
                tool_name,
                error.code,
                error.message
            ));
        }

        // Parse the result
        let result = rpc_response.result.unwrap_or(json!({}));

        // Try to extract coderun_name from the result
        let coderun_name = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
            .and_then(|text| {
                text.lines()
                    .find(|line| line.contains("CodeRun:") || line.contains("coderun:"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim().to_string())
            });

        tracing::info!(
            tool = tool_name,
            coderun = ?coderun_name,
            "MCP tool call completed"
        );

        Ok(InstallResult {
            success: true,
            message: format!("Successfully triggered {tool_name}"),
            coderun_name,
        })
    }
}

/// Result of an install operation.
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Whether the install was triggered successfully.
    pub success: bool,
    /// Human-readable message about the result.
    pub message: String,
    /// Name of the CodeRun created (if any).
    pub coderun_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ToolingConfig::default();
        assert!(config.enabled);
        assert!(!config.skip_merge);
        assert_eq!(config.default_agents.len(), 3);
    }

    #[test]
    fn test_install_result() {
        let result = InstallResult {
            success: true,
            message: "Test".to_string(),
            coderun_name: Some("test-coderun".to_string()),
        };
        assert!(result.success);
        assert!(result.coderun_name.is_some());
    }
}
