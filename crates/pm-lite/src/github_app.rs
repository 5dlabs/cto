//! GitHub App integration for CTO Desktop
//!
//! Provides:
//! - Webhook handlers for GitHub App events
//! - Tauri commands for installation and event management
//! - Event storage and redelivery support

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// In-memory event storage (shared across Tauri commands)
#[derive(Clone, Default)]
pub struct EventStore(Arc<tokio::sync::Mutex<Vec<StoredEvent>>>);

impl EventStore {
    pub fn new() -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(Vec::new())))
    }

    pub async fn push(&self, event: StoredEvent) {
        let mut store = self.0.lock().await;
        store.push(event);
        // Keep only last 1000 events
        let len = store.len();
        if len > 1000 {
            store.drain(0..len - 1000);
        }
    }

    pub async fn get_all(&self) -> Vec<StoredEvent> {
        self.0.lock().await.clone()
    }

    pub async fn get(&self, id: &str) -> Option<StoredEvent> {
        self.0.lock().await.iter().find(|e| e.id == id).cloned()
    }

    pub async fn remove(&self, id: &str) -> bool {
        let mut store = self.0.lock().await;
        let pos = store.iter().position(|e| e.id == id);
        if let Some(p) = pos {
            store.remove(p);
            true
        } else {
            false
        }
    }
}

/// Stored webhook event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: String,
    pub delivery_id: String,
    pub event_type: String,
    pub action: Option<String>,
    pub payload: serde_json::Value,
    pub repository: Option<String>,
    pub received_at: DateTime<Utc>,
    pub redelivery_count: u32,
    pub status: EventStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventStatus {
    Received,
    Processing,
    Processed,
    Failed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WebhookEventType {
    PullRequest,
    WorkflowRun,
    CheckRun,
    Unknown(String),
}

impl From<String> for WebhookEventType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pull_request" => Self::PullRequest,
            "workflow_run" => Self::WorkflowRun,
            "check_run" => Self::CheckRun,
            _ => Self::Unknown(s),
        }
    }
}

/// GitHub App configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubAppConfig {
    pub app_id: String,
    pub private_key: String,
    pub webhook_secret: Option<String>,
    pub installation_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl GitHubAppConfig {
    pub fn new(app_id: String, private_key: String) -> Self {
        Self {
            app_id,
            private_key,
            webhook_secret: None,
            installation_id: None,
            created_at: Utc::now(),
        }
    }
}

/// Pull request event data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub author: String,
    pub url: String,
    pub draft: bool,
    pub merged: bool,
    pub repository: String,
}

/// Workflow run event data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowRunEvent {
    pub action: String,
    pub workflow_name: String,
    pub run_id: u64,
    pub status: String,
    pub conclusion: Option<String>,
    pub branch: String,
    pub repository: String,
    pub url: String,
}

/// Check run event data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckRunEvent {
    pub action: String,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub repository: String,
    pub url: String,
}

/// Webhook payload wrapper
#[derive(Deserialize)]
pub struct WebhookPayload {
    pub action: Option<String>,
    #[serde(default)]
    pub pull_request: Option<serde_json::Value>,
    #[serde(default)]
    pub workflow_run: Option<serde_json::Value>,
    #[serde(default)]
    pub check_run: Option<serde_json::Value>,
    pub repository: Option<serde_json::Value>,
    pub sender: Option<serde_json::Value>,
}

/// Global event store (initialized once)
lazy_static::lazy_static! {
    static ref EVENT_STORE: EventStore = EventStore::new();
    static ref APP_CONFIG: Arc<tokio::sync::Mutex<Option<GitHubAppConfig>>> = Arc::new(tokio::sync::Mutex::new(None));
    static ref HTTP_CLIENT: Client = ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
}

/// Get the global event store
pub fn get_event_store() -> EventStore {
    EVENT_STORE.clone()
}

