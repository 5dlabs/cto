// Tauri entry point - registers all IPC commands
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod keychain;
mod runtime;

use commands::cluster::*;
use commands::{auto_provision_runtime, check_docker_running, get_docker_socket};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Runtime detection commands
            check_docker_running,
            get_docker_socket,
            auto_provision_runtime,
            // Cluster management commands
            list_clusters,
            get_cluster_status,
            start_kind_cluster,
            delete_kind_cluster,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
