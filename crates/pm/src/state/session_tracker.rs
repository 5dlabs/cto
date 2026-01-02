//! Session tracking for Linear agent sessions.
//!
//! Maps Linear session IDs to running Argo workflows and agent pods.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Information about a tracked session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Linear session ID.
    pub session_id: String,
    /// Agent name (e.g., "rex", "morgan").
    pub agent_name: String,
    /// Argo workflow name.
    pub workflow_name: Option<String>,
    /// Pod name running the agent.
    pub pod_name: Option<String>,
    /// Pod IP for direct communication.
    pub pod_ip: Option<String>,
    /// Linear issue ID.
    pub issue_id: String,
    /// Linear issue identifier (e.g., "TSK-123").
    pub issue_identifier: String,
    /// When the session was created.
    pub created_at: Instant,
    /// When the session was last updated.
    pub updated_at: Instant,
    /// Session status.
    pub status: SessionStatus,
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session created, waiting for workflow to start.
    Pending,
    /// Workflow started, agent running.
    Running,
    /// Agent completed successfully.
    Completed,
    /// Agent failed or was stopped.
    Failed,
    /// Session timed out.
    TimedOut,
}

impl SessionInfo {
    /// Create a new session info.
    #[must_use]
    pub fn new(
        session_id: String,
        agent_name: String,
        issue_id: String,
        issue_identifier: String,
    ) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            agent_name,
            workflow_name: None,
            pod_name: None,
            pod_ip: None,
            issue_id,
            issue_identifier,
            created_at: now,
            updated_at: now,
            status: SessionStatus::Pending,
        }
    }

    /// Update the workflow information.
    pub fn set_workflow(&mut self, workflow_name: String) {
        self.workflow_name = Some(workflow_name);
        self.updated_at = Instant::now();
    }

    /// Update the pod information.
    pub fn set_pod(&mut self, pod_name: String, pod_ip: Option<String>) {
        self.pod_name = Some(pod_name);
        self.pod_ip = pod_ip;
        self.status = SessionStatus::Running;
        self.updated_at = Instant::now();
    }

    /// Mark the session as completed.
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.updated_at = Instant::now();
    }

    /// Mark the session as failed.
    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
        self.updated_at = Instant::now();
    }

    /// Check if the session is still active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self.status, SessionStatus::Pending | SessionStatus::Running)
    }

    /// Get the session age.
    #[must_use]
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Thread-safe session tracker.
#[derive(Debug, Clone)]
pub struct SessionTracker {
    /// Map of session ID to session info.
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    /// Map of issue ID to session IDs (for reverse lookup).
    issue_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Map of workflow name to session ID.
    workflow_sessions: Arc<RwLock<HashMap<String, String>>>,
    /// Session timeout duration.
    timeout: Duration,
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionTracker {
    /// Create a new session tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            issue_sessions: Arc::new(RwLock::new(HashMap::new())),
            workflow_sessions: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::from_secs(3600), // 1 hour default timeout
        }
    }

    /// Create a new session tracker with custom timeout.
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::new()
        }
    }

    /// Register a new session.
    pub async fn register(
        &self,
        session_id: String,
        agent_name: String,
        issue_id: String,
        issue_identifier: String,
    ) {
        let info = SessionInfo::new(
            session_id.clone(),
            agent_name,
            issue_id.clone(),
            issue_identifier,
        );

        debug!(
            session_id = %session_id,
            issue_id = %issue_id,
            "Registering new session"
        );

        // Add to sessions map
        self.sessions.write().await.insert(session_id.clone(), info);

        // Add to issue lookup
        self.issue_sessions
            .write()
            .await
            .entry(issue_id)
            .or_default()
            .push(session_id);
    }

    /// Update workflow information for a session.
    pub async fn set_workflow(&self, session_id: &str, workflow_name: String) {
        if let Some(info) = self.sessions.write().await.get_mut(session_id) {
            info.set_workflow(workflow_name.clone());

            // Add to workflow lookup
            self.workflow_sessions
                .write()
                .await
                .insert(workflow_name, session_id.to_string());
        }
    }

    /// Update pod information for a session.
    pub async fn set_pod(&self, session_id: &str, pod_name: String, pod_ip: Option<String>) {
        if let Some(info) = self.sessions.write().await.get_mut(session_id) {
            info.set_pod(pod_name, pod_ip);
        }
    }

    /// Mark a session as completed.
    pub async fn complete(&self, session_id: &str) {
        if let Some(info) = self.sessions.write().await.get_mut(session_id) {
            info.complete();
            info!(session_id = %session_id, "Session completed");
        }
    }

    /// Mark a session as failed.
    pub async fn fail(&self, session_id: &str) {
        if let Some(info) = self.sessions.write().await.get_mut(session_id) {
            info.fail();
            info!(session_id = %session_id, "Session failed");
        }
    }

    /// Get session info by session ID.
    pub async fn get(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Get session info by workflow name.
    pub async fn get_by_workflow(&self, workflow_name: &str) -> Option<SessionInfo> {
        let session_id = self
            .workflow_sessions
            .read()
            .await
            .get(workflow_name)?
            .clone();
        self.get(&session_id).await
    }

    /// Get all sessions for an issue.
    pub async fn get_by_issue(&self, issue_id: &str) -> Vec<SessionInfo> {
        let session_ids = self
            .issue_sessions
            .read()
            .await
            .get(issue_id)
            .cloned()
            .unwrap_or_default();

        let sessions = self.sessions.read().await;
        session_ids
            .iter()
            .filter_map(|id| sessions.get(id).cloned())
            .collect()
    }

    /// Get all active sessions for an issue.
    pub async fn get_active_by_issue(&self, issue_id: &str) -> Vec<SessionInfo> {
        self.get_by_issue(issue_id)
            .await
            .into_iter()
            .filter(SessionInfo::is_active)
            .collect()
    }

    /// Get the pod IP for a session.
    pub async fn get_pod_ip(&self, session_id: &str) -> Option<String> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .and_then(|info| info.pod_ip.clone())
    }

    /// List all active sessions.
    pub async fn list_active(&self) -> Vec<SessionInfo> {
        self.sessions
            .read()
            .await
            .values()
            .filter(|info| info.is_active())
            .cloned()
            .collect()
    }

    /// Clean up expired sessions.
    pub async fn cleanup_expired(&self) {
        let expired: Vec<String> = {
            let sessions = self.sessions.read().await;
            sessions
                .iter()
                .filter(|(_, info)| info.age() > self.timeout && !info.is_active())
                .map(|(id, _)| id.clone())
                .collect()
        };

        if expired.is_empty() {
            return;
        }

        let mut sessions = self.sessions.write().await;
        let mut issue_sessions = self.issue_sessions.write().await;
        let mut workflow_sessions = self.workflow_sessions.write().await;

        for session_id in &expired {
            if let Some(info) = sessions.remove(session_id) {
                // Clean up issue lookup
                if let Some(ids) = issue_sessions.get_mut(&info.issue_id) {
                    ids.retain(|id| id != session_id);
                }

                // Clean up workflow lookup
                if let Some(workflow) = &info.workflow_name {
                    workflow_sessions.remove(workflow);
                }

                debug!(session_id = %session_id, "Cleaned up expired session");
            }
        }
    }

    /// Get session statistics.
    pub async fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        SessionStats {
            total: sessions.len(),
            pending: sessions
                .values()
                .filter(|s| s.status == SessionStatus::Pending)
                .count(),
            running: sessions
                .values()
                .filter(|s| s.status == SessionStatus::Running)
                .count(),
            completed: sessions
                .values()
                .filter(|s| s.status == SessionStatus::Completed)
                .count(),
            failed: sessions
                .values()
                .filter(|s| s.status == SessionStatus::Failed)
                .count(),
        }
    }
}

