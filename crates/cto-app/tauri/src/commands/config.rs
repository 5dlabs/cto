// Settings CRUD commands
use std::collections::HashMap;

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
