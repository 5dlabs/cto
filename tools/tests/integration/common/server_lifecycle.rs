#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::redundant_else)]

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tokio::time::{sleep, timeout};

use crate::common::mcp_test_client::McpTestClient;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub timeout_secs: u64,
    pub setup_delay_ms: u64,
    pub working_directory: Option<String>,
}

impl ServerConfig {
    pub fn new(name: &str, command: &str, args: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args,
            env: HashMap::new(),
            timeout_secs: 30,
            setup_delay_ms: 1000,
            working_directory: None,
        }
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_setup_delay(mut self, delay_ms: u64) -> Self {
        self.setup_delay_ms = delay_ms;
        self
    }

    pub fn with_working_directory(mut self, working_dir: &str) -> Self {
        self.working_directory = Some(working_dir.to_string());
        self
    }
}

pub struct TestServer {
    pub config: ServerConfig,
    pub client: McpTestClient,
}

impl TestServer {
    pub async fn start(config: ServerConfig) -> Result<Self> {
        println!(
            "Starting server: {} with command: {} {:?}",
            config.name, config.command, config.args
        );

        let client = timeout(
            Duration::from_secs(config.timeout_secs),
            McpTestClient::new(
                &config.command,
                &config.args,
                config.working_directory.as_deref(),
            ),
        )
        .await
        .context("Server startup timeout")?
        .context("Failed to create MCP client")?;

        // Give the server time to start up
        sleep(Duration::from_millis(config.setup_delay_ms)).await;

        let mut server = Self { config, client };

        // Test basic connectivity
        server
            .health_check()
            .await
            .context("Server health check failed after startup")?;

        println!("Server {} started successfully", server.config.name);
        Ok(server)
    }

    pub async fn health_check(&mut self) -> Result<()> {
        // Try to ping the server
        match self.client.ping().await {
            Ok(_) => Ok(()),
            Err(_) => {
                // If ping fails, try initialize as some servers don't support ping
                match self.client.initialize().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
        }
    }

    pub async fn initialize(&mut self) -> Result<serde_json::Value> {
        self.client.initialize().await
    }

    pub async fn list_tools(&mut self) -> Result<serde_json::Value> {
        self.client.list_tools().await
    }

    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.client.call_tool(tool_name, arguments).await
    }

    pub async fn get_stderr(&mut self) -> Vec<String> {
        self.client.get_stderr_output().await
    }

    pub async fn shutdown(mut self) -> Result<()> {
        println!("Shutting down server: {}", self.config.name);
        self.client.shutdown().await
    }
}

pub async fn check_runtime_available(runtime: &str) -> bool {
    match runtime {
        "node" | "npm" | "npx" => check_command_available("npx", &["--version"]).await,
        "python" | "uvx" => {
            check_command_available("uvx", &["--version"]).await
                || check_command_available("python", &["--version"]).await
        }
        "docker" => check_command_available("docker", &["--version"]).await,
        _ => false,
    }
}

pub async fn check_command_available(command: &str, args: &[&str]) -> bool {
    match Command::new(command).args(args).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn wait_for_server_ready(server: &mut TestServer, max_attempts: u32) -> Result<()> {
    let mut attempts = 0;
    while attempts < max_attempts {
        if server.health_check().await.is_ok() {
            return Ok(());
        }
        attempts += 1;
        sleep(Duration::from_millis(500)).await;
    }
    Err(anyhow::anyhow!(
        "Server not ready after {} attempts",
        max_attempts
    ))
}

// Helper function to create common server configurations
pub fn create_npx_server_config(package: &str, args: Vec<String>) -> ServerConfig {
    let mut full_args = vec!["-y".to_string(), package.to_string()];
    full_args.extend(args);

    ServerConfig::new(&format!("npx-{package}"), "npx", full_args).with_setup_delay(2000)
    // NPX servers need more time to download and start
}

pub fn create_uvx_server_config(package: &str, args: Vec<String>) -> ServerConfig {
    let mut full_args = vec![package.to_string()];
    full_args.extend(args);

    ServerConfig::new(&format!("uvx-{package}"), "uvx", full_args).with_setup_delay(3000)
    // UVX servers need time to install dependencies
}

pub fn create_docker_server_config(image: &str, args: Vec<String>) -> ServerConfig {
    let mut full_args = vec![
        "run".to_string(),
        "-i".to_string(),
        "--rm".to_string(),
        image.to_string(),
    ];
    full_args.extend(args);

    ServerConfig::new(&format!("docker-{image}"), "docker", full_args).with_setup_delay(5000)
    // Docker servers need time to pull and start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_runtime_available() {
        // These tests will only pass if the runtimes are installed
        // In CI, we'll ensure these are available
        if check_runtime_available("node").await {
            println!("Node.js runtime is available");
        }
        if check_runtime_available("docker").await {
            println!("Docker runtime is available");
        }
    }
}
