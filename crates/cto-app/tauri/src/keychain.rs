// OS Keychain integration for secrets
use keyring::Entry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubToken {
    pub access_token: String,
    pub expires_at: Option<u64>,
}

#[tauri::command]
pub fn get_github_token() -> Result<Option<GitHubToken>, String> {
    let entry = Entry::new("cto-app", "github_token")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(GitHubToken {
            access_token: token,
            expires_at: None,
        })),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to get GitHub token: {}", e)),
    }
}

#[tauri::command]
pub fn set_github_token(token: &str) -> Result<(), String> {
    let entry = Entry::new("cto-app", "github_token")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    entry
        .set_password(token)
        .map_err(|e| format!("Failed to save GitHub token: {}", e))
}

#[tauri::command]
pub fn delete_github_token() -> Result<(), String> {
    let entry = Entry::new("cto-app", "github_token")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    entry
        .delete_password()
        .map_err(|e| format!("Failed to delete GitHub token: {}", e))
}

#[tauri::command]
pub fn get_cf_tunnel_token() -> Result<Option<String>, String> {
    let entry = Entry::new("cto-app", "cf_tunnel_token")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to get Cloudflare tunnel token: {}", e)),
    }
}

#[tauri::command]
pub fn set_cf_tunnel_token(token: &str) -> Result<(), String> {
    let entry = Entry::new("cto-app", "cf_tunnel_token")
        .map_err(|e| format!("Failed to access keychain: {}", e))?;

    entry
        .set_password(token)
        .map_err(|e| format!("Failed to save Cloudflare tunnel token: {}", e))
}
