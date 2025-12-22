//! Agent communication for two-way interaction with running agents.
//!
//! This module handles forwarding user messages to running Claude agents
//! via direct HTTP calls to the sidecar endpoint.
//!
//! ## Architecture
//!
//! ```text
//! Linear Comment → PM Server → K8s API (get pod IP) → HTTP POST to Pod:8080/input
//!                     ↓
//!              [SessionCache]
//!              session_id → {pod_ip, expires_at}
//! ```

use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    Client as KubeClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// =============================================================================
// Session Cache for Pod IP Lookups
// =============================================================================

/// Cached information about a pod running an agent.
#[derive(Debug, Clone)]
pub struct CachedPodInfo {
    /// Pod IP address.
    pub pod_ip: String,
    /// Pod name.
    pub pod_name: String,
    /// Container name (usually "main" or "agent").
    pub container_name: String,
    /// When this cache entry was created.
    pub created_at: Instant,
    /// Agent type (intake, play, etc).
    pub agent_type: String,
}

impl CachedPodInfo {
    /// Check if this cache entry has expired.
    #[must_use]
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Session cache for fast pod IP lookups.
///
/// Maps Linear session IDs to pod information for efficient routing.
#[derive(Default)]
pub struct SessionCache {
    /// Map of session_id -> pod info.
    entries: RwLock<HashMap<String, CachedPodInfo>>,
    /// Cache entry TTL (default: 5 minutes).
    ttl: Duration,
}

impl SessionCache {
    /// Create a new session cache with default TTL (5 minutes).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create a new session cache with custom TTL.
    #[must_use]
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// Get cached pod info for a session.
    pub async fn get(&self, session_id: &str) -> Option<CachedPodInfo> {
        let entries = self.entries.read().await;
        entries.get(session_id).and_then(|info| {
            if info.is_expired(self.ttl) {
                None
            } else {
                Some(info.clone())
            }
        })
    }

    /// Insert or update pod info for a session.
    pub async fn insert(&self, session_id: impl Into<String>, info: CachedPodInfo) {
        let mut entries = self.entries.write().await;
        entries.insert(session_id.into(), info);
    }

    /// Remove a session from the cache.
    pub async fn remove(&self, session_id: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(session_id);
    }

    /// Clear expired entries from the cache.
    pub async fn evict_expired(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, info| !info.is_expired(self.ttl));
    }

    /// Get the number of cached entries.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

// =============================================================================
// Agent Router - Main entry point for routing messages
// =============================================================================

/// Router for sending messages to running agents.
///
/// Uses HTTP to communicate with the sidecar endpoint on each pod.
pub struct AgentRouter {
    /// Kubernetes client for pod lookups.
    kube_client: KubeClient,
    /// HTTP client for sidecar communication.
    http_client: reqwest::Client,
    /// Session cache for fast lookups.
    cache: SessionCache,
    /// Default namespace for pod lookups.
    namespace: String,
    /// Sidecar HTTP port (default: 8080).
    sidecar_port: u16,
}

impl AgentRouter {
    /// Create a new agent router.
    #[must_use]
    pub fn new(kube_client: KubeClient, namespace: impl Into<String>) -> Self {
        Self {
            kube_client,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache: SessionCache::new(),
            namespace: namespace.into(),
            sidecar_port: 8080,
        }
    }

    /// Create a new agent router with custom settings.
    #[must_use]
    pub fn with_settings(
        kube_client: KubeClient,
        namespace: impl Into<String>,
        cache_ttl: Duration,
        sidecar_port: u16,
    ) -> Self {
        Self {
            kube_client,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache: SessionCache::with_ttl(cache_ttl),
            namespace: namespace.into(),
            sidecar_port,
        }
    }