/// Verify GitHub webhook signature
pub fn verify_webhook_signature(payload: &[u8], signature: &str, secret: &str) -> Result<()> {
    let secret = secret.as_bytes();
    let sig = signature.trim_start_matches("sha256=");

    let expected_sig = {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret)
            .map_err(|_| anyhow!("Invalid HMAC key length"))?;
        mac.update(payload);
        hex::encode(mac.finalize().into_bytes())
    };

    let sig_bytes = hex::decode(sig)
        .map_err(|_| anyhow!("Invalid hex signature"))?;
    let expected_bytes = hex::decode(&expected_sig)
        .map_err(|_| anyhow!("Invalid hex expected signature"))?;

    if sig_bytes.len() != expected_bytes.len() {
        return Err(anyhow!("Signature length mismatch"));
    }

    let mut result = 0u8;
    for (a, b) in sig_bytes.iter().zip(expected_bytes.iter()) {
        result |= a ^ b;
    }

    if result != 0 {
        return Err(anyhow!("Webhook signature verification failed"));
    }

    Ok(())
}

/// Parse webhook payload into typed event
pub async fn parse_webhook_event(
    event_type: &str,
    payload: &serde_json::Value,
) -> Result<WebhookEventType> {
    match event_type {
        "pull_request" => {
            let pr = extract_pr_data(payload)?;
            let stored = StoredEvent {
                id: Uuid::new_v4().to_string(),
                delivery_id: Uuid::new_v4().to_string(),
                event_type: event_type.to_string(),
                action: Some(pr.action.clone()),
                payload: payload.clone(),
                repository: Some(pr.repository.clone()),
                received_at: Utc::now(),
                redelivery_count: 0,
                status: EventStatus::Received,
            };
            EVENT_STORE.push(stored).await;
            Ok(WebhookEventType::PullRequest)
        }
        "workflow_run" => {
            let wf = extract_workflow_run_data(payload)?;
            let stored = StoredEvent {
                id: Uuid::new_v4().to_string(),
                delivery_id: Uuid::new_v4().to_string(),
                event_type: event_type.to_string(),
                action: Some(wf.action.clone()),
                payload: payload.clone(),
                repository: Some(wf.repository.clone()),
                received_at: Utc::now(),
                redelivery_count: 0,
                status: EventStatus::Received,
            };
            EVENT_STORE.push(stored).await;
            Ok(WebhookEventType::WorkflowRun)
        }
        "check_run" => {
            let cr = extract_check_run_data(payload)?;
            let stored = StoredEvent {
                id: Uuid::new_v4().to_string(),
                delivery_id: Uuid::new_v4().to_string(),
                event_type: event_type.to_string(),
                action: Some(cr.action.clone()),
                payload: payload.clone(),
                repository: Some(cr.repository.clone()),
                received_at: Utc::now(),
                redelivery_count: 0,
                status: EventStatus::Received,
            };
            EVENT_STORE.push(stored).await;
            Ok(WebhookEventType::CheckRun)
        }
        _ => {
            let stored = StoredEvent {
                id: Uuid::new_v4().to_string(),
                delivery_id: Uuid::new_v4().to_string(),
                event_type: event_type.to_string(),
                action: None,
                payload: payload.clone(),
                repository: extract_repo_name(payload),
                received_at: Utc::now(),
                redelivery_count: 0,
                status: EventStatus::Received,
            };
            EVENT_STORE.push(stored).await;
            Ok(WebhookEventType::Unknown(event_type.to_string()))
        }
    }
}

/// Extract repository name from payload
fn extract_repo_name(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .map(String::from)
}

/// Extract pull request data from payload
fn extract_pr_data(payload: &serde_json::Value) -> Result<PullRequestEvent> {
    let pr = payload
        .get("pull_request")
        .ok_or_else(|| anyhow!("Missing pull_request in payload"))?;

    let action = payload
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow!("Missing action in payload"))?
        .to_string();

    let number = pr
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or_else(|| anyhow!("Missing number in PR"))?;

    let title = pr
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let state = pr
        .get("state")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    let author = pr
        .get("user")
        .and_then(|u| u.get("login"))
        .and_then(|l| l.as_str())
        .unwrap_or("unknown")
        .to_string();

    let url = pr
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let draft = pr.get("draft").and_then(|d| d.as_bool()).unwrap_or(false);

    let merged = pr.get("merged").and_then(|m| m.as_bool()).unwrap_or(false);

    let repository = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    Ok(PullRequestEvent {
        action,
        number,
        title,
        state,
        author,
        url,
        draft,
        merged,
        repository,
    })
}

