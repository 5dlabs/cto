//! MCP Lite - Model Context Protocol server for CTO Lite
//!
//! This binary runs the MCP server as a standalone process.

use mcp_lite::run_server;

#[tokio::main]
async fn main() {
    // Run the server (logging is initialized in run_server)
    if let Err(e) = run_server().await {
        eprintln!("MCP server error: {}", e);
        std::process::exit(1);
    }
}
