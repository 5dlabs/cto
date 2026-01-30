//! GitHub OAuth and repository management commands

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::Database;
use crate::error::AppError;
use crate::keychain::{self, CredentialKey};

// GitHub OAuth App credentials (CTO Lite app)
// These are public - the secret is validated server-side
const GITHUB_CLIENT_ID: &str = "Ov23liXXXXXXXXXXXXXX"; // TODO: Replace with real client ID
const GITHUB_REDIRECT_URI: &str = "http://localhost:19284/callback/github";

/// GitHub connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubStatus {
    pub connected: bool,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
}

/// GitHub user info from API
#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    avatar_url: String,
    email: Option<String>,
}

/// Repository info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub full_name: String,
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub private: bool,
    pub description: Option<String>,
}

/// GitHub repo from API
#[derive(Debug, Deserialize)]
struct GitHubRepo {
    id: u64,
    full_name: String,
    owner: GitHubRepoOwner,
    name: String,
    default_branch: String,
    private: bool,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoOwner {
    login: String,
}

/// Start GitHub OAuth flow
/// Returns the authorization URL to open in browser
#[tauri::command]
pub async fn start_github_oauth() -> Result<String, AppError> {
    // Generate state parameter for CSRF protection
    let state = uuid::Uuid::new_v4().to_string();
    
    // Build authorization URL
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?\
        client_id={}&\
        redirect_uri={}&\
        scope=repo,read:user,user:email&\
        state={}",
        GITHUB_CLIENT_ID,
        urlencoding::encode(GITHUB_REDIRECT_URI),
        state
    );

    // TODO: Start local HTTP server to handle callback
    // For now, return the URL and handle callback separately

    tracing::info!("Starting GitHub OAuth flow");
    Ok(auth_url)
}

/// Get GitHub connection status
#[tauri::command]
pub async fn get_github_status(db: State<'_, Database>) -> Result<GitHubStatus, AppError> {
    // Check if we have a token
    let has_token = keychain::has_credential(CredentialKey::GithubAccessToken)?;
    
    if !has_token {
        return Ok(GitHubStatus {
            connected: false,
            username: None,
            avatar_url: None,
            email: None,
        });
    }

    // Try to get user info
    let token = keychain::get_credential(CredentialKey::GithubAccessToken)?
        .ok_or_else(|| AppError::NotConfigured("GitHub token not found".to_string()))?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "CTO-Lite")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !response.status().is_success() {
        // Token might be expired
        return Ok(GitHubStatus {
            connected: false,
            username: None,
            avatar_url: None,
            email: None,
        });
    }

    let user: GitHubUser = response.json().await?;

    // Store username in db for quick access
    db.set_config("github_username", &user.login)?;

    Ok(GitHubStatus {
        connected: true,
        username: Some(user.login),
        avatar_url: Some(user.avatar_url),
        email: user.email,
    })
}

/// Disconnect GitHub
#[tauri::command]
pub async fn disconnect_github(db: State<'_, Database>) -> Result<(), AppError> {
    keychain::delete_credential(CredentialKey::GithubAccessToken)?;
    keychain::delete_credential(CredentialKey::GithubRefreshToken)?;
    
    // Clear stored username
    db.set_config("github_username", "")?;
    
    tracing::info!("Disconnected from GitHub");
    Ok(())
}

/// List repositories the user has access to
#[tauri::command]
pub async fn list_repositories() -> Result<Vec<Repository>, AppError> {
    let token = keychain::get_credential(CredentialKey::GithubAccessToken)?
        .ok_or_else(|| AppError::NotConfigured("GitHub not connected".to_string()))?;

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user/repos")
        .query(&[
            ("sort", "updated"),
            ("per_page", "100"),
            ("affiliation", "owner,collaborator,organization_member"),
        ])
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "CTO-Lite")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::OAuthError(
            format!("GitHub API error {}: {}", status, body)
        ));
    }

    let repos: Vec<GitHubRepo> = response.json().await?;

    Ok(repos
        .into_iter()
        .map(|r| Repository {
            id: r.id.to_string(),
            full_name: r.full_name,
            owner: r.owner.login,
            name: r.name,
            default_branch: r.default_branch,
            private: r.private,
            description: r.description,
        })
        .collect())
}
