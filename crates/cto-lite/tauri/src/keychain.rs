//! Secure credential storage using OS keychain
//!
//! - macOS: Keychain Access
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring / KWallet)

use crate::error::{AppError, AppResult};
use keyring::Entry;

const SERVICE_NAME: &str = "cto";
const LEGACY_SERVICE_NAME: &str = "cto-lite";

/// Credential types we store in the keychain
#[derive(Debug, Clone, Copy)]
pub enum CredentialKey {
    AnthropicApiKey,
    OpenAiApiKey,
    GithubAccessToken,
    GithubRefreshToken,
    CloudflareAccessToken,
    CloudflareRefreshToken,
    CloudflareTunnelToken,
}

impl CredentialKey {
    fn as_str(&self) -> &'static str {
        match self {
            Self::AnthropicApiKey => "anthropic_api_key",
            Self::OpenAiApiKey => "openai_api_key",
            Self::GithubAccessToken => "github_access_token",
            Self::GithubRefreshToken => "github_refresh_token",
            Self::CloudflareAccessToken => "cloudflare_access_token",
            Self::CloudflareRefreshToken => "cloudflare_refresh_token",
            Self::CloudflareTunnelToken => "cloudflare_tunnel_token",
        }
    }
}

impl std::str::FromStr for CredentialKey {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "anthropic_api_key" | "anthropic" => Ok(Self::AnthropicApiKey),
            "openai_api_key" | "openai" => Ok(Self::OpenAiApiKey),
            "github_access_token" | "github" => Ok(Self::GithubAccessToken),
            "github_refresh_token" => Ok(Self::GithubRefreshToken),
            "cloudflare_access_token" | "cloudflare" => Ok(Self::CloudflareAccessToken),
            "cloudflare_refresh_token" => Ok(Self::CloudflareRefreshToken),
            "cloudflare_tunnel_token" => Ok(Self::CloudflareTunnelToken),
            _ => Err(AppError::KeychainError(format!(
                "Unknown credential key: {}",
                s
            ))),
        }
    }
}

/// Store a credential in the keychain
pub fn set_credential(key: CredentialKey, value: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE_NAME, key.as_str())
        .map_err(|e| AppError::KeychainError(e.to_string()))?;

    entry
        .set_password(value)
        .map_err(|e| AppError::KeychainError(e.to_string()))?;

    tracing::debug!("Stored credential: {}", key.as_str());
    Ok(())
}

/// Retrieve a credential from the keychain
pub fn get_credential(key: CredentialKey) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, key.as_str())
        .map_err(|e| AppError::KeychainError(e.to_string()))?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => {
            // Backward compatibility: read from the previous service name and migrate forward.
            let legacy = Entry::new(LEGACY_SERVICE_NAME, key.as_str())
                .map_err(|e| AppError::KeychainError(e.to_string()))?;
            match legacy.get_password() {
                Ok(password) => {
                    if let Err(err) = set_credential(key, &password) {
                        tracing::warn!("Failed to migrate legacy keychain item: {}", err);
                    }
                    Ok(Some(password))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(AppError::KeychainError(e.to_string())),
            }
        }
        Err(e) => Err(AppError::KeychainError(e.to_string())),
    }
}

/// Delete a credential from the keychain
pub fn delete_credential(key: CredentialKey) -> AppResult<()> {
    let entry = Entry::new(SERVICE_NAME, key.as_str())
        .map_err(|e| AppError::KeychainError(e.to_string()))?;
    let legacy = Entry::new(LEGACY_SERVICE_NAME, key.as_str())
        .map_err(|e| AppError::KeychainError(e.to_string()))?;

    match entry.delete_credential() {
        Ok(()) => {
            tracing::debug!("Deleted credential: {}", key.as_str());
        }
        Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(AppError::KeychainError(e.to_string())),
    }

    match legacy.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::KeychainError(e.to_string())),
    }
}

/// Check if a credential exists in the keychain
pub fn has_credential(key: CredentialKey) -> AppResult<bool> {
    get_credential(key).map(|v| v.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests interact with the real keychain
    // Run with: cargo test -- --ignored

    #[test]
    #[ignore]
    fn test_credential_roundtrip() {
        let key = CredentialKey::AnthropicApiKey;
        let value = "test-api-key-12345";

        // Set
        set_credential(key, value).unwrap();

        // Get
        let retrieved = get_credential(key).unwrap();
        assert_eq!(retrieved, Some(value.to_string()));

        // Delete
        delete_credential(key).unwrap();

        // Verify deleted
        let after_delete = get_credential(key).unwrap();
        assert_eq!(after_delete, None);
    }
}
