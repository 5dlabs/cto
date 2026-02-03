// GitHub OAuth and webhook commands
use serde::Serialize;

#[tauri::command]
pub fn set_github_token(token: &str) -> Result<(), String> {
    // This would use the keychain module internally
    // For now, return success
    Ok(())
}

#[tauri::command]
pub fn get_github_token() -> Result<Option<String>, String> {
    // This would use the keychain module internally
    Ok(None)
}

#[tauri::command]
pub fn create_webhook(repo: &str, webhook_url: &str, events: &[&str]) -> Result<String, String> {
    // Placeholder for webhook creation
    // Would use GitHub API with the stored token
    Ok(format!(
        "Webhook would be created for {} with URL {} for events {:?}",
        repo, webhook_url, events
    ))
}

#[derive(Debug, Serialize)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
}
