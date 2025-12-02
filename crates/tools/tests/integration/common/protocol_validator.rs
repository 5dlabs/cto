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

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::common::server_lifecycle::TestServer;

pub struct ProtocolValidator;

impl ProtocolValidator {
    pub async fn validate_initialization(server: &mut TestServer) -> Result<Value> {
        println!(
            "Validating initialization for server: {}",
            server.config.name
        );

        let response = server
            .initialize()
            .await
            .context("Failed to initialize server")?;

        // Validate basic response structure
        if let Some(protocol_version) = response.get("protocolVersion") {
            println!("Protocol version: {protocol_version}");
        } else {
            return Err(anyhow::anyhow!(
                "Missing protocolVersion in initialize response"
            ));
        }

        if let Some(capabilities) = response.get("capabilities") {
            println!("Server capabilities: {capabilities}");
        } else {
            return Err(anyhow::anyhow!(
                "Missing capabilities in initialize response"
            ));
        }

        if let Some(server_info) = response.get("serverInfo") {
            println!("Server info: {server_info}");
        }

        Ok(response)
    }

    pub async fn validate_tool_listing(server: &mut TestServer) -> Result<Vec<String>> {
        println!("Validating tool listing for server: {}", server.config.name);

        let response = server.list_tools().await.context("Failed to list tools")?;

        let tools = response
            .get("tools")
            .and_then(|t| t.as_array())
            .context("Invalid tools response format")?;

        let mut tool_names = Vec::new();
        for tool in tools {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                tool_names.push(name.to_string());
                println!("Found tool: {name}");

                // Validate tool structure
                if tool.get("description").is_none() {
                    println!("Warning: Tool {name} missing description");
                }
                if tool.get("inputSchema").is_none() {
                    println!("Warning: Tool {name} missing inputSchema");
                }
            }
        }

        if tool_names.is_empty() {
            return Err(anyhow::anyhow!("No tools found in server response"));
        }

        Ok(tool_names)
    }

    pub async fn validate_basic_tool_call(
        server: &mut TestServer,
        tool_name: &str,
        args: Value,
    ) -> Result<Value> {
        println!("Validating tool call: {tool_name} with args: {args}");

        let response = server
            .call_tool(tool_name, args)
            .await
            .context(format!("Failed to call tool: {tool_name}"))?;

        // Validate response structure
        if response.get("content").is_none() {
            return Err(anyhow::anyhow!("Tool response missing content field"));
        }

        println!(
            "Tool call successful: {}",
            serde_json::to_string_pretty(&response)?
        );
        Ok(response)
    }

    pub async fn validate_error_handling(server: &mut TestServer) -> Result<()> {
        println!(
            "Validating error handling for server: {}",
            server.config.name
        );

        // Test invalid tool call
        let result = server.call_tool("non_existent_tool", json!({})).await;
        if result.is_ok() {
            println!("Warning: Server should have failed for non-existent tool");
        } else {
            println!("Correct: Server properly handled invalid tool call");
        }

        // Test invalid method
        let result = server
            .client
            .send_request("invalid_method", json!({}))
            .await;
        if let Ok(response) = result {
            if let Some(error) = response.error {
                println!("Correct: Server returned error for invalid method: {error}");
            } else {
                println!("Warning: Server should have returned error for invalid method");
            }
        }

        Ok(())
    }

    pub async fn run_comprehensive_validation(server: &mut TestServer) -> Result<ValidationReport> {
        let mut report = ValidationReport::new(&server.config.name);

        // Test initialization
        match Self::validate_initialization(server).await {
            Ok(response) => {
                report.initialization_passed = true;
                report.protocol_version = response
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
            }
            Err(e) => {
                report.add_error(format!("Initialization failed: {e}"));
            }
        }

        // Test tool listing
        match Self::validate_tool_listing(server).await {
            Ok(tools) => {
                report.tools_list_passed = true;
                report.available_tools = tools;
            }
            Err(e) => {
                report.add_error(format!("Tool listing failed: {e}"));
            }
        }

        // Test error handling
        match Self::validate_error_handling(server).await {
            Ok(_) => {
                report.error_handling_passed = true;
            }
            Err(e) => {
                report.add_error(format!("Error handling validation failed: {e}"));
            }
        }

        // Check stderr for any concerning messages
        let stderr_output = server.get_stderr().await;
        for line in stderr_output {
            if line.contains("error") || line.contains("ERROR") || line.contains("failed") {
                report.add_warning(format!("Stderr: {line}"));
            }
        }

        println!("Validation complete for server: {}", server.config.name);
        report.print_summary();

        Ok(report)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub server_name: String,
    pub initialization_passed: bool,
    pub tools_list_passed: bool,
    pub error_handling_passed: bool,
    pub protocol_version: String,
    pub available_tools: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn new(server_name: &str) -> Self {
        Self {
            server_name: server_name.to_string(),
            initialization_passed: false,
            tools_list_passed: false,
            error_handling_passed: false,
            protocol_version: "unknown".to_string(),
            available_tools: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_healthy(&self) -> bool {
        self.initialization_passed && self.tools_list_passed && self.errors.is_empty()
    }

    pub fn print_summary(&self) {
        println!("\n=== Validation Report for {} ===", self.server_name);
        println!(
            "Initialization: {}",
            if self.initialization_passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "Tools List: {}",
            if self.tools_list_passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!(
            "Error Handling: {}",
            if self.error_handling_passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            }
        );
        println!("Protocol Version: {}", self.protocol_version);
        println!("Available Tools: {}", self.available_tools.join(", "));

        if !self.errors.is_empty() {
            println!("Errors:");
            for error in &self.errors {
                println!("  ❌ {error}");
            }
        }

        if !self.warnings.is_empty() {
            println!("Warnings:");
            for warning in &self.warnings {
                println!("  ⚠️  {warning}");
            }
        }

        println!(
            "Overall Status: {}",
            if self.is_healthy() {
                "✅ HEALTHY"
            } else {
                "❌ UNHEALTHY"
            }
        );
        println!("====================================\n");
    }
}

// Common test scenarios for different server types
pub struct CommonTestScenarios;

impl CommonTestScenarios {
    pub async fn test_filesystem_server(server: &mut TestServer, test_dir: &str) -> Result<()> {
        println!("Running filesystem server tests...");

        // Test reading a file
        let read_result = server
            .call_tool(
                "read_file",
                json!({
                    "path": format!("{}/test.txt", test_dir)
                }),
            )
            .await;

        match read_result {
            Ok(response) => println!("File read successful: {response}"),
            Err(e) => println!("File read failed (expected if file doesn't exist): {e}"),
        }

        // Test listing directory
        let list_result = server
            .call_tool(
                "list_directory",
                json!({
                    "path": test_dir
                }),
            )
            .await;

        match list_result {
            Ok(response) => println!("Directory listing successful: {response}"),
            Err(e) => println!("Directory listing failed: {e}"),
        }

        Ok(())
    }

    pub async fn test_fetch_server(server: &mut TestServer) -> Result<()> {
        println!("Running fetch server tests...");

        // Test fetching a simple URL
        let fetch_result = server
            .call_tool(
                "fetch",
                json!({
                    "url": "https://httpbin.org/get",
                    "method": "GET"
                }),
            )
            .await;

        match fetch_result {
            Ok(response) => println!("Fetch successful: {response}"),
            Err(e) => println!("Fetch failed: {e}"),
        }

        Ok(())
    }
}
