#![allow(clippy::uninlined_format_args)]

use crate::config::ClientConfig;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Local server process handle
#[derive(Debug)]
pub struct LocalServerProcess {
    pub child: Child,
    pub name: String,
    pub started_at: std::time::SystemTime,
    pub stdin: Option<tokio::process::ChildStdin>,
    pub stdout: Option<BufReader<tokio::process::ChildStdout>>,
    pub tools: Vec<Value>, // Store actual tool schemas from MCP handshake
}

pub struct McpClient {
    http_base_url: String,
    client: reqwest::Client,
    rt: Runtime,
    working_dir: Option<String>,
    client_config: Option<ClientConfig>,
    session_id: String,
    local_servers: Arc<Mutex<HashMap<String, LocalServerProcess>>>,
}

impl McpClient {
    pub fn new(http_base_url: String, working_dir: Option<String>) -> Result<Self> {
        let client = reqwest::Client::new();
        let rt = Runtime::new()?;

        // Load new client configuration format
        let client_config = Self::load_client_config(&working_dir)?;

        // Generate session ID
        let session_id = format!("client-{}", Uuid::new_v4());
        tracing::debug!("[Bridge] Generated session ID: {session_id}");

        Ok(Self {
            http_base_url,
            client,
            rt,
            working_dir,
            client_config,
            session_id,
            local_servers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Load new client configuration from environment variable or default location
    fn load_client_config(working_dir: &Option<String>) -> Result<Option<ClientConfig>> {
        let config_path = if let Ok(config_path) = std::env::var("MCP_CLIENT_CONFIG") {
            PathBuf::from(config_path)
        } else if let Some(dir) = working_dir {
            PathBuf::from(dir).join("client-config.json")
        } else {
            PathBuf::from("client-config.json")
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: ClientConfig = serde_json::from_str(&content)?;
            tracing::debug!(
                "[Bridge] Loaded client config from: {}",
                config_path.display()
            );
            tracing::debug!("[Bridge] Remote tools: {:?}", config.remote_tools);
            tracing::debug!(
                "[Bridge] Local servers: {:?}",
                config.local_servers.keys().collect::<Vec<_>>()
            );
            Ok(Some(config))
        } else {
            tracing::debug!(
                "[Bridge] No client config found at {}, using legacy mode",
                config_path.display()
            );
            Ok(None)
        }
    }

    /// Check if a tool should be included based on the new client configuration
    ///
    /// For remote tools:
    /// - If `remoteTools` is empty or missing → include ALL remote tools (no filtering)
    /// - If `remoteTools` has items → only include those specific tools (whitelist mode)
    ///
    /// For local tools:
    /// - Always check against explicit tool lists from local server configs
    fn should_include_tool(&self, tool_name: &str, is_local: bool) -> bool {
        if let Some(ref config) = self.client_config {
            if is_local {
                // Check if any local server exposes this tool
                config
                    .local_servers
                    .values()
                    .any(|server| server.tools.contains(&tool_name.to_string()))
            } else {
                // For remote tools: empty list means "include all" (no filtering)
                // This allows dynamic discovery without explicit whitelisting
                if config.remote_tools.is_empty() {
                    true
                } else {
                    // Explicit whitelist mode: only include tools in the list
                    config.remote_tools.contains(&tool_name.to_string())
                }
            }
        } else {
            // Legacy mode: include all tools
            true
        }
    }

    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        // Spawn local servers on startup
        if let Err(e) = self.rt.block_on(async { self.spawn_local_servers().await }) {
            tracing::debug!("[Bridge] Warning: Failed to spawn some local servers: {e}");
        }

        // Perform MCP handshakes with local servers
        if let Err(e) = self
            .rt
            .block_on(async { self.handshake_local_servers().await })
        {
            tracing::debug!("[Bridge] Warning: Failed to handshake with some local servers: {e}");
        }

        // Send initial capabilities
        self.send_capabilities(&mut stdout)?;

        // Set up cleanup handler with static reference
        let cleanup_servers = self.local_servers.clone();
        std::thread::spawn(move || {
            ctrlc::set_handler(move || {
                tracing::debug!("[Bridge] 🛑 Received shutdown signal, cleaning up...");
                let rt = Runtime::new().expect("Failed to create cleanup runtime");
                rt.block_on(async {
                    let mut servers = cleanup_servers.lock().await;
                    for (server_name, mut process) in servers.drain() {
                        tracing::debug!("[Bridge] 🔚 Stopping local server: {server_name}");
                        if let Err(e) = process.child.kill().await {
                            tracing::debug!(
                                "[Bridge] Warning: Failed to kill server '{server_name}': {e}"
                            );
                        }
                    }
                });
                std::process::exit(0);
            })
            .expect("Failed to set cleanup handler");
        });

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&line) {
                Ok(request) => {
                    if let Some(response) = self.handle_request(&request)? {
                        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                        stdout.flush()?;
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to parse JSON: {e}");
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": "Parse error"
                        }
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                }
            }
        }

        // Cleanup on normal exit
        tracing::debug!("[Bridge] 🛑 Normal shutdown, cleaning up local servers...");
        self.rt
            .block_on(async { self.cleanup_local_servers().await })?;

        Ok(())
    }