    /// Route a message to a running agent by session ID.
    ///
    /// Uses the session cache for fast lookups, falling back to K8s API.
    pub async fn route_message(&self, session_id: &str, text: &str) -> Result<()> {
        // 1. Check cache first
        if let Some(pod_info) = self.cache.get(session_id).await {
            debug!(
                session_id = %session_id,
                pod_ip = %pod_info.pod_ip,
                "Cache hit for session"
            );
            return self.send_http(&pod_info.pod_ip, text).await;
        }

        // 2. Cache miss - look up pod via K8s API
        debug!(session_id = %session_id, "Cache miss, querying K8s API");
        let agents = find_running_agents(&self.kube_client, &self.namespace, session_id).await?;

        let agent = agents
            .first()
            .ok_or_else(|| anyhow!("No running agents found for session {session_id}"))?;

        // 3. Get pod IP
        let pod_ip = self.get_pod_ip(&agent.pod_name).await?;

        // 4. Cache the result
        let cached_info = CachedPodInfo {
            pod_ip: pod_ip.clone(),
            pod_name: agent.pod_name.clone(),
            container_name: agent.container_name.clone(),
            created_at: Instant::now(),
            agent_type: agent.agent_type.clone(),
        };
        self.cache.insert(session_id, cached_info).await;

        // 5. Send the message
        self.send_http(&pod_ip, text).await
    }

    /// Get the IP address of a pod by name.
    async fn get_pod_ip(&self, pod_name: &str) -> Result<String> {
        let pods: Api<Pod> = Api::namespaced(self.kube_client.clone(), &self.namespace);
        let pod = pods
            .get(pod_name)
            .await
            .context(format!("Failed to get pod {pod_name}"))?;

        pod.status
            .and_then(|s| s.pod_ip)
            .ok_or_else(|| anyhow!("Pod {pod_name} has no IP address"))
    }

    /// Send a message to a pod's sidecar via HTTP.
    async fn send_http(&self, pod_ip: &str, text: &str) -> Result<()> {
        let url = format!("http://{}:{}/input", pod_ip, self.sidecar_port);

        info!(
            url = %url,
            text_len = text.len(),
            "Sending message to sidecar via HTTP"
        );

        let response = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await
            .context("Failed to send HTTP request to sidecar")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Sidecar returned error: {} - {}",
                status,
                body
            ));
        }

        debug!("Message sent successfully via HTTP");
        Ok(())
    }

    /// Invalidate cache entry for a session.
    pub async fn invalidate_session(&self, session_id: &str) {
        self.cache.remove(session_id).await;
    }

    /// Evict expired cache entries.
    pub async fn evict_expired(&self) {
        self.cache.evict_expired().await;
    }

    /// Get cache statistics.
    pub async fn cache_stats(&self) -> (usize, bool) {
        (self.cache.len().await, self.cache.is_empty().await)
    }
}

// =============================================================================
// Global Router Instance (for backward compatibility)
// =============================================================================

lazy_static::lazy_static! {
    /// Global agent router instance (created on first use).
    static ref GLOBAL_ROUTER: Arc<RwLock<Option<AgentRouter>>> = Arc::new(RwLock::new(None));
}

/// Initialize the global router (call once at startup).
pub async fn init_global_router(kube_client: KubeClient, namespace: impl Into<String>) {
    let mut router = GLOBAL_ROUTER.write().await;
    *router = Some(AgentRouter::new(kube_client, namespace));
    info!("Global agent router initialized");
}

/// Send a message using the global router.
pub async fn route_message_global(session_id: &str, text: &str) -> Result<()> {
    let router = GLOBAL_ROUTER.read().await;
    let router = router
        .as_ref()
        .ok_or_else(|| anyhow!("Global router not initialized"))?;
    router.route_message(session_id, text).await
}

// =============================================================================
// Message Types
// =============================================================================

/// Message types that can be sent to a running agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentMessage {
    /// A message from the user (Linear mention/prompt).
    UserMessage {
        /// Message content from the user.
        content: String,
        /// Optional session ID for tracking.
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        /// Optional issue identifier for context.
        #[serde(skip_serializing_if = "Option::is_none")]
        issue_identifier: Option<String>,
    },
    /// A stop/cancel signal.
    Stop {
        /// Reason for stopping.
        reason: String,
    },
}

impl AgentMessage {
    /// Create a user message.
    #[must_use]
    pub fn user_message(content: impl Into<String>) -> Self {
        Self::UserMessage {
            content: content.into(),
            session_id: None,
            issue_identifier: None,
        }
    }

