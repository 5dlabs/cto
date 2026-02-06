//! OAuth callback handler for Linear agent applications.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use chrono::Utc;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::server::ServerState;

/// Query parameters from OAuth callback.
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Response from Linear token exchange.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[allow(dead_code)]
    pub token_type: String,
    pub expires_in: Option<i64>,
    #[allow(dead_code)]
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenErrorResponse {
    error: String,
    #[allow(dead_code)]
    error_description: Option<String>,
}

/// Handle OAuth callback from Linear.
#[allow(clippy::too_many_lines)]
pub async fn handle_oauth_callback(
    State(state): State<ServerState>,
    Query(params): Query<OAuthCallback>,
) -> impl IntoResponse {
    if let Some(error) = &params.error {
        let description = params.error_description.as_deref().unwrap_or("No description");
        error!(error = %error, description = %description, "OAuth authorization failed");
        return Html(format!(
            r#"<!DOCTYPE html>
<html><head><title>Authorization Failed</title></head>
<body><h1>❌ Authorization Failed</h1>
<p><strong>Error:</strong> {error}</p>
<p><strong>Description:</strong> {description}</p>
<p><a href="/">Return</a></p></body></html>"#
        ))
        .into_response();
    }

    let agent_name = params.state.as_deref().unwrap_or("morgan");
    info!(agent = %agent_name, "Received OAuth callback");

    // Get app config
    let config = Config::load();
    let Some(app_config) = config.linear.apps.get(agent_name) else {
        warn!(agent = %agent_name, "Unknown agent");
        return Html(format!(
            r#"<!DOCTYPE html>
<html><head><title>Unknown Agent</title></head>
<body><h1>❌ Unknown Agent</h1>
<p>Agent "{agent_name}" not configured.</p></body></html>"#
        ))
        .into_response();
    };

    let redirect_uri = config.linear.redirect_uri.clone();

    // Exchange code for token
    let token_result = exchange_code_for_token(
        &params.code,
        &app_config.client_id,
        &app_config.client_secret,
        &redirect_uri,
    )
    .await;

    match token_result {
        Ok(token_response) => {
            info!(agent = %agent_name, "Successfully obtained access token");

            // Store tokens
            let mut updated_config = config;
            if let Some(app) = updated_config.linear.apps.get_mut(agent_name) {
                app.access_token = Some(token_response.access_token.clone());
                if let Some(rt) = &token_response.refresh_token {
                    app.refresh_token = Some(rt.clone());
                }
                app.expires_at = token_response.expires_in.map(|secs| Utc::now().timestamp() + secs);
            }

            if let Err(e) = updated_config.save() {
                error!(agent = %agent_name, error = %e, "Failed to save tokens");
            }

            Html(format!(
                r#"<!DOCTYPE html>
<html><head><title>Authorization Successful</title></head>
<body><h1>✅ Authorization Successful</h1>
<p>Agent <strong>{agent_name}</strong> authorized.</p>
<p>Token saved. Close this window.</p></body></html>"#
            ))
            .into_response()
        }
        Err(e) => {
            error!(agent = %agent_name, error = %e, "Token exchange failed");
            Html(format!(
                r#"<!DOCTYPE html>
<html><head><title>Token Exchange Failed</title></head>
<body><h1>❌ Token Exchange Failed</h1>
<p><strong>Error:</strong> {e}</p>
<p><a href="/oauth/start">Try again</a></p></body></html>"#
            ))
            .into_response()
        }
    }
}

async fn exchange_code_for_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
    ];

    debug!(client_id = %client_id, "Exchanging code for token");
    let response = client
        .post("https://api.linear.app/oauth/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse: {e}"))?;
        debug!(has_refresh = token_response.refresh_token.is_some(), "Token exchange successful");
        Ok(token_response)
    } else {
        let error_body = response.text().await.unwrap_or_default();
        if let Ok(error_response) = serde_json::from_str::<TokenErrorResponse>(&error_body) {
            Err(format!("{} ({})", error_response.error, error_response.error_description.unwrap_or_default()))
        } else {
            Err(format!("Failed with status {status}: {error_body}"))
        }
    }
}

pub async fn refresh_access_token(
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    debug!(client_id = %client_id, "Refreshing token");
    let response = client
        .post("https://api.linear.app/oauth/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        let token_response: TokenResponse = response.json().await.map_err(|e| format!("Failed: {e}"))?;
        Ok(token_response)
    } else {
        let error_body = response.text().await.unwrap_or_default();
        Err(format!("Failed: {error_body}"))
    }
}

pub async fn handle_oauth_refresh(
    State(state): State<ServerState>,
    axum::extract::Path(agent): axum::extract::Path<String>,
) -> impl IntoResponse {
    info!(agent = %agent, "Manual token refresh");

    let config = Config::load();
    let Some(app_config) = config.linear.apps.get(&agent) else {
        return (StatusCode::NOT_FOUND, format!("Unknown agent")).into_response();
    };

    let Some(refresh_token) = &app_config.refresh_token else {
        return (StatusCode::BAD_REQUEST, "No refresh token").into_response();
    };

    match refresh_access_token(refresh_token, &app_config.client_id, &app_config.client_secret).await {
        Ok(token_response) => {
            let mut updated_config = config;
            if let Some(app) = updated_config.linear.apps.get_mut(&agent) {
                app.access_token = Some(token_response.access_token);
                if let Some(rt) = token_response.refresh_token {
                    app.refresh_token = Some(rt);
                }
                app.expires_at = token_response.expires_in.map(|secs| Utc::now().timestamp() + secs);
            }
            if let Err(e) = updated_config.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Save failed: {e}")).into_response();
            }
            (StatusCode::OK, "Token refreshed".to_string()).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_REQUEST, format!("Refresh failed: {e}")).into_response()
        }
    }
}

pub async fn handle_oauth_start(
    State(state): State<ServerState>,
    Query(params): Query<OAuthStartParams>,
) -> impl IntoResponse {
    let agent = params.agent.as_deref().unwrap_or("morgan");
    let config = Config::load();
    let Some(app_config) = config.linear.apps.get(agent) else {
        return (StatusCode::NOT_FOUND, format!("Unknown agent")).into_response();
    };

    let url = format!(
        "https://linear.app/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=read,write&prompt=consent&actor=app&state={}",
        app_config.client_id,
        urlencoding::encode(&config.linear.redirect_uri),
        agent
    );

    info!(agent = %agent, "Redirecting to Linear OAuth");
    Redirect::temporary(&url).into_response()
}

#[derive(Debug, Deserialize)]
pub struct OAuthStartParams {
    #[serde(default)]
    pub agent: Option<String>,
}
