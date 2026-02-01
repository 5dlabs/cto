//! MCP Lite - Model Context Protocol server for CTO Lite
//!
//! This server runs as a background process and exposes tools for
//! IDEs to trigger CTO workflows.
//!
//! Protocol: JSON-RPC 2.0 over stdio
//!
//! Tools:
//! - `cto_trigger` - Trigger a workflow from a PRD
//! - `cto_status` - Get workflow status
//! - `cto_logs` - Stream workflow logs
//! - `cto_jobs` - List recent workflows

use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info};

mod k8s;
mod tools;

use tools::{handle_tool_call, list_tools};

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// MCP Server Info
#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

/// MCP Initialize Response
#[derive(Debug, Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
}

/// Server Capabilities
#[derive(Debug, Serialize)]
struct Capabilities {
    tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
struct ToolsCapability {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_lite=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("MCP Lite starting...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => handle_request(request).await,
            Err(e) => JsonRpcResponse::error(Value::Null, -32700, format!("Parse error: {e}")),
        };

        let response_str = serde_json::to_string(&response)?;
        debug!("Sending: {}", response_str);

        writeln!(stdout, "{response_str}")?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => {
            info!("Initialize request received");
            JsonRpcResponse::success(
                id,
                serde_json::to_value(InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: Capabilities {
                        tools: ToolsCapability {
                            list_changed: false,
                        },
                    },
                    server_info: ServerInfo {
                        name: "cto-lite".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                })
                .unwrap(),
            )
        }

        "initialized" => {
            info!("Client initialized");
            JsonRpcResponse::success(id, json!({}))
        }

        "tools/list" => {
            debug!("Tools list requested");
            JsonRpcResponse::success(id, json!({ "tools": list_tools() }))
        }

        "tools/call" => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            info!("Tool call: {}", name);

            match handle_tool_call(name, arguments).await {
                Ok(result) => JsonRpcResponse::success(id, result),
                Err(e) => {
                    error!("Tool error: {}", e);
                    JsonRpcResponse::error(id, -32000, e.to_string())
                }
            }
        }

        "ping" => JsonRpcResponse::success(id, json!({})),

        method => {
            error!("Unknown method: {}", method);
            JsonRpcResponse::error(id, -32601, format!("Method not found: {method}"))
        }
    }
}
