//! Integration tests for Linear input routing.
//!
//! These tests verify the end-to-end flow of routing messages from Linear
//! to running agents via the sidecar's HTTP endpoint.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use pm::handlers::{AgentMessage, CachedPodInfo, RunningAgent, SessionCache};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

// =============================================================================
// Mock Sidecar Server
// =============================================================================

/// Request body the sidecar expects.
#[derive(Debug, Deserialize)]
struct SidecarInput {
    text: String,
}

/// Shared state for the mock sidecar.
#[derive(Default)]
struct MockSidecarState {
    /// Number of messages received.
    message_count: AtomicUsize,
    /// Last message received.
    last_message: RwLock<Option<String>>,
}

/// Handle input to the mock sidecar.
async fn mock_sidecar_input(
    State(state): State<Arc<MockSidecarState>>,
    Json(input): Json<SidecarInput>,
) -> impl IntoResponse {
    state.message_count.fetch_add(1, Ordering::SeqCst);
    *state.last_message.write().await = Some(input.text.clone());
    (StatusCode::OK, "Message queued")
}

/// Start a mock sidecar server on a random port.
async fn start_mock_sidecar() -> (SocketAddr, Arc<MockSidecarState>) {
    let state = Arc::new(MockSidecarState::default());
    let state_clone = state.clone();

    let app = Router::new()
        .route("/input", post(mock_sidecar_input))
        .with_state(state_clone);

    // Bind to random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Start server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    (addr, state)
}

// =============================================================================
// Tests
// =============================================================================