    /// Create a user message with context.
    #[must_use]
    pub fn user_message_with_context(
        content: impl Into<String>,
        session_id: impl Into<String>,
        issue_identifier: impl Into<String>,
    ) -> Self {
        Self::UserMessage {
            content: content.into(),
            session_id: Some(session_id.into()),
            issue_identifier: Some(issue_identifier.into()),
        }
    }

    /// Create a stop message.
    #[must_use]
    pub fn stop(reason: impl Into<String>) -> Self {
        Self::Stop {
            reason: reason.into(),
        }
    }
}

/// Information about a running agent pod.
#[derive(Debug, Clone)]
pub struct RunningAgent {
    /// Pod name.
    pub pod_name: String,
    /// Pod namespace.
    pub namespace: String,
    /// Container name (usually "main" or "agent").
    pub container_name: String,
    /// Session ID the agent is handling.
    pub session_id: String,
    /// Issue identifier (e.g., "CTOPA-21").
    pub issue_identifier: Option<String>,
    /// Agent type (intake, play, etc).
    pub agent_type: String,
    /// Pod IP address (for direct HTTP communication).
    pub pod_ip: Option<String>,
}

/// Find running agent pods for a Linear session.
///
/// Looks for pods with the `linear-session` label matching the session ID.
///
/// # Errors
/// Returns error if Kubernetes API call fails.
pub async fn find_running_agents(
    kube_client: &KubeClient,
    namespace: &str,
    session_id: &str,
) -> Result<Vec<RunningAgent>> {
    let pods: Api<Pod> = Api::namespaced(kube_client.clone(), namespace);

    // Look for pods with linear-session label
    let label_selector = format!("linear-session={session_id}");
    let lp = ListParams::default().labels(&label_selector);

    let pod_list = pods
        .list(&lp)
        .await
        .context("Failed to list pods for session")?;

    let mut agents = Vec::new();

    for pod in pod_list {
        let pod_name = pod.metadata.name.clone().unwrap_or_default();
        let labels = pod.metadata.labels.clone().unwrap_or_default();

        // Check if pod is running
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_ref())
            .map_or("Unknown", String::as_str);

        if phase != "Running" {
            debug!(pod = %pod_name, phase = %phase, "Skipping non-running pod");
            continue;
        }

        // Get pod IP for direct HTTP communication
        let pod_ip = pod
            .status
            .as_ref()
            .and_then(|s| s.pod_ip.clone());

        // Determine container name and agent type
        let container_name = determine_main_container(&pod);
        let agent_type = labels
            .get("cto.5dlabs.io/agent-type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let issue_identifier = labels.get("cto.5dlabs.io/linear-issue").cloned();

        agents.push(RunningAgent {
            pod_name,
            namespace: namespace.to_string(),
            container_name,
            session_id: session_id.to_string(),
            issue_identifier,
            agent_type,
            pod_ip,
        });
    }

    info!(
        session_id = %session_id,
        count = agents.len(),
        "Found running agents for session"
    );

    Ok(agents)
}

/// Find running agents by issue identifier.
///
/// # Errors
/// Returns error if Kubernetes API call fails.
pub async fn find_agents_by_issue(
    kube_client: &KubeClient,
    namespace: &str,
    issue_identifier: &str,
) -> Result<Vec<RunningAgent>> {
    let pods: Api<Pod> = Api::namespaced(kube_client.clone(), namespace);

    // Look for pods with linear-issue label
    let label_selector = format!("cto.5dlabs.io/linear-issue={issue_identifier}");
    let lp = ListParams::default().labels(&label_selector);

    let pod_list = pods
        .list(&lp)
        .await
        .context("Failed to list pods for issue")?;

    let mut agents = Vec::new();

    for pod in pod_list {
        let pod_name = pod.metadata.name.clone().unwrap_or_default();
        let labels = pod.metadata.labels.clone().unwrap_or_default();

        // Check if pod is running
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_ref())
            .map_or("Unknown", String::as_str);

        if phase != "Running" {
            continue;
        }

        // Get pod IP for direct HTTP communication
        let pod_ip = pod
            .status
            .as_ref()
            .and_then(|s| s.pod_ip.clone());

        let container_name = determine_main_container(&pod);
        let agent_type = labels
            .get("cto.5dlabs.io/agent-type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let session_id = labels
            .get("linear-session")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        agents.push(RunningAgent {
            pod_name,
            namespace: namespace.to_string(),
            container_name,
            session_id,
            issue_identifier: Some(issue_identifier.to_string()),
            agent_type,
            pod_ip,
        });
    }

    Ok(agents)
}

