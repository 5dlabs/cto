//! Desktop MCP Server - Tauri-compatible MCP server using stdio
//!
//! This module implements an MCP server that communicates via stdio,
//! designed to be spawned by Tauri as a child process.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

/// MCP Protocol Messages
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
enum McpRequest {
    Initialize {
        #[allow(dead_code)]
        params: Option<Value>,
    },
    #[allow(dead_code)]
    ListTools {
        #[allow(dead_code)]
        params: Option<Value>,
    },
    CallTool {
        name: String,
        arguments: Option<HashMap<String, Value>>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<McpError>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Server configuration for MCP Lite
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Kubernetes namespace for workflows
    pub namespace: String,
    /// Argo Workflows template for play
    pub play_template: String,
    /// Argo Workflows template for intake
    pub intake_template: String,
    /// Default model for tasks
    pub default_model: String,
    /// Path to kubectl
    pub kubectl_path: String,
    /// Path to argo CLI
    pub argo_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            namespace: "cto".to_string(),
            play_template: "workflowtemplate/play-template".to_string(),
            intake_template: "workflowtemplate/intake-template".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            kubectl_path: "kubectl".to_string(),
            argo_path: "argo".to_string(),
        }
    }
}

/// Result type for tool calls
#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

/// Desktop MCP Server implementation
pub struct DesktopServer {
    config: Arc<ServerConfig>,
}

impl DesktopServer {
    /// Create a new DesktopServer with default configuration
    pub fn new() -> Self {
        Self::with_config(ServerConfig::default())
    }

