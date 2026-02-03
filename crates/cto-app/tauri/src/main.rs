// Tauri entry point - registers all IPC commands
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Runtime commands
            cto_app_tauri::get_container_runtime,
            cto_app_tauri::install_kind,
            // Cluster commands
            cto_app_tauri::start_kind_cluster,
            cto_app_tauri::stop_kind_cluster,
            cto_app_tauri::get_cluster_status,
            cto_app_tauri::list_clusters,
            // Settings commands
            cto_app_tauri::get_setting,
            cto_app_tauri::set_setting,
            cto_app_tauri::list_settings,
            // GitHub commands
            cto_app_tauri::get_github_token,
            cto_app_tauri::set_github_token,
            cto_app_tauri::create_webhook,
            // Tunnel commands
            cto_app_tauri::get_cf_tunnel_token,
            cto_app_tauri::set_cf_tunnel_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
