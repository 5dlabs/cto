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
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

pub struct McpTestClient {
    process: Option<Child>,
    request_id: AtomicU64,
    stdin_sender: mpsc::UnboundedSender<String>,
    stdout_receiver: mpsc::UnboundedReceiver<String>,
    stderr_receiver: mpsc::UnboundedReceiver<String>,
}

impl McpTestClient {
    pub async fn new(
        command: &str,
        args: &[String],
        working_directory: Option<&str>,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(working_dir) = working_directory {
            cmd.current_dir(working_dir);
        }

        let mut child = cmd.spawn().context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        let (stdin_sender, mut stdin_receiver) = mpsc::unbounded_channel::<String>();
        let (stdout_sender, stdout_receiver) = mpsc::unbounded_channel::<String>();
        let (stderr_sender, stderr_receiver) = mpsc::unbounded_channel::<String>();

        // Handle stdin
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(line) = stdin_receiver.recv().await {
                if let Err(e) = stdin.write_all(line.as_bytes()).await {
                    eprintln!("Failed to write to stdin: {e}");
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    eprintln!("Failed to flush stdin: {e}");
                    break;
                }
            }
        });

        // Handle stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                let _ = stdout_sender.send(line.trim().to_string());
                line.clear();
            }
        });

        // Handle stderr
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                let _ = stderr_sender.send(line.trim().to_string());
                line.clear();
            }
        });

        Ok(Self {
            process: Some(child),
            request_id: AtomicU64::new(1),
            stdin_sender,
            stdout_receiver,
            stderr_receiver,
        })
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> Result<McpResponse> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: if params.is_null() { None } else { Some(params) },
        };

        let request_json = serde_json::to_string(&request)?;
        self.stdin_sender.send(format!("{request_json}\n"))?;

        // Wait for response with timeout
        let response = timeout(Duration::from_secs(10), async {
            while let Some(line) = self.stdout_receiver.recv().await {
                if let Ok(response) = serde_json::from_str::<McpResponse>(&line) {
                    if response.id == id {
                        return Ok(response);
                    }
                }
            }
            Err(anyhow::anyhow!("No response received"))
        })
        .await??;

        Ok(response)
    }

    pub async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = McpNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: if params.is_null() { None } else { Some(params) },
        };

        let notification_json = serde_json::to_string(&notification)?;
        self.stdin_sender.send(format!("{notification_json}\n"))?;
        Ok(())
    }

    pub async fn initialize(&mut self) -> Result<Value> {
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": true
                },
                "sampling": {}
            },
            "clientInfo": {
                "name": "tools-test",
                "version": "1.0.0"
            }
        });

        let response = self.send_request("initialize", init_params).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Initialize failed: {}", error));
        }

        // Send initialized notification
        self.send_notification("notifications/initialized", json!({}))
            .await?;

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in initialize response"))
    }

    pub async fn list_tools(&mut self) -> Result<Value> {
        let response = self.send_request("tools/list", json!({})).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("List tools failed: {}", error));
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in tools/list response"))
    }

    pub async fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Result<Value> {
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", params).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Tool call failed: {}", error));
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in tools/call response"))
    }

    pub async fn ping(&mut self) -> Result<()> {
        let response = self.send_request("ping", json!({})).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Ping failed: {}", error));
        }

        Ok(())
    }

    pub async fn get_stderr_output(&mut self) -> Vec<String> {
        let mut stderr_lines = Vec::new();
        while let Ok(line) = self.stderr_receiver.try_recv() {
            stderr_lines.push(line);
        }
        stderr_lines
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        // Send shutdown notification
        self.send_notification("notifications/cancelled", json!({}))
            .await?;

        // Wait a bit for graceful shutdown
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Kill the process if it's still running
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }

        Ok(())
    }
}

impl Drop for McpTestClient {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}
