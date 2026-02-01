//! MCP server management commands
//!
//! The MCP server runs as a background process that IDEs can connect to
//! for triggering CTO workflows.

use std::process::{Child as ProcessChild, Command, Stdio};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};

/// MCP server process state
pub struct McpState {
    process: Mutex<Option<ProcessChild>>,
}

impl McpState {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }
}

impl Default for McpState {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP server status
#[derive(Debug, Clone, Serialize)]
pub struct McpStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub socket_path: String,
}

/// Get the MCP socket path
fn get_socket_path() -> String {
    #[cfg(target_os = "windows")]
    {
        r"\\.\pipe\cto-lite-mcp".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{home}/.cto-lite/mcp.sock")
    }
}

/// Get the path to the mcp-lite binary
fn get_mcp_binary_path() -> std::path::PathBuf {
    // In development, use cargo build path
    if cfg!(debug_assertions) {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("mcp-lite/target/debug/mcp-lite")
    } else {
        // In production, bundled alongside the app
        #[cfg(target_os = "macos")]
        {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Resources/mcp-lite")
        }
        #[cfg(target_os = "windows")]
        {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("mcp-lite.exe")
        }
        #[cfg(target_os = "linux")]
        {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("mcp-lite")
        }
    }
}

/// Start the MCP server
#[tauri::command]
pub async fn start_mcp_server(state: State<'_, McpState>) -> AppResult<McpStatus> {
    let mut process_guard = state
        .process
        .lock()
        .map_err(|e| AppError::CommandFailed(format!("Failed to acquire lock: {e}")))?;

    // Check if already running
    if let Some(ref mut child) = *process_guard {
        match child.try_wait() {
            Ok(None) => {
                // Still running
                return Ok(McpStatus {
                    running: true,
                    pid: Some(child.id()),
                    socket_path: get_socket_path(),
                });
            }
            _ => {
                // Process exited, clear it
                *process_guard = None;
            }
        }
    }

    let binary_path = get_mcp_binary_path();
    tracing::info!("Starting MCP server: {:?}", binary_path);

    // Create socket directory
    #[cfg(not(target_os = "windows"))]
    {
        let socket_path = get_socket_path();
        if let Some(parent) = std::path::Path::new(&socket_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let child = Command::new(&binary_path)
        .env("CTO_NAMESPACE", "cto-lite")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to start MCP server: {e}")))?;

    let pid = child.id();
    *process_guard = Some(child);

    tracing::info!("MCP server started with PID: {}", pid);

    Ok(McpStatus {
        running: true,
        pid: Some(pid),
        socket_path: get_socket_path(),
    })
}

/// Stop the MCP server
#[tauri::command]
pub async fn stop_mcp_server(state: State<'_, McpState>) -> AppResult<McpStatus> {
    let mut process_guard = state
        .process
        .lock()
        .map_err(|e| AppError::CommandFailed(format!("Failed to acquire lock: {e}")))?;

    if let Some(mut child) = process_guard.take() {
        tracing::info!("Stopping MCP server (PID: {})", child.id());
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(McpStatus {
        running: false,
        pid: None,
        socket_path: get_socket_path(),
    })
}

/// Get MCP server status
#[tauri::command]
pub async fn get_mcp_status(state: State<'_, McpState>) -> AppResult<McpStatus> {
    let mut process_guard = state
        .process
        .lock()
        .map_err(|e| AppError::CommandFailed(format!("Failed to acquire lock: {e}")))?;

    if let Some(ref mut child) = *process_guard {
        match child.try_wait() {
            Ok(None) => {
                // Still running
                return Ok(McpStatus {
                    running: true,
                    pid: Some(child.id()),
                    socket_path: get_socket_path(),
                });
            }
            _ => {
                // Process exited
                *process_guard = None;
            }
        }
    }

    Ok(McpStatus {
        running: false,
        pid: None,
        socket_path: get_socket_path(),
    })
}

/// Get MCP configuration for IDE integration
#[tauri::command]
pub async fn get_mcp_config() -> AppResult<serde_json::Value> {
    let config = serde_json::json!({
        "mcpServers": {
            "cto-lite": {
                "command": get_mcp_binary_path().to_string_lossy(),
                "args": [],
                "env": {
                    "CTO_NAMESPACE": "cto-lite"
                }
            }
        }
    });

    Ok(config)
}