/// Determine the main container name for an agent pod.
fn determine_main_container(pod: &Pod) -> String {
    let Some(spec) = &pod.spec else {
        return "main".to_string();
    };

    // Look for specific container names in order of preference
    let preferred_names = ["agent", "main", "claude", "opencode"];

    for name in preferred_names {
        if spec.containers.iter().any(|c| c.name == name) {
            return name.to_string();
        }
    }

    // Fall back to first container
    spec.containers
        .first()
        .map_or_else(|| "main".to_string(), |c| c.name.clone())
}

/// Send a message to a running agent via HTTP to the sidecar.
///
/// This sends a POST request to the sidecar's `/input` endpoint,
/// which then writes to the FIFO for the agent to consume.
///
/// # Errors
/// Returns error if the message cannot be sent.
pub async fn send_message_to_agent(agent: &RunningAgent, message: &AgentMessage) -> Result<()> {
    info!(
        pod = %agent.pod_name,
        container = %agent.container_name,
        agent_type = %agent.agent_type,
        "Sending message to agent via HTTP"
    );

    // Extract the text content from the message
    let text = match message {
        AgentMessage::UserMessage { content, .. } => content.clone(),
        AgentMessage::Stop { reason } => format!("[STOP] {reason}"),
    };

    // If we have a pod IP, use HTTP directly
    if let Some(pod_ip) = &agent.pod_ip {
        let url = format!("http://{}:8080/input", pod_ip);
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let response = http_client
            .post(&url)
            .json(&serde_json::json!({ "text": text }))
            .send()
            .await
            .context("Failed to send HTTP request to sidecar")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Sidecar returned error: {} - {}",
                status,
                body
            ));
        }

        debug!(
            pod = %agent.pod_name,
            "Message sent successfully via HTTP"
        );
        return Ok(());
    }

    // Fallback to kubectl exec if no pod IP available
    warn!(
        pod = %agent.pod_name,
        "No pod IP available, falling back to kubectl exec"
    );

    let message_json = serde_json::to_string(message).context("Failed to serialize message")?;

    // Use kubectl exec to write to the agent input file
    let shell_script = format!(
        r#"
        # Write to the standard input FIFO location
        if [ -p /workspace/agent-input.jsonl ]; then
            echo '{}' >> /workspace/agent-input.jsonl
        elif [ -d /workspace ]; then
            echo '{}' >> /workspace/agent-input.jsonl
        else
            echo '{}' >> /tmp/agent-input.jsonl
        fi
        "#,
        message_json.replace('\'', "'\\''"),
        message_json.replace('\'', "'\\''"),
        message_json.replace('\'', "'\\''")
    );

    let output = tokio::process::Command::new("kubectl")
        .args([
            "exec",
            "-n",
            &agent.namespace,
            &agent.pod_name,
            "-c",
            &agent.container_name,
            "--",
            "sh",
            "-c",
            &shell_script,
        ])
        .output()
        .await
        .context("Failed to execute kubectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(
            pod = %agent.pod_name,
            error = %stderr,
            "Failed to send message to agent via kubectl"
        );
        return Err(anyhow!("kubectl exec failed: {stderr}"));
    }

    debug!(
        pod = %agent.pod_name,
        "Message sent successfully via kubectl"
    );

    Ok(())
}