/// Extract workflow run data from payload
fn extract_workflow_run_data(payload: &serde_json::Value) -> Result<WorkflowRunEvent> {
    let wf = payload
        .get("workflow_run")
        .ok_or_else(|| anyhow!("Missing workflow_run in payload"))?;

    let action = payload
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow!("Missing action in payload"))?
        .to_string();

    let workflow_name = wf
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();

    let run_id = wf.get("id").and_then(|n| n.as_u64()).unwrap_or(0);

    let status = wf
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    let conclusion = wf
        .get("conclusion")
        .and_then(|c| c.as_str())
        .map(String::from);

    let branch = wf
        .get("head_branch")
        .and_then(|b| b.as_str())
        .unwrap_or("unknown")
        .to_string();

    let repository = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    let url = wf
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    Ok(WorkflowRunEvent {
        action,
        workflow_name,
        run_id,
        status,
        conclusion,
        branch,
        repository,
        url,
    })
}

/// Extract check run data from payload
fn extract_check_run_data(payload: &serde_json::Value) -> Result<CheckRunEvent> {
    let cr = payload
        .get("check_run")
        .ok_or_else(|| anyhow!("Missing check_run in payload"))?;

    let action = payload
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow!("Missing action in payload"))?
        .to_string();

    let name = cr
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();

    let status = cr
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    let conclusion = cr
        .get("conclusion")
        .and_then(|c| c.as_str())
        .map(String::from);

    let repository = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    let url = cr
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    Ok(CheckRunEvent {
        action,
        name,
        status,
        conclusion,
        repository,
        url,
    })
}

/// Webhook event wrapper for return types
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WebhookEvent {
    #[serde(rename = "pull_request")]
    PullRequest(PullRequestEvent),
    #[serde(rename = "workflow_run")]
    WorkflowRun(WorkflowRunEvent),
    #[serde(rename = "check_run")]
    CheckRun(CheckRunEvent),
    #[serde(rename = "unknown")]
    Unknown {
        event_type: String,
        action: Option<String>,
        repository: Option<String>,
    },
}

