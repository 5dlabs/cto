//! CTO Lite - Desktop application for AI-assisted development
//!
//! This is the Tauri backend that manages:
//! - Setup wizard state
//! - Docker/Kind cluster management
//! - Helm chart deployment
//! - API key storage (via system keychain)
//! - Workflow triggering and monitoring
//! - MCP server management for IDE integration

mod commands;
mod docker;
mod helm;
mod keychain;
mod kind;
mod state;
mod workflows;

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub use commands::*;
pub use state::AppState;

/// Initialize and run the Tauri application
#[allow(clippy::disallowed_macros)] // tauri::generate_context! internally uses eprintln
pub fn run() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CTO Lite v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Initialize app state
            let state = AppState::new();
            app.manage(state);

            #[cfg(debug_assertions)]
            {
                // Open devtools in debug mode
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // MCP Server commands
            commands::spawn_mcp_server,
            commands::mcp_call,
            commands::get_mcp_tools,
            commands::call_mcp_tool,
            commands::kill_mcp_server,
            // MCP Task commands (convenience wrappers)
            commands::get_tasks,
            commands::create_task,
            commands::update_task_status,
            // Setup commands
            commands::check_docker,
            commands::check_kind,
            commands::get_setup_state,
            commands::save_setup_state,
            commands::complete_setup,
            // Keychain commands
            commands::store_api_key,
            commands::get_api_key,
            commands::delete_api_key,
            commands::has_api_key,
            // Cluster commands
            commands::create_cluster,
            commands::delete_cluster,
            commands::get_cluster_status,
            commands::list_clusters,
            // Workflow commands
            commands::trigger_workflow,
            commands::get_workflow_status,
            commands::list_workflows,
            commands::get_workflow_logs,
            commands::delete_workflow,
            commands::stop_workflow,
            commands::check_argo,
            // Helm commands
            commands::check_helm,
            commands::deploy_chart,
            commands::get_release_status,
            commands::uninstall_chart,
            commands::update_helm_dependencies,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
