//! Configuration commands

use crate::db::Database;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Selected backend stack: "go" (Grizz) or "node" (Nova)
    pub backend_stack: Option<String>,
    /// Selected CLI: "claude", "factory", or "codex"
    pub cli: Option<String>,
    /// Selected AI model
    pub model: Option<String>,
    /// Selected AI provider: "anthropic" or "openai"
    pub ai_provider: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_stack: Some("go".to_string()),
            cli: Some("claude".to_string()),
            model: Some("claude-sonnet-4-20250514".to_string()),
            ai_provider: Some("anthropic".to_string()),
        }
    }
}

/// Setup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub current_step: i32,
    pub completed: bool,
    pub steps: Vec<SetupStepStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStepStatus {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub completed: bool,
}

const SETUP_STEPS: &[(&str, &str)] = &[
    ("runtime", "Container Runtime"),
    ("stack", "Choose Your Stack"),
    ("api_keys", "API Keys"),
    ("github", "GitHub Connection"),
    ("cloudflare", "Cloudflare Connection"),
    ("cluster", "Create Cluster"),
];

/// Get application configuration
#[tauri::command]
pub async fn get_config(db: State<'_, Database>) -> Result<AppConfig, AppError> {
    let backend_stack = db.get_config("backend_stack")?;
    let cli = db.get_config("cli")?;
    let model = db.get_config("model")?;
    let ai_provider = db.get_config("ai_provider")?;

    Ok(AppConfig {
        backend_stack,
        cli,
        model,
        ai_provider,
    })
}

/// Set a configuration value
#[tauri::command]
pub async fn set_config(
    db: State<'_, Database>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    // Validate key
    let valid_keys = ["backend_stack", "cli", "model", "ai_provider"];
    if !valid_keys.contains(&key.as_str()) {
        return Err(AppError::ConfigError(format!(
            "Invalid config key: {}",
            key
        )));
    }

    // Validate values
    match key.as_str() {
        "backend_stack" => {
            if !["go", "node"].contains(&value.as_str()) {
                return Err(AppError::ConfigError(
                    "backend_stack must be 'go' or 'node'".to_string(),
                ));
            }
        }
        "cli" => {
            if !["claude", "factory", "codex"].contains(&value.as_str()) {
                return Err(AppError::ConfigError(
                    "cli must be 'claude', 'factory', or 'codex'".to_string(),
                ));
            }
        }
        "ai_provider" => {
            if !["anthropic", "openai"].contains(&value.as_str()) {
                return Err(AppError::ConfigError(
                    "ai_provider must be 'anthropic' or 'openai'".to_string(),
                ));
            }
        }
        _ => {}
    }

    db.set_config(&key, &value)?;
    tracing::debug!("Set config {}={}", key, value);
    Ok(())
}

/// Get setup wizard status
#[tauri::command]
pub async fn get_setup_status(db: State<'_, Database>) -> Result<SetupStatus, AppError> {
    let (current_step, completed) = db.get_setup_progress()?;

    let steps = SETUP_STEPS
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| SetupStepStatus {
            id: i as i32,
            name: name.to_string(),
            description: desc.to_string(),
            completed: (i as i32) < current_step,
        })
        .collect();

    Ok(SetupStatus {
        current_step,
        completed,
        steps,
    })
}

/// Mark setup as complete
#[tauri::command]
pub async fn mark_setup_complete(db: State<'_, Database>) -> Result<(), AppError> {
    db.mark_setup_complete()?;
    tracing::info!("Setup marked as complete");
    Ok(())
}
