//! Credential management commands using OS keychain

use crate::error::AppError;
use crate::keychain::{self, CredentialKey};

/// Set an API key in the keychain
#[tauri::command]
pub async fn set_api_key(provider: String, api_key: String) -> Result<(), AppError> {
    let key = match provider.as_str() {
        "anthropic" => CredentialKey::AnthropicApiKey,
        "openai" => CredentialKey::OpenAiApiKey,
        _ => {
            return Err(AppError::KeychainError(format!(
                "Unknown provider: {}",
                provider
            )))
        }
    };

    keychain::set_credential(key, &api_key)?;
    tracing::info!("Stored API key for provider: {}", provider);
    Ok(())
}

/// Get an API key from the keychain (returns masked version for display)
#[tauri::command]
pub async fn get_api_key(provider: String) -> Result<Option<String>, AppError> {
    let key = match provider.as_str() {
        "anthropic" => CredentialKey::AnthropicApiKey,
        "openai" => CredentialKey::OpenAiApiKey,
        _ => {
            return Err(AppError::KeychainError(format!(
                "Unknown provider: {}",
                provider
            )))
        }
    };

    // Return masked version for security
    match keychain::get_credential(key)? {
        Some(value) => {
            // Mask the key: show first 7 and last 4 chars
            let masked = if value.len() > 15 {
                format!("{}...{}", &value[..7], &value[value.len() - 4..])
            } else {
                "***".to_string()
            };
            Ok(Some(masked))
        }
        None => Ok(None),
    }
}

/// Delete an API key from the keychain
#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<(), AppError> {
    let key = match provider.as_str() {
        "anthropic" => CredentialKey::AnthropicApiKey,
        "openai" => CredentialKey::OpenAiApiKey,
        _ => {
            return Err(AppError::KeychainError(format!(
                "Unknown provider: {}",
                provider
            )))
        }
    };

    keychain::delete_credential(key)?;
    tracing::info!("Deleted API key for provider: {}", provider);
    Ok(())
}

/// Check if an API key exists
#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<bool, AppError> {
    let key = match provider.as_str() {
        "anthropic" => CredentialKey::AnthropicApiKey,
        "openai" => CredentialKey::OpenAiApiKey,
        _ => {
            return Err(AppError::KeychainError(format!(
                "Unknown provider: {}",
                provider
            )))
        }
    };

    keychain::has_credential(key)
}