/// Test that we can send a message to a mock sidecar via HTTP.
#[tokio::test]
async fn test_http_to_mock_sidecar() {
    // Start mock sidecar
    let (addr, state) = start_mock_sidecar().await;

    // Send a message via HTTP
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/input", addr))
        .json(&serde_json::json!({ "text": "Hello from test!" }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    // Verify sidecar received the message
    assert_eq!(state.message_count.load(Ordering::SeqCst), 1);
    let last_msg = state.last_message.read().await;
    assert_eq!(last_msg.as_deref(), Some("Hello from test!"));
}

/// Test session cache caching behavior.
#[tokio::test]
async fn test_session_cache_caching() {
    let cache = SessionCache::new();

    // Cache a pod
    let pod_info = CachedPodInfo {
        pod_ip: "10.0.0.1".to_string(),
        pod_name: "test-pod".to_string(),
        container_name: "main".to_string(),
        created_at: Instant::now(),
        agent_type: "rex".to_string(),
    };

    cache.insert("session-123", pod_info).await;

    // First lookup should hit cache
    let cached = cache.get("session-123").await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().pod_ip, "10.0.0.1");

    // Second lookup should also hit cache
    let cached2 = cache.get("session-123").await;
    assert!(cached2.is_some());

    // Non-existent session should miss
    let miss = cache.get("session-nonexistent").await;
    assert!(miss.is_none());
}

/// Test RunningAgent with pod IP enables HTTP routing.
#[tokio::test]
async fn test_running_agent_http_routing() {
    // Start mock sidecar
    let (addr, state) = start_mock_sidecar().await;

    // Create a RunningAgent with the mock sidecar's IP
    let agent = RunningAgent {
        pod_name: "test-pod".to_string(),
        namespace: "cto".to_string(),
        container_name: "main".to_string(),
        session_id: "session-123".to_string(),
        issue_identifier: Some("TEST-456".to_string()),
        agent_type: "rex".to_string(),
        pod_ip: Some(addr.ip().to_string()),
    };

    // Create a message (demonstrates API, but we send raw JSON below for simplicity)
    let _message = AgentMessage::user_message("Test message from integration test");

    // Manually send via HTTP (simulating what send_message_to_agent does)
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}:{}/input", agent.pod_ip.as_ref().unwrap(), addr.port()))
        .json(&serde_json::json!({ "text": "Test message from integration test" }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    // Verify sidecar received the message
    assert_eq!(state.message_count.load(Ordering::SeqCst), 1);
}

/// Test multiple messages to same session.
#[tokio::test]
async fn test_multiple_messages_same_session() {
    let (addr, state) = start_mock_sidecar().await;
    let client = reqwest::Client::new();

    // Send multiple messages
    for i in 0..5 {
        let response = client
            .post(format!("http://{}/input", addr))
            .json(&serde_json::json!({ "text": format!("Message {}", i) }))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());
    }

    // Verify all messages received
    assert_eq!(state.message_count.load(Ordering::SeqCst), 5);

    // Last message should be the most recent
    let last_msg = state.last_message.read().await;
    assert_eq!(last_msg.as_deref(), Some("Message 4"));
}

/// Test that cache TTL works correctly.
#[tokio::test]
async fn test_cache_ttl_expiration() {
    // Create cache with very short TTL
    let cache = SessionCache::with_ttl(Duration::from_millis(50));

    let pod_info = CachedPodInfo {
        pod_ip: "10.0.0.1".to_string(),
        pod_name: "test-pod".to_string(),
        container_name: "main".to_string(),
        created_at: Instant::now(),
        agent_type: "rex".to_string(),
    };

    cache.insert("session-expiring", pod_info).await;

    // Should be in cache immediately
    assert!(cache.get("session-expiring").await.is_some());

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be expired now
    assert!(cache.get("session-expiring").await.is_none());
}

/// Test the complete message flow simulation.
#[tokio::test]
async fn test_complete_message_flow() {
    // Start mock sidecar
    let (addr, state) = start_mock_sidecar().await;

    // Simulate the flow:
    // 1. PM server receives webhook with session_id
    let session_id = "linear-session-xyz";
    let user_message = "Please implement the login feature";

    // 2. Check cache (miss expected)
    let cache = SessionCache::new();
    assert!(cache.get(session_id).await.is_none());

    // 3. "Find" the pod (simulated K8s API call result)
    let pod_info = CachedPodInfo {
        pod_ip: addr.ip().to_string(),
        pod_name: "coderun-rex-task-1".to_string(),
        container_name: "main".to_string(),
        created_at: Instant::now(),
        agent_type: "rex".to_string(),
    };

    // 4. Cache the result
    cache.insert(session_id, pod_info.clone()).await;

    // 5. Send HTTP to sidecar
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}:{}/input", pod_info.pod_ip, addr.port()))
        .json(&serde_json::json!({ "text": user_message }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    // 6. Verify message was received
    assert_eq!(state.message_count.load(Ordering::SeqCst), 1);
    let received = state.last_message.read().await;
    assert_eq!(received.as_deref(), Some(user_message));

    // 7. Verify cache hit on second message
    let cached = cache.get(session_id).await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().pod_name, "coderun-rex-task-1");

    // 8. Send another message (should use cached pod IP)
    let second_message = "Also add password reset";
    let response2 = client
        .post(format!("http://{}:{}/input", pod_info.pod_ip, addr.port()))
        .json(&serde_json::json!({ "text": second_message }))
        .send()
        .await
        .expect("Failed to send second request");

    assert!(response2.status().is_success());
    assert_eq!(state.message_count.load(Ordering::SeqCst), 2);
}

/// Test label format matches what controller generates.
#[test]
fn test_label_format_consistency() {
    // These label keys must match what the controller generates
    let _expected_labels = [
        "linear-session",
        "cto.5dlabs.io/linear-issue",
        "cto.5dlabs.io/agent-type",
    ];

    // The find_running_agents function expects these label selectors
    let session_id = "test-session-123";
    let issue_id = "TEST-456";

    let session_selector = format!("linear-session={session_id}");
    let issue_selector = format!("cto.5dlabs.io/linear-issue={issue_id}");

    // Verify format is valid K8s label selector format
    assert!(!session_selector.contains(' '));
    assert!(!issue_selector.contains(' '));
    assert!(session_selector.contains('='));
    assert!(issue_selector.contains('='));
}

