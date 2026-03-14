//! CTO - Desktop Application
//!
//! A freemium desktop application that runs the CTO AI development platform
//! on a local Kind cluster. Built with Tauri for cross-platform native experience.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
// Allow tauri::generate_context! which internally uses eprintln
#![allow(clippy::disallowed_macros)]

mod commands;
mod db;
mod error;
mod keychain;
mod paths;
mod runtime;

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn get_legacy_data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        return dirs::data_dir().map(|p| p.join("ai.5dlabs.cto-lite"));
    }
    #[cfg(target_os = "windows")]
    {
        return dirs::data_dir().map(|p| p.join("ai.5dlabs").join("cto-lite"));
    }
    #[cfg(target_os = "linux")]
    {
        return dirs::data_dir().map(|p| p.join("ai.5dlabs.cto-lite"));
    }

    #[allow(unreachable_code)]
    None
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if !dst_path.exists() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cto_tauri=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CTO v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            // One-time migration for previous app identifier data directory.
            if let Some(legacy_data_dir) = get_legacy_data_dir() {
                if legacy_data_dir.exists() && !app_data_dir.exists() {
                    tracing::info!(
                        "Migrating app data directory from {:?} to {:?}",
                        legacy_data_dir,
                        app_data_dir
                    );
                    copy_dir_all(&legacy_data_dir, &app_data_dir)?;
                }
            }

            // Initialize database on startup
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("cto-lite.db");
            let db = db::Database::new(&db_path)?;
            db.migrate()?;

            app.manage(db);

            // Initialize MCP state
            app.manage(commands::mcp::McpState::new());
            app.manage(commands::openclaw::LocalBridgeState::new());
            app.manage(commands::openclaw::ConversationState::new());

            tracing::info!("Database initialized at {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Runtime detection
            commands::runtime::detect_container_runtime,
            commands::runtime::get_runtime_status,
            commands::runtime::check_docker_running,
            commands::runtime::scan_runtime_environment,
            commands::runtime::start_container_runtime,
            // Cluster management
            commands::cluster::scan_environment,
            commands::cluster::detect_existing_clusters,
            commands::cluster::create_cluster,
            commands::cluster::delete_cluster,
            commands::cluster::get_cluster_status,
            commands::cluster::list_clusters,
            commands::cluster::use_existing_cluster,
            // Configuration
            commands::config::get_config,
            commands::config::set_config,
            commands::config::get_setup_status,
            commands::config::mark_setup_complete,
            // Credentials (keychain)
            commands::credentials::set_api_key,
            commands::credentials::get_api_key,
            commands::credentials::delete_api_key,
            commands::credentials::has_api_key,
            // GitHub OAuth
            commands::github::start_github_oauth,
            commands::github::get_github_status,
            commands::github::disconnect_github,
            commands::github::list_repositories,
            // Cloudflare OAuth
            commands::cloudflare::start_cloudflare_oauth,
            commands::cloudflare::get_cloudflare_status,
            commands::cloudflare::disconnect_cloudflare,
            // Tunnel management
            commands::tunnel::create_tunnel,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::get_tunnel_status,
            // Workflow management
            commands::workflow::list_workflows,
            commands::workflow::get_workflow_status,
            commands::workflow::get_workflow_logs,
            commands::workflow::cancel_workflow,
            // MCP server management
            commands::mcp::start_mcp_server,
            commands::mcp::stop_mcp_server,
            commands::mcp::get_mcp_status,
            commands::mcp::get_mcp_config,
            // Log streaming
            commands::logs::list_namespaces,
            commands::logs::list_pods,
            commands::logs::list_pods_with_status,
            commands::logs::stream_pod_logs,
            commands::logs::start_log_stream,
            commands::logs::stop_log_stream,
            // Updates
            commands::updates::check_updates,
            commands::updates::pull_updates,
            commands::updates::apply_updates,
            commands::updates::get_component_versions,
            // Installation
            commands::install::check_prerequisites,
            commands::install::run_installation,
            commands::install::get_install_status,
            commands::install::reset_installation,
            // OpenClaw gateway
            commands::openclaw::openclaw_send_message,
            commands::openclaw::openclaw_send_avatar_context,
            commands::openclaw::openclaw_get_messages,
            commands::openclaw::openclaw_start_workflow,
            commands::openclaw::openclaw_get_workflow_status,
            commands::openclaw::openclaw_approve,
            commands::openclaw::openclaw_reject,
            commands::openclaw::openclaw_get_status,
            commands::openclaw::openclaw_exec_cli,
            commands::openclaw::openclaw_start_local_bridge,
            commands::openclaw::openclaw_stop_local_bridge,
            commands::openclaw::openclaw_get_local_bridge_status,
            commands::openclaw::openclaw_get_morgan_diagnostics,
            // Studio state
            commands::studio::studio_get_state,
            commands::studio::studio_save_state,
            commands::studio::studio_render_agent_config,
            commands::studio::studio_export_agent_config,
            commands::studio::studio_apply_agent_config,
            // Smart initialization & runtime
            commands::cluster::smart_init,
            commands::cluster::quick_health_check,
            commands::runtime::auto_detect_and_start_runtime,
            commands::runtime::auto_start_runtime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
