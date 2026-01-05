//! OAuth callback handler for Linear agent applications.
//!
//! This module handles the OAuth authorization callback from Linear,
//! exchanging authorization codes for access tokens.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, Patch, PatchParams};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::server::AppState;

/// Query parameters from OAuth callback.
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    /// Authorization code from Linear.
    pub code: String,
    /// State parameter (should match what we sent).
    #[serde(default)]
    pub state: Option<String>,
    /// Error if authorization failed.
    #[serde(default)]
    pub error: Option<String>,
    /// Error description.
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Response from Linear token exchange.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    /// Access token for API calls.
    access_token: String,
    /// Token type (usually "Bearer").
    #[allow(dead_code)]
    token_type: String,
    /// Expiration time in seconds.
    #[allow(dead_code)]
    expires_in: Option<i64>,
    /// Scopes granted.
    #[allow(dead_code)]
    scope: Option<String>,
}

/// Error response from Linear token exchange.
#[derive(Debug, Deserialize)]
struct TokenErrorResponse {
    error: String,
    #[allow(dead_code)]
    error_description: Option<String>,
}

/// Handle OAuth callback from Linear.
///
/// This endpoint is called by Linear after user authorizes the app.
/// We exchange the code for an access token and store it.
#[allow(clippy::too_many_lines)]
pub async fn handle_oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallback>,
) -> impl IntoResponse {
    // Check for OAuth error
    if let Some(error) = &params.error {
        let description = params
            .error_description
            .as_deref()
            .unwrap_or("No description");
        error!(error = %error, description = %description, "OAuth authorization failed");
        return Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Authorization Failed</title></head>
<body>
<h1>❌ Authorization Failed</h1>
<p><strong>Error:</strong> {error}</p>
<p><strong>Description:</strong> {description}</p>
<p><a href="/">Return to home</a></p>
</body>
</html>"#
        ))
        .into_response();
    }

    // Parse the state to get the agent name
    let agent_name = params.state.as_deref().unwrap_or("unknown");

    info!(
        agent = %agent_name,
        "Received OAuth callback, exchanging code for token"
    );

    // Get the app config for this agent
    let app_config = state.config.linear.get_app(agent_name);
    let Some(app_config) = app_config else {
        warn!(agent = %agent_name, "Unknown agent in OAuth callback");
        return Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Unknown Agent</title></head>
<body>
<h1>❌ Unknown Agent</h1>
<p>Agent "{agent_name}" is not configured.</p>
<p><a href="/">Return to home</a></p>
</body>
</html>"#
        ))
        .into_response();
    };

    // Exchange code for token
    let token_result = exchange_code_for_token(
        &params.code,
        &app_config.client_id,
        &app_config.client_secret,
        &state.config.linear.redirect_uri,
    )
    .await;

    match token_result {
        Ok(token) => {
            info!(
                agent = %agent_name,
                "Successfully obtained access token"
            );

            // Store the token in the Kubernetes secret
            let store_result = store_access_token(
                &state.kube_client,
                &state.config.namespace,
                agent_name,
                &token,
            )
            .await;

            match store_result {
                Ok(()) => {
                    info!(agent = %agent_name, "Access token stored in Kubernetes secret");
                    Html(format!(
                        r"<!DOCTYPE html>
<html>
<head><title>Authorization Successful</title></head>
<body>
<h1>✅ Authorization Successful</h1>
<p>Agent <strong>{agent_name}</strong> has been authorized.</p>
<p>The access token has been stored securely.</p>
<p>You can close this window.</p>
</body>
</html>"
                    ))
                    .into_response()
                }
                Err(e) => {
                    error!(agent = %agent_name, error = %e, "Failed to store access token");
                    Html(format!(
                        r"<!DOCTYPE html>
<html>
<head><title>Token Storage Failed</title></head>
<body>
<h1>⚠️ Authorization Partial Success</h1>
<p>Agent <strong>{agent_name}</strong> was authorized, but the token could not be stored.</p>
<p><strong>Error:</strong> {e}</p>
<p>Please contact an administrator.</p>
</body>
</html>"
                    ))
                    .into_response()
                }
            }
        }
        Err(e) => {
            error!(agent = %agent_name, error = %e, "Token exchange failed");
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Token Exchange Failed</title></head>
<body>
<h1>❌ Token Exchange Failed</h1>
<p>Failed to exchange authorization code for access token.</p>
<p><strong>Error:</strong> {e}</p>
<p><a href="/">Try again</a></p>
</body>
</html>"#
            ))
            .into_response()
        }
    }
}

