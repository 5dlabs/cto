//! System keychain integration for secure API key storage

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "dev.cto.lite";

/// Supported API key types
#[derive(Debug, Clone, Copy)]
pub enum ApiKeyType {
    Anthropic,
    OpenAI,
    GitHub,
}

impl ApiKeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKeyType::Anthropic => "anthropic",
            ApiKeyType::OpenAI => "openai",
            ApiKeyType::GitHub => "github",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "anthropic" => Some(ApiKeyType::Anthropic),
            "openai" => Some(ApiKeyType::OpenAI),
            "github" => Some(ApiKeyType::GitHub),
            _ => None,
        }
    }
}

/// Store an API key in the system keychain
pub fn store_key(key_type: ApiKeyType, value: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, key_type.as_str()).context("Failed to create keychain entry")?;

    entry
        .set_password(value)
        .context("Failed to store API key in keychain")?;

    tracing::info!("Stored {} API key in keychain", key_type.as_str());
    Ok(())
}

/// Retrieve an API key from the system keychain
pub fn get_key(key_type: ApiKeyType) -> Result<Option<String>> {
    let entry =
        Entry::new(SERVICE_NAME, key_type.as_str()).context("Failed to create keychain entry")?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(anyhow::anyhow!("Failed to retrieve API key: {}", e)),
    }
}

/// Delete an API key from the system keychain
pub fn delete_key(key_type: ApiKeyType) -> Result<()> {
    let entry =
        Entry::new(SERVICE_NAME, key_type.as_str()).context("Failed to create keychain entry")?;

    match entry.delete_credential() {
        Ok(()) => {
            tracing::info!("Deleted {} API key from keychain", key_type.as_str());
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(anyhow::anyhow!("Failed to delete API key: {}", e)),
    }
}

/// Check if an API key exists in the keychain
pub fn has_key(key_type: ApiKeyType) -> Result<bool> {
    Ok(get_key(key_type)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a real keychain and may prompt for access
    // Run manually with: cargo test --package cto-lite -- keychain --ignored

    #[test]
    #[ignore]
    fn test_keychain_roundtrip() {
        let test_value = "test-key-12345";

        // Store
        store_key(ApiKeyType::Anthropic, test_value).unwrap();

        // Retrieve
        let retrieved = get_key(ApiKeyType::Anthropic).unwrap();
        assert_eq!(retrieved, Some(test_value.to_string()));

        // Check exists
        assert!(has_key(ApiKeyType::Anthropic).unwrap());

        // Delete
        delete_key(ApiKeyType::Anthropic).unwrap();

        // Verify deleted
        assert!(!has_key(ApiKeyType::Anthropic).unwrap());
    }
}
