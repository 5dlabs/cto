//! CTO Lite MCP Server - Main entry point for stdio mode
//!
//! Run with: cargo run --features stdio
//!
//! This binary runs the MCP server using stdio for communication,
//! designed to be spawned by Tauri as a child process.

use cto_mcp_lite::desktop_server::DesktopServer;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create and run the server
    let server = DesktopServer::new();

    if let Err(e) = server.run_stdio() {
        eprintln!("MCP server error: {:#}", e);
        std::process::exit(1);
    }
}
