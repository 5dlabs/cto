use super::types::HealerAcpSessionRecord;
use acp_runtime::{
    run_oneshot_prompt, AcpClientProfile, AcpPromptRequest, AcpPromptResult, AcpRunState,
    AcpRuntimeRegistry, AcpSessionMetadata,
};
use agent_client_protocol::StopReason;
use anyhow::{anyhow, Result};
use cto_config::AcpDefaults;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ACP client wrapper used by Healer for investigation/remediation prompts.
#[derive(Debug, Clone)]
pub struct HealerAcpClient {
    registry: AcpRuntimeRegistry,
    sessions: Arc<RwLock<HashMap<String, HealerAcpSessionRecord>>>,
}

impl HealerAcpClient {
    /// Create a new Healer ACP client from shared ACP defaults.
    #[must_use]
    pub fn new(acp: AcpDefaults) -> Self {
        Self {
            registry: AcpRuntimeRegistry::new(acp),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Borrow the registry.
    #[must_use]
    pub fn registry(&self) -> &AcpRuntimeRegistry {
        &self.registry
    }

    /// Resolve the current session record for a Healer key.
    pub async fn session(&self, key: &str) -> Option<HealerAcpSessionRecord> {
        self.sessions.read().await.get(key).cloned()
    }

    /// Execute a one-shot ACP prompt against the configured runtime for Healer.
    ///
    /// # Errors
    /// Returns an error if no ACP runtime is enabled for healer or the prompt run fails.
    pub async fn investigate(
        &self,
        key: impl Into<String>,
        issue_id: Option<String>,
        prompt: impl Into<String>,
        cwd: impl Into<PathBuf>,
        requested_runtime: Option<&str>,
    ) -> Result<AcpPromptResult> {
        let key = key.into();
        let selection = self
            .registry
            .select_runtime_for_service("healer", requested_runtime)
            .ok_or_else(|| anyhow!("no ACP runtime enabled for healer"))?;

        let existing = self.session(&key).await;
        let previous_session = existing
            .as_ref()
            .map(|record| record.session.clone())
            .unwrap_or_default();
        self.sessions.write().await.insert(
            key.clone(),
            HealerAcpSessionRecord {
                key: key.clone(),
                issue_id: issue_id.clone(),
                session: AcpSessionMetadata {
                    runtime_id: Some(selection.runtime_id.clone()),
                    run_state: AcpRunState::Pending,
                    ..previous_session.clone()
                },
            },
        );

        let result = run_oneshot_prompt(
            &selection.runtime,
            AcpPromptRequest {
                runtime_id: selection.runtime_id.clone(),
                cwd: cwd.into(),
                prompt: prompt.into(),
                session_id: previous_session.session_id.clone(),
            },
            AcpClientProfile {
                permission_policy: acp_runtime::AcpPermissionPolicy::DenyAll,
                ..AcpClientProfile::default()
            },
        )
        .await?;

        let run_state = match result.stop_reason {
            StopReason::EndTurn => AcpRunState::Completed,
            StopReason::Cancelled => AcpRunState::Cancelled,
            StopReason::MaxTokens | StopReason::MaxTurnRequests => AcpRunState::Running,
            _ => AcpRunState::Failed,
        };

        self.sessions.write().await.insert(
            key.clone(),
            HealerAcpSessionRecord {
                key,
                issue_id,
                session: AcpSessionMetadata {
                    runtime_id: Some(result.runtime_id.clone()),
                    session_id: Some(result.session_id.clone()),
                    run_state,
                    last_event_cursor: result
                        .notifications
                        .len()
                        .checked_sub(1)
                        .map(|idx| idx.to_string()),
                },
            },
        );

        Ok(result)
    }

    /// Mark a tracked session as cancelled.
    pub async fn cancel(&self, key: &str) {
        if let Some(record) = self.sessions.write().await.get_mut(key) {
            record.session.run_state = AcpRunState::Cancelled;
        }
    }
}
