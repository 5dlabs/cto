//! HTTP API for Play session management.
//!
//! Provides endpoints for:
//! - Starting new Play sessions (called by MCP server)
//! - Querying session status
//! - Listing active sessions

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

use super::session::{
    PlaySession, SessionStatus, SessionStore, SessionStoreHandle, StartSessionRequest,
    StartSessionResponse,
};

/// State for the Play API server.
pub struct PlayApiState {
    /// Session store
    pub sessions: SessionStoreHandle,
    /// Default namespace
    pub namespace: String,
}

impl PlayApiState {
    /// Create new API state.
    #[must_use]
    pub fn new(namespace: &str) -> Self {
        Self {
            sessions: Arc::new(SessionStore::new()),
            namespace: namespace.to_string(),
        }
    }

    /// Create API state with an existing session store.
    #[must_use]
    pub fn with_store(sessions: SessionStoreHandle, namespace: &str) -> Self {
        Self {
            sessions,
            namespace: namespace.to_string(),
        }
    }
}

/// Build the Play API router.
pub fn build_play_api_router(state: Arc<PlayApiState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/session/start", post(start_session_handler))
        .route("/api/v1/session/{play_id}", get(get_session_handler))
        .route("/api/v1/sessions", get(list_sessions_handler))
        .route("/api/v1/sessions/active", get(list_active_sessions_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Start the Play API server.
///
/// # Errors
///
/// Returns an error if the server fails to start or bind to the address.
pub async fn run_play_api_server(state: Arc<PlayApiState>, addr: &str) -> anyhow::Result<()> {
    let app = build_play_api_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Healer Play API server listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    service: &'static str,
}

/// Health check handler.
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        service: "healer-play-api",
    })
}

/// Start a new Play session.
///
/// Uses atomic check-and-insert to prevent TOCTOU race conditions.
/// Concurrent requests with the same `play_id` will be properly rejected with 409.
async fn start_session_handler(
    State(state): State<Arc<PlayApiState>>,
    Json(request): Json<StartSessionRequest>,
) -> impl IntoResponse {
    let play_id = request.play_id.clone();
    let task_count = request.tasks.len();
    let agent_count = request.cto_config.agents.len();

    info!(
        play_id = %play_id,
        repository = %request.repository,
        tasks = %task_count,
        agents_configured = %agent_count,
        "Received session start request from MCP server"
    );

    // Atomically try to start the session - this holds the write lock during
    // both the existence check and insertion, preventing race conditions
    match state.sessions.try_start_session(request).await {
        Ok(session) => {
            // Log expected tools for each agent
            for (agent_name, agent_config) in &session.cto_config.agents {
                let remote_tools = agent_config.tools.remote.len();
                let local_servers = agent_config.tools.local_servers.len();
                info!(
                    play_id = %play_id,
                    agent = %agent_name,
                    remote_tools = %remote_tools,
                    local_servers = %local_servers,
                    "Agent tool expectations registered"
                );
            }

            (
                StatusCode::OK,
                Json(StartSessionResponse {
                    status: "ok",
                    session_id: session.play_id,
                    message: format!(
                        "Session started with {} tasks and {} agent configurations",
                        task_count, agent_count
                    ),
                }),
            )
        }
        Err(_existing) => {
            // Session already exists and is active - return 409 Conflict
            (
                StatusCode::CONFLICT,
                Json(StartSessionResponse {
                    status: "error",
                    session_id: play_id,
                    message: "Session already exists and is active".to_string(),
                }),
            )
        }
    }
}

/// Session detail response.
#[derive(Debug, Serialize)]
struct SessionDetailResponse {
    status: &'static str,
    session: Option<PlaySession>,
    message: Option<String>,
}

/// Get a specific session.
async fn get_session_handler(
    State(state): State<Arc<PlayApiState>>,
    axum::extract::Path(play_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.sessions.get_session(&play_id).await {
        Some(session) => (
            StatusCode::OK,
            Json(SessionDetailResponse {
                status: "ok",
                session: Some(session),
                message: None,
            }),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(SessionDetailResponse {
                status: "error",
                session: None,
                message: Some(format!("Session not found: {play_id}")),
            }),
        ),
    }
}

/// Sessions list response.
#[derive(Debug, Serialize)]
struct SessionsListResponse {
    status: &'static str,
    count: usize,
    sessions: Vec<SessionSummary>,
}

/// Summary of a session (for listing).
#[derive(Debug, Serialize)]
struct SessionSummary {
    play_id: String,
    repository: String,
    status: SessionStatus,
    task_count: usize,
    issue_count: usize,
    started_at: chrono::DateTime<chrono::Utc>,
}

/// List all sessions (active and inactive).
async fn list_sessions_handler(State(state): State<Arc<PlayApiState>>) -> impl IntoResponse {
    let sessions = state.sessions.get_all_sessions().await;

    let summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|s| SessionSummary {
            play_id: s.play_id,
            repository: s.repository,
            status: s.status,
            task_count: s.tasks.len(),
            issue_count: s.issues.len(),
            started_at: s.started_at,
        })
        .collect();

    Json(SessionsListResponse {
        status: "ok",
        count: summaries.len(),
        sessions: summaries,
    })
}

/// List only active sessions.
async fn list_active_sessions_handler(State(state): State<Arc<PlayApiState>>) -> impl IntoResponse {
    let sessions = state.sessions.get_active_sessions().await;

    let summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|s| SessionSummary {
            play_id: s.play_id,
            repository: s.repository,
            status: s.status,
            task_count: s.tasks.len(),
            issue_count: s.issues.len(),
            started_at: s.started_at,
        })
        .collect();

    Json(SessionsListResponse {
        status: "ok",
        count: summaries.len(),
        sessions: summaries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(PlayApiState::new("cto"));
        let app = build_play_api_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_start_session() {
        let state = Arc::new(PlayApiState::new("cto"));
        let app = build_play_api_router(state);

        let request_body = serde_json::json!({
            "play_id": "test-play-1",
            "repository": "5dlabs/test",
            "tasks": [],
            "cto_config": {
                "agents": {}
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/session/start")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
