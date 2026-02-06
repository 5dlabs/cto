// Tauri entry point - registers all IPC commands
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod keychain;
mod runtime;

use commands::cluster::*;
use pm_lite::github_app::{install_github_app, list_webhook_events, redeliver_webhook};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Cluster management commands
            list_clusters,
            get_cluster_info,
            start_cluster,
            stop_cluster,
            restart_cluster,
            delete_cluster,
            get_clusters_status,
            // PM-Lite: GitHub App commands
            install_github_app,
            list_webhook_events,
            redeliver_webhook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
