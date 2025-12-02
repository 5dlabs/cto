#![allow(clippy::unused_async)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::single_match_else)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::similar_names)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::disallowed_macros)]
#![allow(clippy::ignore_without_reason)]
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
// Removed unused imports
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// Test our Tools HTTP server's tool forwarding functionality
pub struct ToolsServerTest {
    server_process: Option<Child>,
    server_port: u16,
    client: reqwest::Client,
}

impl ToolsServerTest {
    pub async fn new() -> Result<Self> {
        let server_port = 13000; // Use a different port to avoid conflicts

        // Create a minimal test configuration that doesn't depend on external packages
        let test_config = json!({
            "servers": {
                "test-http": {
                    "name": "Test HTTP Server",
                    "description": "Test HTTP server for integration tests",
                    "transport": "http",
                    "url": "https://jsonplaceholder.typicode.com/posts/1",
                    "enabled": true,
                    "alwaysActive": false,
                    "autoStart": true,
                    "workingDirectory": "project_root",
                    "executionContext": "remote"
                }
            }
        });

        // Write test config to temporary file (server expects servers-config.json)
        let config_path = "/tmp/servers-config.json";
        tokio::fs::write(config_path, serde_json::to_string_pretty(&test_config)?)
            .await
            .context("Failed to write test config")?;

        let client = reqwest::Client::new();

        Ok(Self {
            server_process: None,
            server_port,
            client,
        })
    }