/// Store the access token in the Kubernetes secret for the agent.
///
/// This patches the `linear-app-{agent}` secret to add/update the `access_token` field.
async fn store_access_token(
    kube_client: &kube::Client,
    namespace: &str,
    agent_name: &str,
    token: &str,
) -> Result<(), String> {
    let secret_name = format!("linear-app-{}", agent_name.to_lowercase());
    let secrets: Api<Secret> = Api::namespaced(kube_client.clone(), namespace);

    // Base64 encode the token for Kubernetes secret
    let encoded_token = base64::engine::general_purpose::STANDARD.encode(token);

    // Create a strategic merge patch to update just the access_token field
    let patch = json!({
        "data": {
            "access_token": encoded_token
        }
    });

    debug!(
        secret = %secret_name,
        namespace = %namespace,
        "Patching Kubernetes secret with access token"
    );

    secrets
        .patch(
            &secret_name,
            &PatchParams::default(),
            &Patch::Strategic(patch),
        )
        .await
        .map_err(|e| format!("Failed to patch secret {secret_name}: {e}"))?;

    info!(
        secret = %secret_name,
        agent = %agent_name,
        "Successfully stored access token in Kubernetes secret"
    );

    Ok(())
}

/// Exchange an authorization code for an access token.
async fn exchange_code_for_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<String, String> {
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
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        Ok(token_response.access_token)
    } else {
        let error_body = response.text().await.unwrap_or_default();

        // Try to parse as error response
        if let Ok(error_response) = serde_json::from_str::<TokenErrorResponse>(&error_body) {
            Err(format!(
                "Token exchange failed: {} ({})",
                error_response.error,
                error_response.error_description.unwrap_or_default()
            ))
        } else {
            Err(format!(
                "Token exchange failed with status {status}: {error_body}"
            ))
        }
    }
}

/// Generate OAuth authorization URL for an agent.
///
/// This is a helper endpoint that redirects to Linear's authorization page.
pub async fn handle_oauth_start(
    State(state): State<AppState>,
    Query(params): Query<OAuthStartParams>,
) -> impl IntoResponse {
    let agent = params.agent.as_deref().unwrap_or("morgan");

    let Some(url) = state.config.linear.oauth_url(agent) else {
        return (
            StatusCode::NOT_FOUND,
            format!("Agent '{agent}' is not configured"),
        )
            .into_response();
    };

    // Add state parameter for agent identification
    let url_with_state = format!("{url}&state={agent}");

    info!(agent = %agent, "Redirecting to Linear OAuth");
    Redirect::temporary(&url_with_state).into_response()
}

/// Query parameters for OAuth start.
#[derive(Debug, Deserialize)]
pub struct OAuthStartParams {
    /// Agent name to authorize.
    #[serde(default)]
    pub agent: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_callback_deserialize() {
        let params: OAuthCallback =
            serde_json::from_str(r#"{"code": "abc123", "state": "rex"}"#).unwrap();
        assert_eq!(params.code, "abc123");
        assert_eq!(params.state, Some("rex".to_string()));
    }

    #[test]
    fn test_oauth_callback_error_deserialize() {
        let params: OAuthCallback = serde_json::from_str(
            r#"{"code": "", "error": "access_denied", "error_description": "User denied"}"#,
        )
        .unwrap();
        assert_eq!(params.error, Some("access_denied".to_string()));
    }
}
