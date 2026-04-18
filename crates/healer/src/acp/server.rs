use super::types::MonitorEventStore;
use crate::play::SessionStoreHandle;
use acp_runtime::{ensure_allowed_caller, CallerContext};
use agent_client_protocol::{
    Agent, AuthenticateRequest, AuthenticateResponse, CancelNotification, Error, ExtNotification,
    ExtRequest, ExtResponse, Implementation, InitializeRequest, InitializeResponse,
    LoadSessionRequest, LoadSessionResponse, NewSessionRequest, NewSessionResponse, PromptRequest,
    PromptResponse, ProtocolVersion, SessionId, StopReason,
};
use serde::Serialize;
use serde_json::value::to_raw_value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state exposed by Healer's internal ACP server.
#[derive(Debug, Clone)]
pub struct HealerAcpServerState {
    /// Healer play sessions.
    pub play_sessions: SessionStoreHandle,
    /// Ingested Stakpak monitor events.
    pub monitor_events: MonitorEventStore,
    /// Callers allowed to open ACP sessions.
    pub allowed_callers: Vec<String>,
    /// ACP sessions currently opened through this server.
    active_sessions: Arc<RwLock<HashSet<String>>>,
}

impl HealerAcpServerState {
    /// Create a new server state.
    #[must_use]
    pub fn new(
        play_sessions: SessionStoreHandle,
        monitor_events: MonitorEventStore,
        allowed_callers: Vec<String>,
    ) -> Self {
        Self {
            play_sessions,
            monitor_events,
            allowed_callers,
            active_sessions: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

/// ACP agent surface for `OpenClaw` to query Healer state.
#[derive(Debug, Clone)]
pub struct HealerAcpAgent {
    state: HealerAcpServerState,
}

impl HealerAcpAgent {
    /// Create a new ACP agent.
    #[must_use]
    pub fn new(state: HealerAcpServerState) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait(?Send)]
impl Agent for HealerAcpAgent {
    async fn initialize(
        &self,
        arguments: InitializeRequest,
    ) -> agent_client_protocol::Result<InitializeResponse> {
        Ok(
            InitializeResponse::new(arguments.protocol_version.min(ProtocolVersion::LATEST))
                .agent_info(
                    Implementation::new("cto-healer", env!("CARGO_PKG_VERSION"))
                        .title("CTO Healer"),
                ),
        )
    }

    async fn authenticate(
        &self,
        _arguments: AuthenticateRequest,
    ) -> agent_client_protocol::Result<AuthenticateResponse> {
        Ok(AuthenticateResponse::default())
    }

    async fn new_session(
        &self,
        arguments: NewSessionRequest,
    ) -> agent_client_protocol::Result<NewSessionResponse> {
        let _: CallerContext =
            ensure_allowed_caller(arguments.meta.as_ref(), &self.state.allowed_callers)?;
        let session_id = SessionId::new(format!("healer-{}", uuid::Uuid::new_v4().simple()));
        self.state
            .active_sessions
            .write()
            .await
            .insert(session_id.to_string());
        Ok(NewSessionResponse::new(session_id))
    }

    async fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> agent_client_protocol::Result<LoadSessionResponse> {
        let _: CallerContext =
            ensure_allowed_caller(arguments.meta.as_ref(), &self.state.allowed_callers)?;
        if self
            .state
            .active_sessions
            .read()
            .await
            .contains(arguments.session_id.0.as_ref())
        {
            Ok(LoadSessionResponse::new())
        } else {
            Err(Error::invalid_params().data("unknown healer ACP session"))
        }
    }

    async fn prompt(
        &self,
        arguments: PromptRequest,
    ) -> agent_client_protocol::Result<PromptResponse> {
        if self
            .state
            .active_sessions
            .read()
            .await
            .contains(arguments.session_id.0.as_ref())
        {
            Ok(PromptResponse::new(StopReason::EndTurn))
        } else {
            Err(Error::invalid_params().data("unknown healer ACP session"))
        }
    }

    async fn cancel(&self, arguments: CancelNotification) -> agent_client_protocol::Result<()> {
        self.state
            .active_sessions
            .write()
            .await
            .remove(arguments.session_id.0.as_ref());
        Ok(())
    }

    async fn ext_method(&self, args: ExtRequest) -> agent_client_protocol::Result<ExtResponse> {
        match args.method.as_ref() {
            "5dlabs.healer/status" => {
                #[derive(Serialize)]
                struct HealerStatus {
                    active_sessions: usize,
                    play_sessions: usize,
                    monitor_events: usize,
                }

                let response = HealerStatus {
                    active_sessions: self.state.active_sessions.read().await.len(),
                    play_sessions: self.state.play_sessions.get_all_sessions().await.len(),
                    monitor_events: self.state.monitor_events.len().await,
                };
                Ok(ExtResponse::new(to_raw_value(&response)?.into()))
            }
            "5dlabs.healer/play-sessions" => {
                let sessions = self.state.play_sessions.get_all_sessions().await;
                Ok(ExtResponse::new(to_raw_value(&sessions)?.into()))
            }
            "5dlabs.healer/monitor-events" => {
                let events = self.state.monitor_events.list().await;
                Ok(ExtResponse::new(to_raw_value(&events)?.into()))
            }
            _ => Err(Error::method_not_found()),
        }
    }

    async fn ext_notification(&self, _args: ExtNotification) -> agent_client_protocol::Result<()> {
        Ok(())
    }
}
