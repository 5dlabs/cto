//! CTO Lite - Desktop Application
//!
//! A freemium desktop application that runs the CTO AI development platform
//! on a local Kind cluster. Built with Tauri for cross-platform native experience.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod db;
mod error;
mod keychain;
mod runtime;

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "cto_lite=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CTO Lite v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database on startup
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            
            let db_path = app_data_dir.join("cto-lite.db");
            let db = db::Database::new(&db_path)?;
            db.migrate()?;
            
            app.manage(db);
            
            tracing::info!("Database initialized at {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Runtime detection
            commands::runtime::detect_container_runtime,
            commands::runtime::get_runtime_status,
            commands::runtime::check_docker_running,
            
            // Cluster management
            commands::cluster::create_cluster,
            commands::cluster::delete_cluster,
            commands::cluster::get_cluster_status,
            commands::cluster::list_clusters,
            
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