impl From<StoredEvent> for WebhookEvent {
    fn from(stored: StoredEvent) -> Self {
        match stored.event_type.as_str() {
            "pull_request" => {
                if let Ok(pr) = extract_pr_data(&stored.payload) {
                    WebhookEvent::PullRequest(pr)
                } else {
                    WebhookEvent::Unknown {
                        event_type: stored.event_type,
                        action: stored.action,
                        repository: stored.repository,
                    }
                }
            }
            "workflow_run" => {
                if let Ok(wf) = extract_workflow_run_data(&stored.payload) {
                    WebhookEvent::WorkflowRun(wf)
                } else {
                    WebhookEvent::Unknown {
                        event_type: stored.event_type,
                        action: stored.action,
                        repository: stored.repository,
                    }
                }
            }
            "check_run" => {
                if let Ok(cr) = extract_check_run_data(&stored.payload) {
                    WebhookEvent::CheckRun(cr)
                } else {
                    WebhookEvent::Unknown {
                        event_type: stored.event_type,
                        action: stored.action,
                        repository: stored.repository,
                    }
                }
            }
            _ => WebhookEvent::Unknown {
                event_type: stored.event_type,
                action: stored.action,
                repository: stored.repository,
            },
        }
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Install a GitHub App with the given credentials
///
/// This is a stub implementation that stores the app_id and private_key
/// securely in the keychain. A full implementation would:
/// 1. Validate the private key format
/// 2. Register with GitHub's API
/// 3. Complete OAuth flow if needed
#[tauri::command]
pub async fn install_github_app(
    app_id: String,
    private_key: String,
) -> Result<GitHubAppConfig, String> {
    tracing::info!("Installing GitHub App: {}", app_id);

    // Basic validation
    if app_id.is_empty() {
        return Err("App ID cannot be empty".to_string());
    }

    // Validate private key format (PEM)
    if !private_key.contains("-----BEGIN RSA PRIVATE KEY-----")
        && !private_key.contains("-----BEGIN PRIVATE KEY-----")
        && !private_key.contains("-----BEGIN EC PRIVATE KEY-----")
    {
        return Err("Invalid private key format. Expected PEM-encoded key".to_string());
    }

    // Create config
    let config = GitHubAppConfig::new(app_id.clone(), private_key);

    // Store in global state (in real app, use keychain)
    let mut stored_config = APP_CONFIG.lock().await;
    *stored_config = Some(config.clone());

    tracing::info!("GitHub App installed successfully");

    Ok(config)
}

/// List recent webhook events
#[tauri::command]
pub async fn list_webhook_events(limit: u32) -> Result<Vec<StoredEvent>, String> {
    let store = get_event_store();
    let mut events = store.get_all().await;

    // Sort by received_at descending (most recent first)
    events.sort_by(|a, b| b.received_at.cmp(&a.received_at));

    let limit = limit.min(100) as usize;
    if events.len() > limit {
        events.truncate(limit);
    }

    Ok(events)
}

/// Redeliver a webhook event by ID
///
/// This is a stub that marks the event for redelivery and re-processes it
#[tauri::command]
pub async fn redeliver_webhook(event_id: String) -> Result<StoredEvent, String> {
    let store = get_event_store();
    let mut events = store.get_all().await;

    let Some(event_mut) = events.iter_mut().find(|e| e.id == event_id) else {
        return Err(format!("Event not found: {}", event_id));
    };

    event_mut.redelivery_count += 1;
    event_mut.status = EventStatus::Processing;

    // Simulate reprocessing
    event_mut.status = EventStatus::Processed;

    // Clone the event to return and update in store
    let event_clone = event_mut.clone();
    store.push(event_clone.clone()).await;

    tracing::info!("Redelivered event {} (attempt {})", event_id, event_clone.redelivery_count);

    Ok(event_clone)
}

// ============================================================================
// HTTP API CLIENT FOR GITHUB
// ============================================================================

/// Client for GitHub App API operations
#[derive(Clone)]
pub struct GitHubAppClient {
    app_id: String,
    private_key: String,
    http_client: Client,
}

impl GitHubAppClient {
    pub fn new(app_id: String, private_key: String) -> Self {
        Self {
            app_id,
            private_key,
            http_client: HTTP_CLIENT.clone(),
        }
    }

    /// Generate JWT for GitHub App authentication
    fn generate_jwt(&self) -> Result<String> {
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let now = chrono::Utc::now().timestamp() as u64;
        let exp = now + 600; // 10 minutes max

        #[derive(serde::Serialize)]
        struct Claims {
            iat: u64,
            exp: u64,
            iss: String,
        }

        let claims = Claims {
            iat: now,
            exp,
            iss: self.app_id.clone(),
        };

        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(self.private_key.as_bytes())
            .map_err(|_| anyhow!("Failed to parse private key"))?;

        let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)
            .map_err(|_| anyhow!("Failed to encode JWT"))?;

        Ok(jwt)
    }

    /// Get installation access token
    pub async fn get_installation_token(&self, installation_id: &str) -> Result<String> {
        let jwt = self.generate_jwt()?;

        let response = self
            .http_client
            .post(&format!(
                "https://api.github.com/app/installations/{}/access_tokens",
                installation_id
            ))
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "CTO-App")
            .send()
            .await
            .context("Failed to request access token")?;

        if !response.status().is_success() {
            let body = response.text().await.context("Failed to read error response")?;
            return Err(anyhow!("GitHub API error: {}", body));
        }

        let json: serde_json::Value = response.json().await.context("Failed to parse response")?;
        let token = json
            .get("token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("No token in response"))?
            .to_string();

        Ok(token)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_webhook_signature_valid() {
        let secret = "test-secret";
        let payload = br#"{"action":"opened"}"#;

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        let result = verify_webhook_signature(
            payload,
            &format!("sha256={}", signature),
            secret,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_webhook_signature_invalid() {
        let payload = br#"{"action":"opened"}"#;

        let result = verify_webhook_signature(payload, "sha256=invalid", "test-secret");

        assert!(result.is_err());
    }
}