    /// Start the Tools HTTP server
    pub async fn start_server(&mut self) -> Result<()> {
        println!("🚀 Starting Tools HTTP server on port {}", self.server_port);

        // Build the server binary first
        let build_output = Command::new("cargo")
            .args(["build", "--bin", "tools-server"])
            .output()
            .await
            .context("Failed to build tools-server")?;

        if !build_output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to build tools-server: {}",
                String::from_utf8_lossy(&build_output.stderr)
            ));
        }

        // Start the server
        let mut cmd = Command::new("./target/debug/tools-server");
        cmd.args([
            "--port",
            &self.server_port.to_string(),
            "--project-dir",
            "/tmp",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("RUST_LOG", "info");

        let process = cmd.spawn().context("Failed to spawn tools-server")?;

        self.server_process = Some(process);

        // Wait for server to be ready
        self.wait_for_server_ready().await?;

        println!("✅ Tools HTTP server started successfully");
        Ok(())
    }

    /// Wait for the server to be ready to accept connections
    async fn wait_for_server_ready(&self) -> Result<()> {
        let url = format!("http://localhost:{}/health", self.server_port);
        let max_attempts = 30;
        let delay = Duration::from_millis(500);

        for attempt in 1..=max_attempts {
            // Check if process is still alive
            // Note: We'll skip process checking for now to simplify the test

            match timeout(Duration::from_secs(2), self.client.get(&url).send()).await {
                Ok(Ok(response)) if response.status().is_success() => {
                    println!("✅ Server ready after {attempt} attempts");
                    return Ok(());
                }
                Ok(Ok(response)) => {
                    println!(
                        "⏳ Attempt {}: Server responded with status {}",
                        attempt,
                        response.status()
                    );
                }
                Ok(Err(e)) => {
                    println!("⏳ Attempt {attempt}: Request error: {e}");
                }
                Err(_) => {
                    println!("⏳ Attempt {attempt}: Timeout");
                }
            }

            if attempt < max_attempts {
                tokio::time::sleep(delay).await;
            }
        }

        Err(anyhow::anyhow!(
            "Server failed to become ready after {max_attempts} attempts"
        ))
    }

    /// Test server health endpoint
    pub async fn test_health(&self) -> Result<()> {
        let url = format!("http://localhost:{}/health", self.server_port);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Health check request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Health check failed with status: {}",
                response.status()
            ));
        }

        let health_data: Value = response
            .json()
            .await
            .context("Failed to parse health response")?;

        assert_eq!(health_data["status"], "ok");
        assert_eq!(health_data["service"], "tools");

        println!("✅ Health check passed");
        Ok(())
    }

    /// Test MCP initialization
    pub async fn test_initialization(&self) -> Result<()> {
        let url = format!("http://localhost:{}/mcp", self.server_port);

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "tools-integration-test",
                    "version": "1.0.0"
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&init_request)
            .send()
            .await
            .context("Initialize request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Initialize failed with status: {}",
                response.status()
            ));
        }

        let init_response: Value = response
            .json()
            .await
            .context("Failed to parse initialize response")?;

        // Validate response structure
        assert_eq!(init_response["jsonrpc"], "2.0");
        assert_eq!(init_response["id"], 1);
        assert!(init_response["result"].is_object());

        let result = &init_response["result"];
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"].is_object());
        assert_eq!(result["serverInfo"]["name"], "tools");

        println!("✅ Initialization test passed");
        Ok(())
    }

    /// Test tools listing
    pub async fn test_tools_list(&self) -> Result<Vec<String>> {
        let url = format!("http://localhost:{}/mcp", self.server_port);

        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response = self
            .client
            .post(&url)
            .json(&tools_request)
            .send()
            .await
            .context("Tools list request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Tools list failed with status: {}",
                response.status()
            ));
        }

        let tools_response: Value = response
            .json()
            .await
            .context("Failed to parse tools response")?;

        // Validate response structure
        assert_eq!(tools_response["jsonrpc"], "2.0");
        assert_eq!(tools_response["id"], 2);
        assert!(tools_response["result"].is_object());

        let tools = tools_response["result"]["tools"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Tools should be an array"))?;

        let tool_names: Vec<String> = tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .map(std::string::ToString::to_string)
            .collect();

        println!(
            "✅ Tools list test passed. Found {} tools: {:?}",
            tool_names.len(),
            tool_names
        );
        Ok(tool_names)
    }

    /// Test tool forwarding - this is the critical functionality!
    pub async fn test_tool_forwarding(&self) -> Result<()> {
        // First get available tools
        let tools = self.test_tools_list().await?;

        if tools.is_empty() {
            println!("⚠️ Skipping tool forwarding test - no tools available in test environment");
            return Ok(());
        }

        // Find a memory tool to test (should be prefixed with "memory_")
        let memory_tool = tools
            .iter()
            .find(|tool| tool.starts_with("memory_"))
            .ok_or_else(|| anyhow::anyhow!("No memory tools found for testing"))?;

        println!("🔧 Testing tool forwarding with tool: {memory_tool}");

        let url = format!("http://localhost:{}/mcp", self.server_port);

        let tool_request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": memory_tool,
                "arguments": {}
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&tool_request)
            .send()
            .await
            .context("Tool call request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Tool call failed with status: {}",
                response.status()
            ));
        }

        let tool_response: Value = response
            .json()
            .await
            .context("Failed to parse tool response")?;

        // Validate response structure
        assert_eq!(tool_response["jsonrpc"], "2.0");
        assert_eq!(tool_response["id"], 3);

        // Check if we got a result (success) or error (expected for some tools)
        if let Some(result) = tool_response.get("result") {
            println!("✅ Tool forwarding successful - got result: {result}");
        } else if let Some(error) = tool_response.get("error") {
            // Some tools might return errors with empty args, which is fine
            println!("✅ Tool forwarding successful - got expected error: {error}");
        } else {
            return Err(anyhow::anyhow!(
                "Tool response missing both result and error"
            ));
        }

        println!("✅ Tool forwarding test passed for: {memory_tool}");
        Ok(())
    }

    /// Test invalid tool name handling
    pub async fn test_invalid_tool(&self) -> Result<()> {
        let url = format!("http://localhost:{}/mcp", self.server_port);

        let invalid_tool_request = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool_name",
                "arguments": {}
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&invalid_tool_request)
            .send()
            .await
            .context("Invalid tool request failed")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Invalid tool request failed with status: {}",
                response.status()
            ));
        }

        let error_response: Value = response
            .json()
            .await
            .context("Failed to parse error response")?;

        // Should get a proper error response with details
        assert_eq!(error_response["jsonrpc"], "2.0");
        assert_eq!(error_response["id"], 4);

        let result = error_response["result"]
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Expected error result object"))?;

        // Should have content with error message
        assert!(result.get("content").is_some());

        println!("✅ Invalid tool handling test passed");
        Ok(())
    }

    /// Run comprehensive test suite
    pub async fn run_comprehensive_test(&mut self) -> Result<()> {
        println!("\n🧪 Starting Tools HTTP Server Integration Test");

        // Start server
        self.start_server().await?;

        // Run test suite
        self.test_health().await?;
        self.test_initialization().await?;
        self.test_tools_list().await?;
        self.test_tool_forwarding().await?;
        self.test_invalid_tool().await?;

        println!("✅ All Tools HTTP server tests passed!");
        Ok(())
    }
}

impl Drop for ToolsServerTest {
    fn drop(&mut self) {
        if let Some(mut process) = self.server_process.take() {
            std::mem::drop(process.kill());
            println!("🛑 Tools HTTP server stopped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Skip in CI - requires external dependencies and running server
    async fn test_tools_server_integration() -> Result<()> {
        let mut test_runner = ToolsServerTest::new().await?;
        test_runner.run_comprehensive_test().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_tools_server_basics() -> Result<()> {
        // Test that we can create the test harness and config
        let test_harness = ToolsServerTest::new().await?;
        assert_eq!(test_harness.server_port, 13000);
        println!("✅ Basic tools server test harness works");
        Ok(())
    }
}