/// Session statistics.
#[derive(Debug, Clone, Copy)]
pub struct SessionStats {
    /// Total sessions tracked.
    pub total: usize,
    /// Sessions in pending status.
    pub pending: usize,
    /// Sessions in running status.
    pub running: usize,
    /// Sessions in completed status.
    pub completed: usize,
    /// Sessions in failed status.
    pub failed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "session-1".to_string(),
                "rex".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;

        let info = tracker.get("session-1").await.unwrap();
        assert_eq!(info.session_id, "session-1");
        assert_eq!(info.agent_name, "rex");
        assert_eq!(info.issue_id, "issue-1");
        assert_eq!(info.status, SessionStatus::Pending);
    }

    #[tokio::test]
    async fn test_set_workflow_and_pod() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "session-1".to_string(),
                "rex".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;

        tracker
            .set_workflow("session-1", "workflow-1".to_string())
            .await;
        tracker
            .set_pod(
                "session-1",
                "pod-1".to_string(),
                Some("10.0.0.1".to_string()),
            )
            .await;

        let info = tracker.get("session-1").await.unwrap();
        assert_eq!(info.workflow_name, Some("workflow-1".to_string()));
        assert_eq!(info.pod_name, Some("pod-1".to_string()));
        assert_eq!(info.pod_ip, Some("10.0.0.1".to_string()));
        assert_eq!(info.status, SessionStatus::Running);
    }

    #[tokio::test]
    async fn test_get_by_workflow() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "session-1".to_string(),
                "rex".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;
        tracker
            .set_workflow("session-1", "workflow-1".to_string())
            .await;

        let info = tracker.get_by_workflow("workflow-1").await.unwrap();
        assert_eq!(info.session_id, "session-1");
    }

    #[tokio::test]
    async fn test_get_by_issue() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "session-1".to_string(),
                "rex".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;
        tracker
            .register(
                "session-2".to_string(),
                "blaze".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;

        let sessions = tracker.get_by_issue("issue-1").await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_complete_and_fail() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "session-1".to_string(),
                "rex".to_string(),
                "issue-1".to_string(),
                "TSK-1".to_string(),
            )
            .await;

        tracker.complete("session-1").await;
        let info = tracker.get("session-1").await.unwrap();
        assert_eq!(info.status, SessionStatus::Completed);
        assert!(!info.is_active());
    }

    #[tokio::test]
    async fn test_stats() {
        let tracker = SessionTracker::new();
        tracker
            .register(
                "s1".to_string(),
                "rex".to_string(),
                "i1".to_string(),
                "T1".to_string(),
            )
            .await;
        tracker
            .register(
                "s2".to_string(),
                "blaze".to_string(),
                "i2".to_string(),
                "T2".to_string(),
            )
            .await;
        tracker.set_pod("s1", "p1".to_string(), None).await;
        tracker.complete("s2").await;

        let stats = tracker.stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.running, 1);
        assert_eq!(stats.completed, 1);
    }
}
