//! Cloudflare OAuth and tunnel management commands

use crate::error::AppError;
use crate::keychain::{self, CredentialKey};
use serde::{Deserialize, Serialize};

// Cloudflare OAuth App credentials (CTO Lite app)
// Register at: https://dash.cloudflare.com/profile/api-tokens
const CLOUDFLARE_CLIENT_ID: &str = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"; // TODO: Replace with real client ID
const CLOUDFLARE_REDIRECT_URI: &str = "http://localhost:19284/callback/cloudflare";

/// Cloudflare connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareStatus {
    pub connected: bool,
    pub email: Option<String>,
    pub account_name: Option<String>,
}

/// Cloudflare user info from API
#[derive(Debug, Deserialize)]
struct CloudflareUserResult {
    result: CloudflareUser,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct CloudflareUser {
    email: String,
}

/// Cloudflare accounts from API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CloudflareAccountsResult {
    result: Vec<CloudflareAccount>,
    success: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CloudflareAccount {
    id: String,
    name: String,
}

/// Start Cloudflare OAuth flow
/// Returns the authorization URL to open in browser
#[tauri::command]
pub async fn start_cloudflare_oauth() -> Result<String, AppError> {
    // Generate state parameter for CSRF protection
    let state = uuid::Uuid::new_v4().to_string();

    // Cloudflare OAuth scopes for tunnel management
    // See: https://developers.cloudflare.com/fundamentals/api/reference/permissions/
    let scopes = "account:read zone:read tunnel:edit";

    // Build authorization URL
    let auth_url = format!(
        "https://dash.cloudflare.com/oauth2/authorize?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope={}&\
        state={}",
        CLOUDFLARE_CLIENT_ID,
        urlencoding::encode(CLOUDFLARE_REDIRECT_URI),
        urlencoding::encode(scopes),
        state
    );

    // TODO: Start local HTTP server to handle callback
    // For now, return the URL and handle callback separately

    tracing::info!("Starting Cloudflare OAuth flow");
    Ok(auth_url)
}

/// Get Cloudflare connection status
#[tauri::command]
pub async fn get_cloudflare_status() -> Result<CloudflareStatus, AppError> {
    // Check if we have a token
    let has_token = keychain::has_credential(CredentialKey::CloudflareAccessToken)?;

    if !has_token {
        return Ok(CloudflareStatus {
            connected: false,
            email: None,
            account_name: None,
        });
    }

    // Try to get user info
    let token = keychain::get_credential(CredentialKey::CloudflareAccessToken)?
        .ok_or_else(|| AppError::NotConfigured("Cloudflare token not found".to_string()))?;

    let client = reqwest::Client::new();

    // Get user info
    let user_response = client
        .get("https://api.cloudflare.com/client/v4/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if !user_response.status().is_success() {
        // Token might be expired
        return Ok(CloudflareStatus {
            connected: false,
            email: None,
            account_name: None,
        });
    }

    let user_result: CloudflareUserResult = user_response.json().await?;

    if !user_result.success {
        return Ok(CloudflareStatus {
            connected: false,
            email: None,
            account_name: None,
        });
    }

    // Get account info
    let accounts_response = client
        .get("https://api.cloudflare.com/client/v4/accounts")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let account_name = if accounts_response.status().is_success() {
        let accounts_result: CloudflareAccountsResult = accounts_response.json().await?;
        accounts_result.result.first().map(|a| a.name.clone())
    } else {
        None
    };

    Ok(CloudflareStatus {
        connected: true,
        email: Some(user_result.result.email),
        account_name,
    })
}

/// Disconnect Cloudflare
#[tauri::command]
pub async fn disconnect_cloudflare() -> Result<(), AppError> {
    keychain::delete_credential(CredentialKey::CloudflareAccessToken)?;
    keychain::delete_credential(CredentialKey::CloudflareRefreshToken)?;
    keychain::delete_credential(CredentialKey::CloudflareTunnelToken)?;

    tracing::info!("Disconnected from Cloudflare");
    Ok(())
}