    /// Create a new DesktopServer with custom configuration
    pub fn with_config(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Run the server using stdio communication
    /// This is the main entry point for Tauri integration
    pub fn run_stdio(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdin_lock = stdin.lock();

        let mut line = String::new();

        loop {
            line.clear();

            match stdin_lock.read_line(&mut line) {
                Ok(0) => {
                    // EOF - stdin closed
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Parse and handle request
                    match self.handle_request(trimmed) {
                        Ok(response) => {
                            // Write response to stdout
                            let stdout = std::io::stdout();
                            let mut stdout_lock = stdout.lock();
                            writeln!(stdout_lock, "{}", response)
                                .context("Failed to write response")?;
                            stdout_lock.flush().context("Failed to flush stdout")?;
                        }
                        Err(e) => {
                            eprintln!("Error handling request: {:#}", e);
                            let error_response = self.create_error_response(
                                None,
                                -32603,
                                &format!("Internal error: {:#}", e),
                                None,
                            );
                            let stdout = std::io::stdout();
                            let mut stdout_lock = stdout.lock();
                            writeln!(stdout_lock, "{}", error_response)
                                .context("Failed to write error response")?;
                            stdout_lock.flush().context("Failed to flush stdout")?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {:#}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a single MCP request
    fn handle_request(&self, raw: &str) -> Result<String> {
        let request: McpRequest = serde_json::from_str(raw)
            .with_context(|| format!("Failed to parse request: {}", raw))?;

        match request {
            McpRequest::Initialize { .. } => {
                Ok(self.create_success_response(None, self.create_initialize_result()))
            }
            McpRequest::ListTools { .. } => {
                let tools = self.get_tool_schemas();
                Ok(self.create_success_response(None, tools))
            }
            McpRequest::CallTool { name, arguments } => {
                let result = self.execute_tool(&name, arguments.as_ref());
                match result {
                    Ok(value) => Ok(self.create_success_response(None, value)),
                    Err(e) => Ok(self.create_error_response(
                        None,
                        -32603,
                        &format!("Tool execution failed: {:#}", e),
                        None,
                    )),
                }
            }
            McpRequest::Unknown => Ok(self.create_error_response(
                None,
                -32600,
                "Invalid request method",
                None,
            )),
        }
    }

    /// Create the initialize response
    fn create_initialize_result(&self) -> Value {
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "cto-mcp-lite",
                "title": "CTO Lite MCP Server",
                "version": "0.1.0"
            }
        })
    }

    /// Get tool schemas for the server
    fn get_tool_schemas(&self) -> Value {
        json!({
            "tools": [
                self.get_play_schema(),
                self.get_play_status_schema(),
                self.get_jobs_schema(),
                self.get_stop_job_schema(),
                self.get_input_schema(),
                self.get_check_setup_schema(),
                self.get_task_schema()
            ]
        })
    }

    /// Execute a tool by name
    fn execute_tool(&self, name: &str, arguments: Option<&HashMap<String, Value>>) -> Result<Value> {
        let args = arguments.cloned().unwrap_or_default();

        match name {
            "play" => self.execute_play(&args),
            "play_status" => self.execute_play_status(&args),
            "jobs" => self.execute_jobs(&args),
            "stop_job" => self.execute_stop_job(&args),
            "input" => self.execute_input(&args),
            "check_setup" => self.execute_check_setup(&args),
            "task" => self.execute_task(&args),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    // ========== Tool Implementations ==========

    /// Execute play workflow
    fn execute_play(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let task_id = args.get("task_id").and_then(|v| v.as_u64()).map(|v| v as u32);
        let repository = args.get("repository").and_then(|v| v.as_str()).map(|s| s.to_string());
        let repository_path = args.get("repository_path").and_then(|v| v.as_str()).map(|s| s.to_string());
        let model = args.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
        let service = args.get("service").and_then(|v| v.as_str()).map(|s| s.to_string());

        // Build argo command with owned strings
        let mut argo_args: Vec<String> = vec![
            "submit".to_string(),
            "--from".to_string(),
            self.config.play_template.clone(),
            "-n".to_string(),
            self.config.namespace.clone(),
        ];

        // Add parameters
        if let Some(ref repo) = repository {
            argo_args.push("-p".to_string());
            argo_args.push(format!("repository-url={}", repo));
        }

        if let Some(ref path) = repository_path {
            argo_args.push("-p".to_string());
            argo_args.push(format!("repository-path={}", path));
        }

        if let Some(tid) = task_id {
            argo_args.push("-p".to_string());
            argo_args.push(format!("task-id={}", tid));
        }

        if let Some(ref m) = model {
            argo_args.push("-p".to_string());
            argo_args.push(format!("model={}", m));
        } else {
            argo_args.push("-p".to_string());
            argo_args.push(format!("model={}", self.config.default_model));
        }

        if let Some(ref svc) = service {
            argo_args.push("-p".to_string());
            argo_args.push(format!("service={}", svc));
        }

        // Execute argo CLI
        let output = Command::new(&self.config.argo_path)
            .args(&argo_args)
            .output()
            .context("Failed to execute argo command")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(json!({
                "success": true,
                "message": "Play workflow submitted successfully",
                "output": stdout.trim(),
                "parameters": args
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Argo command failed: {}", stderr))
        }
    }

    /// Get play workflow status
    fn execute_play_status(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let repository = args.get("repository").and_then(|v| v.as_str());

        // Get workflows from Kubernetes
        let kubectl_args = vec![
            "get", "workflows",
            "-n", &self.config.namespace,
            "-o", "json"
        ];

        let output = Command::new(&self.config.kubectl_path)
            .args(&kubectl_args)
            .output()
            .context("Failed to execute kubectl command")?;

        if output.status.success() {
            let workflows: Value = serde_json::from_slice(&output.stdout)?;
            Ok(json!({
                "success": true,
                "workflows": workflows,
                "repository": repository
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Kubectl command failed: {}", stderr))
        }
    }

    /// List running jobs
    fn execute_jobs(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("cto");

        let kubectl_args = vec![
            "get", "workflows",
            "-n", namespace,
            "-o", "json"
        ];

        let output = Command::new(&self.config.kubectl_path)
            .args(&kubectl_args)
            .output()
            .context("Failed to execute kubectl command")?;

        if output.status.success() {
            let workflows: Value = serde_json::from_slice(&output.stdout)?;
            Ok(json!({
                "success": true,
                "jobs": workflows
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Kubectl command failed: {}", stderr))
        }
    }

    /// Stop a running job
    fn execute_stop_job(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let name = args.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;
        let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("cto");

        let kubectl_args = vec![
            "delete", "workflow", name,
            "-n", namespace
        ];

        let output = Command::new(&self.config.kubectl_path)
            .args(&kubectl_args)
            .output()
            .context("Failed to execute kubectl command")?;

        if output.status.success() {
            Ok(json!({
                "success": true,
                "message": format!("Workflow {} stopped successfully", name),
                "name": name
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Kubectl command failed: {}", stderr))
        }
    }

    /// Send input to a running job
    fn execute_input(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let text = args.get("text").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("cto");

        // Write input to a ConfigMap or use kubectl exec
        Ok(json!({
            "success": true,
            "message": "Input sent (placeholder - requires additional implementation)",
            "text": text,
            "namespace": namespace
        }))
    }

    /// Check setup and dependencies
    fn execute_check_setup(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let verbose = args.get("verbose").and_then(|v| v.as_bool()).unwrap_or(false);

        let mut results = Vec::new();

        // Check kubectl
        let kubectl_output = Command::new(&self.config.kubectl_path)
            .arg("version")
            .output()
            .ok();

        let kubectl_ok = kubectl_output.as_ref().map(|o| o.status.success()).unwrap_or(false);
        results.push(json!({
            "name": "kubectl",
            "installed": kubectl_ok,
            "version": if verbose && kubectl_ok {
                Some(String::from_utf8_lossy(&kubectl_output.unwrap().stdout).to_string())
            } else {
                None
            }
        }));

        // Check argo
        let argo_output = Command::new(&self.config.argo_path)
            .arg("version")
            .output()
            .ok();

        let argo_ok = argo_output.as_ref().map(|o| o.status.success()).unwrap_or(false);
        results.push(json!({
            "name": "argo",
            "installed": argo_ok,
            "version": if verbose && argo_ok {
                Some(String::from_utf8_lossy(&argo_output.unwrap().stdout).to_string())
            } else {
                None
            }
        }));

        // Check cluster connectivity
        let cluster_output = Command::new(&self.config.kubectl_path)
            .args(["cluster-info", "--request-timeout=5s"])
            .output()
            .ok();

        let cluster_ok = cluster_output.as_ref().map(|o| o.status.success()).unwrap_or(false);
        results.push(json!({
            "name": "kubernetes-cluster",
            "connected": cluster_ok
        }));

        let all_ok = kubectl_ok && argo_ok && cluster_ok;

        Ok(json!({
            "success": all_ok,
            "checks": results,
            "summary": if all_ok {
                "All dependencies satisfied"
            } else {
                "Some dependencies are missing"
            }
        }))
    }

    /// Task operations (read/update task files)
    fn execute_task(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let task_id = args.get("task_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: task_id"))?;
        let operation = args.get("operation").and_then(|v| v.as_str()).unwrap_or("read");

        match operation {
            "read" => {
                // Read task file from .tasks directory
                let task_path = Path::new(".tasks").join(format!("task-{}.json", task_id));

                if task_path.exists() {
                    let content = std::fs::read_to_string(&task_path)
                        .context("Failed to read task file")?;
                    let task: Value = serde_json::from_str(&content)?;

                    Ok(json!({
                        "success": true,
                        "operation": "read",
                        "task_id": task_id,
                        "task": task
                    }))
                } else {
                    Err(anyhow::anyhow!("Task file not found: {}", task_path.display()))
                }
            }
            "update" => {
                let task_content = args.get("content").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content for update"))?;

                let task_path = Path::new(".tasks").join(format!("task-{}.json", task_id));

                // Ensure .tasks directory exists
                if let Some(parent) = task_path.parent() {
                    std::fs::create_dir_all(parent)
                        .context("Failed to create .tasks directory")?;
                }

                std::fs::write(&task_path, task_content)
                    .context("Failed to write task file")?;

                Ok(json!({
                    "success": true,
                    "operation": "update",
                    "task_id": task_id,
                    "message": "Task file updated successfully"
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown operation: {}", operation)),
        }
    }

    // ========== Tool Schema Definitions ==========

    fn get_play_schema(&self) -> Value {
        json!({
            "name": "play",
            "description": "Submit a Play workflow for multi-agent orchestration. Executes Argo workflow for task implementation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "integer",
                        "description": "Task ID to implement (optional - auto-detects next available task)"
                    },
                    "repository": {
                        "type": "string",
                        "description": "Target repository URL (e.g., org/repo)"
                    },
                    "repository_path": {
                        "type": "string",
                        "description": "Absolute path to repository on disk"
                    },
                    "service": {
                        "type": "string",
                        "description": "Service identifier for persistent workspace"
                    },
                    "model": {
                        "type": "string",
                        "description": "AI model to use (optional, defaults to configuration)"
                    }
                }
            }
        })
    }

    fn get_play_status_schema(&self) -> Value {
        json!({
            "name": "play_status",
            "description": "Query current play workflow status and progress.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repository": {
                        "type": "string",
                        "description": "Filter by repository URL"
                    }
                }
            }
        })
    }

    fn get_jobs_schema(&self) -> Value {
        json!({
            "name": "jobs",
            "description": "List running Argo workflows.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": {
                        "type": "string",
                        "description": "Kubernetes namespace (default: cto)"
                    }
                }
            }
        })
    }

    fn get_stop_job_schema(&self) -> Value {
        json!({
            "name": "stop_job",
            "description": "Stop a running Argo workflow.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Workflow name to stop"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Kubernetes namespace (default: cto)"
                    }
                },
                "required": ["name"]
            }
        })
    }

    fn get_input_schema(&self) -> Value {
        json!({
            "name": "input",
            "description": "Send a live user message to a running job.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Message to send"
                    },
                    "namespace": {
                        "type": "string",
                        "description": "Kubernetes namespace (default: cto)"
                    }
                },
                "required": ["text"]
            }
        })
    }

    fn get_check_setup_schema(&self) -> Value {
        json!({
            "name": "check_setup",
            "description": "Check MCP server dependencies (kubectl, argo) and cluster connectivity.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "verbose": {
                        "type": "boolean",
                        "description": "Show detailed version information"
                    }
                }
            }
        })
    }

    fn get_task_schema(&self) -> Value {
        json!({
            "name": "task",
            "description": "Read or update task files in .tasks directory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Task ID (e.g., '1' for task-1.json)"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["read", "update"],
                        "description": "Operation to perform"
                    },
                    "content": {
                        "type": "string",
                        "description": "Task JSON content (for update operation)"
                    }
                },
                "required": ["task_id", "operation"]
            }
        })
    }

    // ========== Response Helpers ==========

    fn create_success_response(&self, id: Option<Value>, result: Value) -> String {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        };
        serde_json::to_string(&response).unwrap_or_else(|_| String::from("{}"))
    }

    fn create_error_response(&self, id: Option<Value>, code: i32, message: &str, data: Option<Value>) -> String {
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpError {
                code,
                message: message.to_string(),
                data,
            }),
            id,
        };
        serde_json::to_string(&response).unwrap_or_else(|_| String::from("{}"))
    }
}

/// Spawn the MCP server as a background task for Tauri
pub fn spawn_server(config: Option<ServerConfig>) -> tokio::task::JoinHandle<()> {
    let config = config.unwrap_or_default();

    tokio::spawn(async move {
        let server = DesktopServer::with_config(config);
        if let Err(e) = server.run_stdio() {
            eprintln!("MCP server error: {:#}", e);
        }
    })
}
