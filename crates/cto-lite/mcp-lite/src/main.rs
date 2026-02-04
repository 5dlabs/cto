//! MCP Lite - Model Context Protocol server for CTO Lite
//!
//! This binary runs the MCP server as a standalone process.

use mcp_lite::run_server;

fn main() {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or("RUST_LOG", "info"),
    );

    // Run the server
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    rt.block_on(run_server());
}
