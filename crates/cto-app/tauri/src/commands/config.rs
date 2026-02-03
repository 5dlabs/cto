// Settings CRUD commands
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub theme: String,
    pub auto_start: bool,
    pub cluster_defaults: ClusterDefaults,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterDefaults {
    pub k8s_version: String,
    pub node_count: usize,
}

lazy_static::lazy_static! {
    static ref SETTINGS: parking_lot::RwLock<HashMap<String, String>> = parking_lot::RwLock::new(HashMap::new());
}

#[tauri::command]
pub fn get_setting(key: &str) -> Result<Option<String>, String> {
    let settings = SETTINGS.read();
    Ok(settings.get(key).cloned())
}

#[tauri::command]
pub fn set_setting(key: &str, value: &str) -> Result<(), String> {
    let mut settings = SETTINGS.write();
    settings.insert(key.to_string(), value.to_string());
    Ok(())
}

#[tauri::command]
pub fn list_settings() -> Result<HashMap<String, String>, String> {
    let settings = SETTINGS.read();
    Ok(settings.clone())
}
