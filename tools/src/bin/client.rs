#![allow(clippy::doc_markdown)]
#![allow(clippy::disallowed_macros)]

use anyhow::Result;
use clap::Parser;
use tools::client::McpClient;

/// Tools MCP Client
///
/// A client-side MCP implementation that provides intelligent routing between local and remote MCP servers,
/// enabling dynamic server management and tool switching for AI development workflows.
#[derive(Parser)]
#[command(name = "tools-client")]
#[command(about = "Tools MCP Client - client-side MCP implementation with local/remote routing")]
#[command(version)]
struct Args {
    /// HTTP server URL to connect to for remote tools
    ///
    /// URL of the Tools HTTP server to connect to for remote tools.
    /// Can also be set via TOOLS_SERVER_URL environment variable.
    #[arg(long)]
    url: Option<String>,

    /// Working directory for local servers and configuration
    ///
    /// The working directory to use for local server spawning and config lookup.
    /// If not provided, uses the current working directory.
    #[arg(long)]
    working_dir: Option<String>,

    /// HTTP server URL (positional argument for compatibility)
    #[arg(value_name = "HTTP_URL", help = "HTTP server URL for remote tools")]
    http_url: Option<String>,

    /// Working directory (positional argument for compatibility)
    #[arg(
        value_name = "WORKING_DIR",
        help = "Working directory for local servers"
    )]
    pos_working_dir: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Log environment variables for debugging workspace detection
    tracing::debug!("[Client] === Environment Variable Detection ===");
    tracing::debug!(
        "[Client] WORKSPACE_FOLDER: {:?}",
        std::env::var("WORKSPACE_FOLDER").ok()
    );
    tracing::debug!(
        "[Client] VSCODE_CWD: {:?}",
        std::env::var("VSCODE_CWD").ok()
    );
    tracing::debug!(
        "[Client] PROJECT_ROOT: {:?}",
        std::env::var("PROJECT_ROOT").ok()
    );
    tracing::debug!("[Client] PWD: {:?}", std::env::var("PWD").ok());
    tracing::debug!(
        "[Client] Current directory: {:?}",
        std::env::current_dir().ok()
    );

    // Show all environment variables that might be IDE-related
    tracing::debug!("[Client] === IDE-Related Environment Variables ===");
    for (key, value) in std::env::vars() {
        if key.contains("WORKSPACE")
            || key.contains("PROJECT")
            || key.contains("VSCODE")
            || key.contains("CURSOR")
            || key.contains("IDE")
            || key.contains("JETBRAINS")
        {
            tracing::debug!("[Client] {key}: {value}");
        }
    }
    tracing::debug!("[Client] ============================================");

    // Determine HTTP base URL with priority: positional arg > flag > env var > default
    let http_base_url = args
        .http_url
        .or(args.url)
        .or_else(|| std::env::var("TOOLS_SERVER_URL").ok())
        .unwrap_or_else(|| "http://tools.cto.svc.cluster.local:3000/mcp".to_string());

    // Auto-detect working directory from various sources
    // Priority: CLI args > IDE workspace env vars > PWD
    let working_dir = args
        .pos_working_dir
        .or(args.working_dir)
        .or_else(|| {
            let wd = std::env::var("WORKSPACE_FOLDER_PATHS").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using WORKSPACE_FOLDER_PATHS for working directory");
            }
            wd
        })
        .or_else(|| {
            let wd = std::env::var("WORKSPACE_FOLDER").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using WORKSPACE_FOLDER for working directory");
            }
            wd
        })
        .or_else(|| {
            let wd = std::env::var("TASK_MASTER_PROJECT_ROOT").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using TASK_MASTER_PROJECT_ROOT for working directory");
            }
            wd
        })
        .or_else(|| {
            let wd = std::env::var("VSCODE_CWD").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using VSCODE_CWD for working directory");
            }
            wd
        })
        .or_else(|| {
            let wd = std::env::var("PROJECT_ROOT").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using PROJECT_ROOT for working directory");
            }
            wd
        })
        .or_else(|| {
            let wd = std::env::var("PWD").ok();
            if wd.is_some() {
                tracing::debug!("[Client] Using PWD for working directory");
            }
            wd
        });

    tracing::debug!("[Client] Final working_dir: {working_dir:?}");

    let client = McpClient::new(http_base_url, working_dir)?;
    client.run()?;

    Ok(())
}
