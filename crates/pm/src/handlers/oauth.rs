//! OAuth callback handler for Linear agent applications.
//!
//! This module handles the OAuth authorization callback from Linear,
//! exchanging authorization codes for access tokens and refreshing them.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use base64::Engine;
use chrono::Utc;
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
pub struct TokenResponse {
    /// Access token for API calls.
    pub access_token: String,
    /// Token type (usually "Bearer").
    #[allow(dead_code)]
    pub token_type: String,
    /// Expiration time in seconds (typically 315360000 = 10 years for Linear).
    pub expires_in: Option<i64>,
    /// Scopes granted.
    #[allow(dead_code)]
    pub scope: Option<String>,
    /// Refresh token for obtaining new access tokens.
    pub refresh_token: Option<String>,
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
#[allow(clippy::too_many_lines)] // Complex function not easily split
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
    let (client_id, client_secret) = {
        let linear_config = match state.config.linear.read() {
            Ok(config) => config,
            Err(e) => {
                error!(error = %e, "Failed to acquire read lock on linear config");
                return Html(format!(
                    r"<!DOCTYPE html>
<html>
<head><title>Internal Error</title></head>
<body>
<h1>❌ Internal Error</h1>
<p>Failed to read configuration: {e}</p>
</body>
</html>"
                ))
                .into_response();
            }
        };
        let Some(app_config) = linear_config.get_app(agent_name) else {
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
        (
            app_config.client_id.clone(),
            app_config.client_secret.clone(),
        )
    };

    let redirect_uri = state
        .config
        .linear
        .read()
        .map(|c| c.redirect_uri.clone())
        .unwrap_or_default();

    // Exchange code for token
    let token_result =
        exchange_code_for_token(&params.code, &client_id, &client_secret, &redirect_uri).await;

    match token_result {
        Ok(token_response) => {
            info!(
                agent = %agent_name,
                has_refresh_token = token_response.refresh_token.is_some(),
                expires_in = ?token_response.expires_in,
                "Successfully obtained access token"
            );

            // Store the token in the Kubernetes secret
            let store_result = store_access_token(
                &state.kube_client,
                &state.config.namespace,
                agent_name,
                &token_response.access_token,
                token_response.refresh_token.as_deref(),
                token_response.expires_in,
            )
            .await;

            match store_result {
                Ok(()) => {
                    info!(agent = %agent_name, "Access token stored in Kubernetes secret");

                    // Also update in-memory config so subsequent calls see the fresh token
                    if let Ok(mut linear_config) = state.config.linear.write() {
                        linear_config.update_tokens(
                            agent_name,
                            &token_response.access_token,
                            token_response.refresh_token.as_deref(),
                            token_response.expires_in,
                        );
                        info!(agent = %agent_name, "Updated in-memory token config");
                    }

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

/// Store the access token (and optionally refresh token) in the Kubernetes secret for the agent.
///
/// This patches the `linear-app-{agent}` secret to add/update token fields.
async fn store_access_token(
    kube_client: &kube::Client,
    namespace: &str,
    agent_name: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<i64>,
) -> Result<(), String> {
    let secret_name = format!("linear-app-{}", agent_name.to_lowercase());
    let secrets: Api<Secret> = Api::namespaced(kube_client.clone(), namespace);

    // Base64 encode the tokens for Kubernetes secret
    let encoded_access_token = base64::engine::general_purpose::STANDARD.encode(access_token);

    // Build the patch data
    let mut patch_data = json!({
        "access_token": encoded_access_token
    });

    // Add refresh token if provided
    if let Some(rt) = refresh_token {
        let encoded_refresh_token = base64::engine::general_purpose::STANDARD.encode(rt);
        patch_data["refresh_token"] = json!(encoded_refresh_token);
    }

    // Always update expires_at: calculate new value if expires_in is provided,
    // otherwise clear it to avoid preserving stale expiration timestamps from
    // previous tokens that may already be expired.
    let encoded_expires_at = if let Some(expires_in_secs) = expires_in {
        let expires_at = Utc::now().timestamp() + expires_in_secs;
        base64::engine::general_purpose::STANDARD.encode(expires_at.to_string())
    } else {
        // Empty string clears the field in the K8s secret
        String::new()
    };
    patch_data["expires_at"] = json!(encoded_expires_at);

    let patch = json!({
        "data": patch_data
    });

    debug!(
        secret = %secret_name,
        namespace = %namespace,
        has_refresh_token = refresh_token.is_some(),
        has_expires_at = expires_in.is_some(),
        "Patching Kubernetes secret with tokens"
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
        "Successfully stored tokens in Kubernetes secret"
    );

    Ok(())
}

/// Public wrapper for `store_access_token` for use by other modules.
pub async fn store_access_token_public(
    kube_client: &kube::Client,
    namespace: &str,
    agent_name: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<i64>,
) -> Result<(), String> {
    store_access_token(
        kube_client,
        namespace,
        agent_name,
        access_token,
        refresh_token,
        expires_in,
    )
    .await
}

/// Exchange an authorization code for an access token.
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
            .map_err(|e| format!("Failed to parse token response: {e}"))?;

        debug!(
            has_refresh_token = token_response.refresh_token.is_some(),
            expires_in = ?token_response.expires_in,
            "Token exchange successful"
        );

        Ok(token_response)
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

/// Refresh an access token using a refresh token.
///
/// This is used to obtain a new access token when the current one has expired.
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

    debug!(client_id = %client_id, "Refreshing access token");

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

        debug!(
            has_new_refresh_token = token_response.refresh_token.is_some(),
            expires_in = ?token_response.expires_in,
            "Token refresh successful"
        );

        Ok(token_response)
    } else {
        let error_body = response.text().await.unwrap_or_default();

        if let Ok(error_response) = serde_json::from_str::<TokenErrorResponse>(&error_body) {
            Err(format!(
                "Token refresh failed: {} ({})",
                error_response.error,
                error_response.error_description.unwrap_or_default()
            ))
        } else {
            Err(format!(
                "Token refresh failed with status {status}: {error_body}"
            ))
        }
    }
}

/// Handle manual token refresh request for an agent.
///
/// This endpoint triggers a token refresh for the specified agent using
/// the stored refresh token. Useful for debugging or forcing a refresh.
pub async fn handle_oauth_refresh(
    State(state): State<AppState>,
    axum::extract::Path(agent): axum::extract::Path<String>,
) -> impl IntoResponse {
    info!(agent = %agent, "Manual token refresh requested");

    // Get the app config for this agent
    let (refresh_token, client_id, client_secret) = {
        let linear_config = match state.config.linear.read() {
            Ok(config) => config,
            Err(e) => {
                error!(error = %e, "Failed to acquire read lock on linear config");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read configuration: {e}"),
                )
                    .into_response();
            }
        };
        let Some(app_config) = linear_config.get_app(&agent) else {
            return (
                StatusCode::NOT_FOUND,
                format!("Agent '{agent}' is not configured"),
            )
                .into_response();
        };

        let Some(rt) = app_config.refresh_token.clone() else {
            return (
                StatusCode::BAD_REQUEST,
                format!("No refresh token available for agent '{agent}'"),
            )
                .into_response();
        };

        (
            rt,
            app_config.client_id.clone(),
            app_config.client_secret.clone(),
        )
    };

    // Refresh the token
    match refresh_access_token(&refresh_token, &client_id, &client_secret).await {
        Ok(token_response) => {
            // Store the new tokens in K8s
            let store_result = store_access_token(
                &state.kube_client,
                &state.config.namespace,
                &agent,
                &token_response.access_token,
                token_response.refresh_token.as_deref(),
                token_response.expires_in,
            )
            .await;

            match store_result {
                Ok(()) => {
                    info!(agent = %agent, "Token refresh successful and stored in K8s");

                    // Also update in-memory config so subsequent calls see the fresh token
                    if let Ok(mut linear_config) = state.config.linear.write() {
                        linear_config.update_tokens(
                            &agent,
                            &token_response.access_token,
                            token_response.refresh_token.as_deref(),
                            token_response.expires_in,
                        );
                        info!(agent = %agent, "Updated in-memory token config");
                    }

                    (
                        StatusCode::OK,
                        format!("Token refreshed successfully for agent '{agent}'"),
                    )
                        .into_response()
                }
                Err(e) => {
                    error!(agent = %agent, error = %e, "Failed to store refreshed token");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Token refreshed but storage failed: {e}"),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!(agent = %agent, error = %e, "Token refresh failed");
            (
                StatusCode::BAD_REQUEST,
                format!("Token refresh failed: {e}"),
            )
                .into_response()
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

    let url = {
        let linear_config = match state.config.linear.read() {
            Ok(config) => config,
            Err(e) => {
                error!(error = %e, "Failed to acquire read lock on linear config");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read configuration: {e}"),
                )
                    .into_response();
            }
        };
        linear_config.oauth_url(agent)
    };

    let Some(url) = url else {
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