    fn send_capabilities(&self, stdout: &mut io::Stdout) -> Result<()> {
        // Get current tools from HTTP server
        let tools = self
            .rt
            .block_on(async { self.get_tools_from_http().await })?;

        tracing::debug!("[Bridge] Notifying clients about initial tools...");
        tracing::debug!("[Bridge] Initial tool count: {}", tools.len());

        // Send notification with empty params like working Node.js version
        let capabilities = json!({
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed",
            "params": {}
        });

        writeln!(stdout, "{}", serde_json::to_string(&capabilities)?)?;
        stdout.flush()?;
        tracing::debug!("[Bridge] Initial tools list changed notification sent successfully");
        Ok(())
    }

    async fn get_tools_from_http(&self) -> Result<Vec<Value>> {
        // Check if HTTP server is running and get tools
        tracing::debug!("[Bridge] Making HTTP request to: {}", self.http_base_url);

        // Get working directory to pass as context (canonicalized for consistency)
        let current_dir = if let Some(ref dir) = self.working_dir {
            // Use provided working directory, canonicalized
            std::path::Path::new(dir)
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| dir.clone())
        } else {
            // Fall back to current working directory
            std::env::current_dir()
                .map(|d| d.canonicalize().unwrap_or(d).to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        };

        tracing::debug!("[Bridge] Sending working directory: {current_dir}");

        let response = self
            .client
            .post(&self.http_base_url)
            .header("X-Session-ID", &self.session_id)
            .header("X-Working-Directory", &current_dir)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }))
            .send()
            .await?;

        tracing::debug!("[Bridge] HTTP response status: {}", response.status());

        let response_text = response.text().await?;
        let json_response: Value = serde_json::from_str(&response_text)?;

        if let Some(result) = json_response.get("result") {
            if let Some(tools) = result.get("tools") {
                if let Ok(tools_vec) = serde_json::from_value::<Vec<Value>>(tools.clone()) {
                    // Apply client-side filtering based on new configuration
                    let filtered_remote_tools = self.filter_remote_tools(tools_vec);

                    // Get local tools from MCP handshake results
                    let local_tools = self.get_local_tools().await;

                    // Combine remote and local tools
                    let mut all_tools = filtered_remote_tools;
                    all_tools.extend(local_tools);

                    // Apply Cursor-specific compatibility fixes
                    let cursor_compatible_tools = self.apply_cursor_compatibility(all_tools);
                    return Ok(cursor_compatible_tools);
                }
            }
        }

        Ok(vec![])
    }

    /// Filter remote tools based on the new client configuration
    fn filter_remote_tools(&self, tools: Vec<Value>) -> Vec<Value> {
        tools
            .into_iter()
            .filter(|tool| {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    // Sanitize the tool name (replace hyphens with underscores) before checking
                    // This ensures we match the config which uses sanitized names
                    let sanitized_name = name.replace('-', "_");
                    let enabled = self.should_include_tool(&sanitized_name, false);
                    if !enabled {
                        tracing::debug!("[Bridge] Filtering out remote tool: {name}");
                    }
                    enabled
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get local tools from actual MCP handshake results
    async fn get_local_tools(&self) -> Vec<Value> {
        let servers = self.local_servers.lock().await;

        let mut all_tools = Vec::new();

        for (server_name, process) in servers.iter() {
            for tool in &process.tools {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    // Apply the same filtering logic as remote tools
                    let enabled = self.should_include_tool(name, true);
                    if !enabled {
                        tracing::debug!("[Bridge] Filtering out local tool: {name}");
                        continue;
                    }
                }

                let mut tool_with_source = tool.clone();
                tool_with_source["_source"] = json!(format!("local:{}", server_name));
                all_tools.push(tool_with_source);
            }
        }

        all_tools
    }

    fn apply_cursor_compatibility(&self, tools: Vec<Value>) -> Vec<Value> {
        tools
            .into_iter()
            .map(|mut tool| {
                // Fix 1: Sanitize tool names (replace dashes with underscores)
                let original_name = tool
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());
                if let Some(name) = &original_name {
                    if name.contains('-') {
                        let sanitized_name = name.replace('-', "_");
                        tool["name"] = json!(sanitized_name);
                        tracing::debug!(
                            "[Bridge] Cursor compatibility: Sanitized tool name {name} -> {sanitized_name}"
                        );
                    }
                }

                // Fix 2: Ensure description is always a string (Cursor requirement)
                if tool.get("description").is_none() {
                    tool["description"] = json!("Tool description");
                }

                // Fix 3: Ensure inputSchema is always present (Cursor requirement)
                if tool.get("inputSchema").is_none() {
                    tool["inputSchema"] = json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    });
                }

                tool
            })
            .collect()
    }

    fn handle_request(&self, request: &Value) -> Result<Option<Value>> {
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let id = request.get("id").cloned();

        match method {
            "initialize" => {
                tracing::debug!(
                    "[Bridge] Client-side initialization with session ID: {}",
                    self.session_id
                );

                // Return standard MCP response - no complex session handshake needed
                Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {
                                "listChanged": true
                            }
                        },
                        "serverInfo": {
                            "name": "tools-mcp-client",
                            "version": "1.0.0"
                        }
                    }
                })))
            }
            "notifications/initialized" => {
                // This is a notification, no response needed
                Ok(None)
            }
            "tools/list" => {
                let tools = self
                    .rt
                    .block_on(async { self.get_tools_from_http().await.unwrap_or_default() });

                Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                })))
            }
            "tools/call" => {
                // Route tool calls based on local vs remote
                let result = self
                    .rt
                    .block_on(async { self.route_tool_call(request).await });

                match result {
                    Ok(response) => Ok(Some(response)),
                    Err(e) => Ok(Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32603,
                            "message": format!("Tool routing error: {e}")
                        }
                    }))),
                }
            }
            _ => {
                // Unknown method
                Ok(Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                })))
            }
        }
    }

    async fn forward_tool_call(&self, request: &Value) -> Result<Value> {
        // Get working directory to pass as context
        let current_dir = if let Some(ref dir) = self.working_dir {
            std::path::Path::new(dir)
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| dir.clone())
        } else {
            std::env::current_dir()
                .map(|d| d.canonicalize().unwrap_or(d).to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        };

        let response = self
            .client
            .post(&self.http_base_url)
            .header("X-Session-ID", &self.session_id)
            .header("X-Working-Directory", &current_dir)
            .json(request)
            .send()
            .await?;

        let response_text = response.text().await?;
        let json_response: Value = serde_json::from_str(&response_text)?;

        Ok(json_response)
    }

    /// Spawn all local servers defined in client configuration
    async fn spawn_local_servers(&self) -> Result<()> {
        if let Some(ref config) = self.client_config {
            tracing::debug!(
                "[Bridge] Spawning {} local servers...",
                config.local_servers.len()
            );

            for (server_name, server_config) in &config.local_servers {
                match self.spawn_local_server(server_name, server_config).await {
                    Ok(()) => {
                        tracing::debug!(
                            "[Bridge] ✅ Successfully spawned local server: {}",
                            server_name
                        );
                    }
                    Err(e) => {
                        tracing::debug!(
                            "[Bridge] ❌ Failed to spawn local server '{}': {}",
                            server_name,
                            e
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Perform MCP handshakes with all spawned local servers
    async fn handshake_local_servers(&self) -> Result<()> {
        if let Some(ref config) = self.client_config {
            tracing::debug!(
                "[Bridge] 🤝 Starting MCP handshakes with {} local servers...",
                config.local_servers.len()
            );

            for server_name in config.local_servers.keys() {
                match self.perform_mcp_handshake(server_name).await {
                    Ok(tools) => {
                        tracing::debug!(
                            "[Bridge] ✅ Handshake successful with '{}': {} tools",
                            server_name,
                            tools.len()
                        );
                    }
                    Err(e) => {
                        tracing::debug!(
                            "[Bridge] ❌ Handshake failed with '{}': {}",
                            server_name,
                            e
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Spawn a single local server process
    async fn spawn_local_server(
        &self,
        server_name: &str,
        config: &crate::config::LocalServerConfig,
    ) -> Result<()> {
        let working_dir = self.resolve_working_directory(&config.working_directory);

        tracing::debug!(
            "[Bridge] Spawning local server '{}' in {}",
            server_name,
            working_dir.display()
        );
        tracing::debug!(
            "[Bridge] Command: {} {}",
            config.command,
            config.args.join(" ")
        );

        // Build command
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .current_dir(&working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // Add environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Inherit parent environment
        cmd.envs(std::env::vars());

        // Add working directory environment variables for servers that need them
        cmd.env(
            "WORKING_DIRECTORY",
            working_dir.to_string_lossy().to_string(),
        );
        cmd.env("PROJECT_DIR", working_dir.to_string_lossy().to_string());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn '{}': {}", config.command, e))?;

        let pid = child.id().unwrap_or(0);
        let started_at = std::time::SystemTime::now();

        tracing::debug!(
            "[Bridge] ✅ Spawned local server '{}' with PID: {}",
            server_name,
            pid
        );

        // Take stdin and stdout for communication
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().map(BufReader::new);

        let local_process = LocalServerProcess {
            child,
            name: server_name.to_string(),
            started_at,
            stdin,
            stdout,
            tools: Vec::new(), // Initialize empty, will be populated during handshake
        };

        // Store the process
        let mut servers = self.local_servers.lock().await;
        servers.insert(server_name.to_string(), local_process);

        Ok(())
    }

    /// Resolve working directory from configuration
    fn resolve_working_directory(&self, config_working_dir: &Option<String>) -> PathBuf {
        match config_working_dir.as_deref() {
            Some("project_root") | Some("project") => {
                // Use the client's working directory
                if let Some(ref dir) = self.working_dir {
                    PathBuf::from(dir)
                } else {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                }
            }
            Some(path) if path.starts_with('/') => {
                // Absolute path
                PathBuf::from(path)
            }
            Some(path) => {
                // Relative path - relative to client's working directory
                if let Some(ref dir) = self.working_dir {
                    PathBuf::from(dir).join(path)
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(path)
                }
            }
            None => {
                // Default to client's working directory
                if let Some(ref dir) = self.working_dir {
                    PathBuf::from(dir)
                } else {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                }
            }
        }
    }

    /// Cleanup all local servers (called on shutdown)
    pub async fn cleanup_local_servers(&self) -> Result<()> {
        let mut servers = self.local_servers.lock().await;

        tracing::debug!("[Bridge] Cleaning up {} local servers...", servers.len());

        for (server_name, mut process) in servers.drain() {
            tracing::debug!("[Bridge] 🔚 Stopping local server: {server_name}");

            if let Some(pid) = process.child.id() {
                tracing::debug!("[Bridge] Killing process PID: {}", pid);
            }

            if let Err(e) = process.child.kill().await {
                tracing::debug!(
                    "[Bridge] Warning: Failed to kill server '{}': {}",
                    server_name,
                    e
                );
            }
        }

        tracing::debug!("[Bridge] ✅ Local server cleanup completed");
        Ok(())
    }

    /// Execute MCP handshake with a local server
    async fn perform_mcp_handshake(&self, server_name: &str) -> Result<Vec<Value>> {
        let mut servers = self.local_servers.lock().await;

        if let Some(process) = servers.get_mut(server_name) {
            tracing::debug!(
                "[Bridge] 🤝 Starting MCP handshake with local server: {}",
                server_name
            );

            let stdin = process.stdin.as_mut().ok_or_else(|| {
                anyhow::anyhow!("No stdin available for server '{}'", server_name)
            })?;

            let stdout = process.stdout.as_mut().ok_or_else(|| {
                anyhow::anyhow!("No stdout available for server '{}'", server_name)
            })?;

            // Step 1: Send initialize request
            let initialize_request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "clientInfo": {
                        "name": "tools-mcp-client",
                        "version": "1.0.0"
                    }
                }
            });

            let request_line = format!("{}\n", initialize_request);
            tracing::debug!(
                "[Bridge] 📤 Sending initialize request to {}: {}",
                server_name,
                request_line.trim()
            );

            stdin
                .write_all(request_line.as_bytes())
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to send initialize request to '{}': {}",
                        server_name,
                        e
                    )
                })?;
            stdin.flush().await.map_err(|e| {
                anyhow::anyhow!("Failed to flush stdin for '{}': {}", server_name, e)
            })?;

            // Step 2: Read initialize response
            let mut response_line = String::new();
            stdout.read_line(&mut response_line).await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read initialize response from '{}': {}",
                    server_name,
                    e
                )
            })?;

            tracing::debug!(
                "[Bridge] 📥 Received initialize response from {}: {}",
                server_name,
                response_line.trim()
            );

            let response: Value = serde_json::from_str(&response_line).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse initialize response from '{}': {}",
                    server_name,
                    e
                )
            })?;

            // Validate initialize response
            if response.get("error").is_some() {
                return Err(anyhow::anyhow!(
                    "Initialize request to '{}' failed: {}",
                    server_name,
                    response.get("error").unwrap()
                ));
            }

            // Step 3: Send initialized notification
            let initialized_notification = json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            });

            let notification_line = format!("{}\n", initialized_notification);
            tracing::debug!(
                "[Bridge] 📤 Sending initialized notification to {}",
                server_name
            );

            stdin
                .write_all(notification_line.as_bytes())
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to send initialized notification to '{}': {}",
                        server_name,
                        e
                    )
                })?;
            stdin.flush().await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to flush stdin after notification for '{}': {}",
                    server_name,
                    e
                )
            })?;

            // Step 4: Send tools/list request
            let tools_list_request = json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            });

            let tools_request_line = format!("{}\n", tools_list_request);
            tracing::debug!("[Bridge] 📤 Sending tools/list request to {}", server_name);

            stdin
                .write_all(tools_request_line.as_bytes())
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to send tools/list request to '{}': {}",
                        server_name,
                        e
                    )
                })?;
            stdin.flush().await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to flush stdin after tools request for '{}': {}",
                    server_name,
                    e
                )
            })?;

            // Step 5: Read tools/list response
            let mut tools_response_line = String::new();
            stdout
                .read_line(&mut tools_response_line)
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to read tools/list response from '{}': {}",
                        server_name,
                        e
                    )
                })?;

            tracing::debug!(
                "[Bridge] 📥 Received tools/list response from {}: {}",
                server_name,
                tools_response_line.trim()
            );

            let tools_response: Value =
                serde_json::from_str(&tools_response_line).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to parse tools/list response from '{}': {}",
                        server_name,
                        e
                    )
                })?;

            // Validate tools/list response
            if tools_response.get("error").is_some() {
                return Err(anyhow::anyhow!(
                    "Tools/list request to '{}' failed: {}",
                    server_name,
                    tools_response.get("error").unwrap()
                ));
            }

            // Extract full tool schemas from response
            let mut tool_schemas = Vec::new();
            if let Some(result) = tools_response.get("result") {
                if let Some(tools) = result.get("tools") {
                    if let Some(tools_array) = tools.as_array() {
                        for tool in tools_array {
                            if tool.get("name").is_some() {
                                tool_schemas.push(tool.clone());
                            }
                        }
                    }
                }
            }

            // Store the tool schemas in the LocalServerProcess
            process.tools = tool_schemas.clone();

            let tool_names: Vec<String> = tool_schemas
                .iter()
                .filter_map(|tool| tool.get("name")?.as_str().map(|s| s.to_string()))
                .collect();

            tracing::debug!(
                "[Bridge] ✅ MCP handshake successful with {}: {} tools discovered",
                server_name,
                tool_schemas.len()
            );
            tracing::debug!("[Bridge] 🔍 Discovered tools: {:?}", tool_names);
            Ok(tool_schemas)
        } else {
            Err(anyhow::anyhow!("Local server '{}' not found", server_name))
        }
    }

    /// Execute a tool call on a local server
    async fn call_local_server(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value> {
        let mut servers = self.local_servers.lock().await;

        if let Some(process) = servers.get_mut(server_name) {
            tracing::debug!(
                "[Bridge] 🔧 Executing tool '{}' on local server '{}'",
                tool_name,
                server_name
            );

            let stdin = process.stdin.as_mut().ok_or_else(|| {
                anyhow::anyhow!("No stdin available for server '{}'", server_name)
            })?;

            let stdout = process.stdout.as_mut().ok_or_else(|| {
                anyhow::anyhow!("No stdout available for server '{}'", server_name)
            })?;

            // Create tools/call JSON-RPC request
            let request_id = 100; // Use a different ID from handshake
            let tools_call_request = json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": arguments
                }
            });

            let request_line = format!("{}\n", tools_call_request);
            tracing::debug!(
                "[Bridge] 📤 Sending tools/call request to {}: {}",
                server_name,
                request_line.trim()
            );

            // Send the request
            stdin
                .write_all(request_line.as_bytes())
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to send tools/call request to '{}': {}",
                        server_name,
                        e
                    )
                })?;
            stdin.flush().await.map_err(|e| {
                anyhow::anyhow!("Failed to flush stdin for '{}': {}", server_name, e)
            })?;

            // Read the response
            let mut response_line = String::new();
            stdout.read_line(&mut response_line).await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read tools/call response from '{}': {}",
                    server_name,
                    e
                )
            })?;

            tracing::debug!(
                "[Bridge] 📥 Received tools/call response from {}: {}",
                server_name,
                response_line.trim()
            );

            // Parse the JSON-RPC response
            let response: Value = serde_json::from_str(&response_line).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse tools/call response from '{}': {}",
                    server_name,
                    e
                )
            })?;

            // Check for JSON-RPC error
            if let Some(error) = response.get("error") {
                return Err(anyhow::anyhow!(
                    "Tool execution failed on '{}': {}",
                    server_name,
                    error
                ));
            }

            // Return the result
            let result = response.get("result").cloned().ok_or_else(|| {
                anyhow::anyhow!("No result in tools/call response from '{}'", server_name)
            })?;

            tracing::debug!("[Bridge] ✅ Tool execution successful on {}", server_name);
            Ok(result)
        } else {
            Err(anyhow::anyhow!("Local server '{}' not found", server_name))
        }
    }

    /// Get which local server provides a specific tool
    fn get_local_server_for_tool(&self, tool_name: &str) -> Option<String> {
        if let Some(ref config) = self.client_config {
            for (server_name, server_config) in &config.local_servers {
                if server_config.tools.contains(&tool_name.to_string()) {
                    return Some(server_name.clone());
                }
            }
        }
        None
    }

    /// Route tool call to local server or HTTP proxy based on configuration
    async fn route_tool_call(&self, request: &Value) -> Result<Value> {
        // Extract tool name from the request
        let tool_name = request
            .get("params")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow::anyhow!("Tool name not found in request"))?;

        let tool_arguments = request
            .get("params")
            .and_then(|p| p.get("arguments"))
            .cloned()
            .unwrap_or(json!({}));

        let request_id = request.get("id").cloned();

        tracing::debug!("[Bridge] 🔀 Routing tool call: {}", tool_name);

        // Check if this is a local tool
        if let Some(server_name) = self.get_local_server_for_tool(tool_name) {
            tracing::debug!(
                "[Bridge] 📍 Routing '{}' to local server: {}",
                tool_name,
                server_name
            );

            // Route to local server
            match self
                .call_local_server(&server_name, tool_name, tool_arguments)
                .await
            {
                Ok(result) => Ok(json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": result
                })),
                Err(e) => {
                    tracing::debug!("[Bridge] ❌ Local tool execution failed: {}", e);
                    Ok(json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32603,
                            "message": format!("Local tool execution failed: {}", e)
                        }
                    }))
                }
            }
        } else if self.should_include_tool(tool_name, false) {
            tracing::debug!("[Bridge] 🌐 Routing '{}' to remote HTTP proxy", tool_name);

            // Route to HTTP proxy
            self.forward_tool_call(request).await
        } else {
            tracing::debug!(
                "[Bridge] ❌ Tool '{}' not available in configuration",
                tool_name
            );

            Ok(json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32601,
                    "message": format!("Tool '{}' not available", tool_name)
                }
            }))
        }
    }
}