/// Send a user message to all running agents for a session.
///
/// # Errors
/// Returns error if no agents are found or message delivery fails.
pub async fn broadcast_to_session(
    kube_client: &KubeClient,
    namespace: &str,
    session_id: &str,
    content: &str,
    issue_identifier: Option<&str>,
) -> Result<usize> {
    let agents = find_running_agents(kube_client, namespace, session_id).await?;

    if agents.is_empty() {
        return Err(anyhow!("No running agents found for session {session_id}"));
    }

    let message = AgentMessage::UserMessage {
        content: content.to_string(),
        session_id: Some(session_id.to_string()),
        issue_identifier: issue_identifier.map(String::from),
    };

    let mut sent_count = 0;
    for agent in &agents {
        match send_message_to_agent(agent, &message).await {
            Ok(()) => sent_count += 1,
            Err(e) => {
                warn!(
                    pod = %agent.pod_name,
                    error = %e,
                    "Failed to send message to agent"
                );
            }
        }
    }

    info!(
        session_id = %session_id,
        sent = sent_count,
        total = agents.len(),
        "Broadcast message to session agents"
    );

    Ok(sent_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_message_serialization() {
        let msg = AgentMessage::user_message("Hello agent!");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user_message"));
        assert!(json.contains("Hello agent!"));

        let msg = AgentMessage::user_message_with_context("Do this task", "session-123", "TSK-1");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("session-123"));
        assert!(json.contains("issue_identifier"));
        assert!(json.contains("TSK-1"));
    }

    #[test]
    fn test_stop_message() {
        let msg = AgentMessage::stop("User requested cancellation");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("stop"));
        assert!(json.contains("User requested cancellation"));
    }

    // ==========================================================================
    // Session Cache Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_session_cache_insert_and_get() {
        let cache = SessionCache::new();

        let pod_info = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "coderun-test-pod".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };

        cache.insert("session-123", pod_info.clone()).await;

        let retrieved = cache.get("session-123").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.pod_ip, "10.0.0.1");
        assert_eq!(retrieved.pod_name, "coderun-test-pod");
        assert_eq!(retrieved.agent_type, "rex");
    }

    #[tokio::test]
    async fn test_session_cache_get_nonexistent() {
        let cache = SessionCache::new();
        let result = cache.get("nonexistent-session").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_session_cache_expiration() {
        // Create cache with very short TTL (1ms)
        let cache = SessionCache::with_ttl(Duration::from_millis(1));

        let pod_info = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "test-pod".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };

        cache.insert("session-123", pod_info).await;

        // Entry should exist immediately
        assert!(cache.get("session-123").await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Entry should be expired
        assert!(cache.get("session-123").await.is_none());
    }

    #[tokio::test]
    async fn test_session_cache_remove() {
        let cache = SessionCache::new();

        let pod_info = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "test-pod".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };

        cache.insert("session-123", pod_info).await;
        assert!(cache.get("session-123").await.is_some());

        cache.remove("session-123").await;
        assert!(cache.get("session-123").await.is_none());
    }

    #[tokio::test]
    async fn test_session_cache_evict_expired() {
        let cache = SessionCache::with_ttl(Duration::from_millis(1));

        // Insert multiple entries
        for i in 0..5 {
            let pod_info = CachedPodInfo {
                pod_ip: format!("10.0.0.{i}"),
                pod_name: format!("test-pod-{i}"),
                container_name: "main".to_string(),
                created_at: Instant::now(),
                agent_type: "rex".to_string(),
            };
            cache.insert(format!("session-{i}"), pod_info).await;
        }

        assert_eq!(cache.len().await, 5);

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Evict expired entries
        cache.evict_expired().await;

        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_session_cache_update_existing() {
        let cache = SessionCache::new();

        let pod_info_v1 = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "test-pod-v1".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };

        cache.insert("session-123", pod_info_v1).await;

        // Update with new info (simulating pod restart with new IP)
        let pod_info_v2 = CachedPodInfo {
            pod_ip: "10.0.0.2".to_string(),
            pod_name: "test-pod-v2".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };

        cache.insert("session-123", pod_info_v2).await;

        let retrieved = cache.get("session-123").await.unwrap();
        assert_eq!(retrieved.pod_ip, "10.0.0.2");
        assert_eq!(retrieved.pod_name, "test-pod-v2");
    }

    // ==========================================================================
    // CachedPodInfo Tests
    // ==========================================================================

    #[test]
    fn test_cached_pod_info_expiration() {
        let ttl = Duration::from_millis(100);

        // Fresh entry should not be expired
        let fresh_info = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "test-pod".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now(),
            agent_type: "rex".to_string(),
        };
        assert!(!fresh_info.is_expired(ttl));

        // Old entry should be expired
        let old_info = CachedPodInfo {
            pod_ip: "10.0.0.1".to_string(),
            pod_name: "test-pod".to_string(),
            container_name: "main".to_string(),
            created_at: Instant::now() - Duration::from_millis(200),
            agent_type: "rex".to_string(),
        };
        assert!(old_info.is_expired(ttl));
    }

    // ==========================================================================
    // RunningAgent Tests
    // ==========================================================================

    #[test]
    fn test_running_agent_with_pod_ip() {
        let agent = RunningAgent {
            pod_name: "coderun-rex-123".to_string(),
            namespace: "cto".to_string(),
            container_name: "main".to_string(),
            session_id: "session-abc".to_string(),
            issue_identifier: Some("TSK-123".to_string()),
            agent_type: "rex".to_string(),
            pod_ip: Some("10.0.0.42".to_string()),
        };

        assert!(agent.pod_ip.is_some());
        assert_eq!(agent.pod_ip.as_deref(), Some("10.0.0.42"));
    }

    #[test]
    fn test_running_agent_without_pod_ip() {
        let agent = RunningAgent {
            pod_name: "coderun-rex-123".to_string(),
            namespace: "cto".to_string(),
            container_name: "main".to_string(),
            session_id: "session-abc".to_string(),
            issue_identifier: Some("TSK-123".to_string()),
            agent_type: "rex".to_string(),
            pod_ip: None,
        };

        assert!(agent.pod_ip.is_none());
    }

    // ==========================================================================
    // End-to-End Flow Simulation Tests
    // ==========================================================================

    /// Simulates the end-to-end flow of routing a message to an agent.
    /// This test verifies the data structures work correctly together.
    #[tokio::test]
    async fn test_e2e_message_routing_flow_simulation() {
        // 1. Simulate finding a running agent (would come from K8s API in production)
        let agent = RunningAgent {
            pod_name: "coderun-rex-task-2".to_string(),
            namespace: "cto".to_string(),
            container_name: "main".to_string(),
            session_id: "linear-session-xyz".to_string(),
            issue_identifier: Some("PROJ-42".to_string()),
            agent_type: "rex".to_string(),
            pod_ip: Some("10.244.0.15".to_string()),
        };

        // 2. Simulate caching the pod info
        let cache = SessionCache::new();
        let cached_info = CachedPodInfo {
            pod_ip: agent.pod_ip.clone().unwrap(),
            pod_name: agent.pod_name.clone(),
            container_name: agent.container_name.clone(),
            created_at: Instant::now(),
            agent_type: agent.agent_type.clone(),
        };
        cache.insert(&agent.session_id, cached_info).await;

        // 3. Verify cache hit on subsequent lookup
        let cached = cache.get(&agent.session_id).await;
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.pod_ip, "10.244.0.15");
        assert_eq!(cached.agent_type, "rex");

        // 4. Create message to send
        let message = AgentMessage::user_message_with_context(
            "Please implement the login feature",
            &agent.session_id,
            agent.issue_identifier.as_deref().unwrap(),
        );

        // 5. Verify message serialization (HTTP body)
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("Please implement the login feature"));
        assert!(json.contains("linear-session-xyz"));
        assert!(json.contains("PROJ-42"));
    }

    /// Verify the complete label set expected by the routing system.
    /// These labels must match what the controller creates.
    #[test]
    fn test_expected_pod_labels_for_routing() {
        // These are the labels the PM server expects to find on pods
        let expected_labels = vec![
            "linear-session",              // For session-based routing
            "cto.5dlabs.io/linear-issue",  // For issue-based routing
            "cto.5dlabs.io/agent-type",    // For agent identification
        ];

        // Verify label selectors are correctly formatted
        let session_id = "session-abc123";
        let issue_id = "PROJ-42";

        let session_selector = format!("linear-session={session_id}");
        assert_eq!(session_selector, "linear-session=session-abc123");

        let issue_selector = format!("cto.5dlabs.io/linear-issue={issue_id}");
        assert_eq!(issue_selector, "cto.5dlabs.io/linear-issue=PROJ-42");

        // Verify these match the expected format
        for label in expected_labels {
            assert!(!label.is_empty());
        }
    }
}
