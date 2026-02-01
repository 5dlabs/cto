//! Cloudflare tunnel management commands

use crate::db::Database;
use crate::error::AppError;
use crate::keychain::{self, CredentialKey};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::State;

// Global tunnel process handle
static TUNNEL_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

/// Tunnel status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub exists: bool,
    pub running: bool,
    pub tunnel_id: Option<String>,
    pub url: Option<String>,
}

/// Cloudflare tunnel info from API
#[derive(Debug, Deserialize)]
struct CloudflareTunnelsResult {
    result: Vec<CloudflareTunnel>,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct CloudflareTunnel {
    id: String,
    name: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareTunnelTokenResult {
    result: String,
    success: bool,
}

/// Create a Cloudflare tunnel
#[tauri::command]
pub async fn create_tunnel(db: State<'_, Database>) -> Result<TunnelStatus, AppError> {
    let token = keychain::get_credential(CredentialKey::CloudflareAccessToken)?
        .ok_or_else(|| AppError::NotConfigured("Cloudflare not connected".to_string()))?;

    let client = reqwest::Client::new();

    // Get account ID first
    let accounts_response = client
        .get("https://api.cloudflare.com/client/v4/accounts")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    #[derive(Deserialize)]
    struct AccountsResult {
        result: Vec<Account>,
    }
    #[derive(Deserialize)]
    struct Account {
        id: String,
    }

    let accounts: AccountsResult = accounts_response.json().await?;
    let account_id = accounts
        .result
        .first()
        .ok_or_else(|| AppError::TunnelError("No Cloudflare account found".to_string()))?
        .id
        .clone();

    // Check if tunnel already exists
    let tunnel_name = "cto-lite";
    let tunnels_response = client
        .get(&format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel",
            account_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("name", tunnel_name)])
        .send()
        .await?;

    let tunnels: CloudflareTunnelsResult = tunnels_response.json().await?;

    let tunnel_id = if let Some(existing) = tunnels.result.first() {
        tracing::info!("Using existing tunnel: {}", existing.id);
        existing.id.clone()
    } else {
        // Create new tunnel
        tracing::info!("Creating new tunnel: {}", tunnel_name);

        let create_response = client
            .post(&format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel",
                account_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "name": tunnel_name,
                "config_src": "local"
            }))
            .send()
            .await?;

        #[derive(Deserialize)]
        struct CreateTunnelResult {
            result: CloudflareTunnel,
            success: bool,
        }

        let create_result: CreateTunnelResult = create_response.json().await?;
        if !create_result.success {
            return Err(AppError::TunnelError("Failed to create tunnel".to_string()));
        }

        create_result.result.id
    };

    // Get tunnel token
    let token_response = client
        .get(&format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/cfd_tunnel/{}/token",
            account_id, tunnel_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let token_result: CloudflareTunnelTokenResult = token_response.json().await?;
    if !token_result.success {
        return Err(AppError::TunnelError(
            "Failed to get tunnel token".to_string(),
        ));
    }

    // Store tunnel token in keychain
    keychain::set_credential(CredentialKey::CloudflareTunnelToken, &token_result.result)?;

    // Store tunnel info in database
    let tunnel_url = format!("https://{}.cfargotunnel.com", tunnel_id);
    db.set_config("tunnel_id", &tunnel_id)?;
    db.set_config("tunnel_url", &tunnel_url)?;

    tracing::info!("Tunnel created: {} -> {}", tunnel_id, tunnel_url);

    Ok(TunnelStatus {
        exists: true,
        running: false,
        tunnel_id: Some(tunnel_id),
        url: Some(tunnel_url),
    })
}

/// Start the tunnel (run cloudflared)
#[tauri::command]
pub async fn start_tunnel(db: State<'_, Database>) -> Result<TunnelStatus, AppError> {
    // Check if already running
    {
        let process = TUNNEL_PROCESS
            .lock()
            .map_err(|e| AppError::TunnelError(e.to_string()))?;
        if process.is_some() {
            // Check if process is still alive
            // For now, assume it's running
            let tunnel_id = db.get_config("tunnel_id")?;
            let tunnel_url = db.get_config("tunnel_url")?;
            return Ok(TunnelStatus {
                exists: tunnel_id.is_some(),
                running: true,
                tunnel_id,
                url: tunnel_url,
            });
        }
    }

    let tunnel_token = keychain::get_credential(CredentialKey::CloudflareTunnelToken)?
        .ok_or_else(|| AppError::NotConfigured("Tunnel not created yet".to_string()))?;

    // Find cloudflared binary
    let cloudflared_path = which::which("cloudflared")
        .map_err(|_| AppError::CommandFailed("cloudflared not found".to_string()))?;

    // Start cloudflared tunnel
    let child = Command::new(cloudflared_path)
        .args(["tunnel", "--no-autoupdate", "run", "--token", &tunnel_token])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| AppError::CommandFailed(format!("Failed to start cloudflared: {}", e)))?;

    // Store process handle
    {
        let mut process = TUNNEL_PROCESS
            .lock()
            .map_err(|e| AppError::TunnelError(e.to_string()))?;
        *process = Some(child);
    }

    let tunnel_id = db.get_config("tunnel_id")?;
    let tunnel_url = db.get_config("tunnel_url")?;

    tracing::info!("Tunnel started");

    Ok(TunnelStatus {
        exists: true,
        running: true,
        tunnel_id,
        url: tunnel_url,
    })
}

/// Stop the tunnel
#[tauri::command]
pub async fn stop_tunnel(db: State<'_, Database>) -> Result<TunnelStatus, AppError> {
    {
        let mut process = TUNNEL_PROCESS
            .lock()
            .map_err(|e| AppError::TunnelError(e.to_string()))?;

        if let Some(ref mut child) = *process {
            let _ = child.kill();
            let _ = child.wait();
        }

        *process = None;
    }

    let tunnel_id = db.get_config("tunnel_id")?;
    let tunnel_url = db.get_config("tunnel_url")?;

    tracing::info!("Tunnel stopped");

    Ok(TunnelStatus {
        exists: tunnel_id.is_some(),
        running: false,
        tunnel_id,
        url: tunnel_url,
    })
}

/// Get tunnel status
#[tauri::command]
pub async fn get_tunnel_status(db: State<'_, Database>) -> Result<TunnelStatus, AppError> {
    let tunnel_id = db.get_config("tunnel_id")?;
    let tunnel_url = db.get_config("tunnel_url")?;

    let running = {
        let process = TUNNEL_PROCESS
            .lock()
            .map_err(|e| AppError::TunnelError(e.to_string()))?;
        process.is_some()
    };

    Ok(TunnelStatus {
        exists: tunnel_id.is_some(),
        running,
        tunnel_id,
        url: tunnel_url,
    })
}
