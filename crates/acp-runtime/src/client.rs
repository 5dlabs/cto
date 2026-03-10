use crate::types::{AcpImplementationInfo, AcpPermissionPolicy, AcpPromptRequest, AcpPromptResult};
use agent_client_protocol::{
    Agent, CancelNotification, Client, ClientCapabilities, ClientSideConnection, ContentBlock,
    FileSystemCapabilities, Implementation, InitializeRequest, LoadSessionRequest,
    NewSessionRequest, PermissionOptionKind, PromptRequest, ProtocolVersion,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionNotification, TextContent,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cto_config::{AcpRuntimeConfig, AcpTransport};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::warn;

/// Client identity and permission behavior for ACP runtime calls.
#[derive(Debug, Clone)]
pub struct AcpClientProfile {
    /// Client implementation name reported to the runtime.
    pub name: String,
    /// Human-readable title.
    pub title: String,
    /// Version string.
    pub version: String,
    /// Permission policy applied to runtime approval requests.
    pub permission_policy: AcpPermissionPolicy,
}

impl Default for AcpClientProfile {
    fn default() -> Self {
        Self {
            name: "cto-acp-runtime".to_string(),
            title: "CTO ACP Runtime".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            permission_policy: AcpPermissionPolicy::DenyAll,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeClient {
    permission_policy: AcpPermissionPolicy,
    notifications: Arc<Mutex<Vec<SessionNotification>>>,
}

impl RuntimeClient {
    fn new(permission_policy: AcpPermissionPolicy) -> Self {
        Self {
            permission_policy,
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn take_notifications(&self) -> Vec<SessionNotification> {
        std::mem::take(
            &mut *self
                .notifications
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}

#[async_trait(?Send)]
impl Client for RuntimeClient {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> agent_client_protocol::Result<RequestPermissionResponse> {
        let outcome = match self.permission_policy {
            AcpPermissionPolicy::AllowAll => args
                .options
                .iter()
                .find(|option| {
                    matches!(
                        option.kind,
                        PermissionOptionKind::AllowOnce | PermissionOptionKind::AllowAlways
                    )
                })
                .map(|option| {
                    RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                        option.option_id.clone(),
                    ))
                })
                .unwrap_or(RequestPermissionOutcome::Cancelled),
            AcpPermissionPolicy::DenyAll => RequestPermissionOutcome::Cancelled,
        };

        Ok(RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(
        &self,
        args: SessionNotification,
    ) -> agent_client_protocol::Result<()> {
        self.notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(args);
        Ok(())
    }
}

/// Execute a one-shot ACP prompt against a stdio runtime such as `stakpak acp`.
///
/// The runtime process is started, initialized, prompted, and then terminated
/// once the prompt completes. The returned session ID can be reused by callers
/// if the underlying runtime supports later `load_session` operations.
pub async fn run_oneshot_prompt(
    runtime: &AcpRuntimeConfig,
    request: AcpPromptRequest,
    profile: AcpClientProfile,
) -> Result<AcpPromptResult> {
    if runtime.transport != AcpTransport::Stdio {
        return Err(anyhow!("only stdio ACP runtimes are supported"));
    }

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let mut command = Command::new(&runtime.command);
            command.args(&runtime.args);
            command.stdin(Stdio::piped());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::inherit());

            if let Some(cwd) = runtime.cwd.as_deref() {
                command.current_dir(cwd);
            } else {
                command.current_dir(&request.cwd);
            }

            for (key, value) in &runtime.env {
                command.env(key, value);
            }

            let mut child = command
                .spawn()
                .with_context(|| format!("failed to spawn ACP runtime {}", runtime.command))?;

            let stdin = child
                .stdin
                .take()
                .context("ACP runtime did not expose stdin")?;
            let stdout = child
                .stdout
                .take()
                .context("ACP runtime did not expose stdout")?;

            let runtime_client = RuntimeClient::new(profile.permission_policy);
            let (connection, io_task) = ClientSideConnection::new(
                runtime_client.clone(),
                stdin.compat_write(),
                stdout.compat(),
                |future| {
                    tokio::task::spawn_local(future);
                },
            );

            tokio::task::spawn_local(async move {
                if let Err(error) = io_task.await {
                    warn!(error = %error, "ACP runtime IO task exited with error");
                }
            });

            let initialize = InitializeRequest::new(ProtocolVersion::LATEST)
                .client_capabilities(
                    ClientCapabilities::new()
                        .fs(FileSystemCapabilities::new())
                        .terminal(false),
                )
                .client_info(
                    Implementation::new(profile.name, profile.version).title(profile.title),
                );
            let initialize_response = connection
                .initialize(initialize)
                .await
                .context("failed to initialize ACP runtime")?;

            let session_id = if let Some(session_id) = request.session_id.as_ref() {
                connection
                    .load_session(LoadSessionRequest::new(
                        session_id.clone(),
                        request.cwd.clone(),
                    ))
                    .await
                    .context("failed to load ACP session")?;
                session_id.clone()
            } else {
                connection
                    .new_session(NewSessionRequest::new(request.cwd.clone()))
                    .await
                    .context("failed to create ACP session")?
                    .session_id
                    .to_string()
            };

            let prompt = PromptRequest::new(
                session_id.clone(),
                vec![ContentBlock::Text(TextContent::new(request.prompt))],
            );
            let prompt_response = connection
                .prompt(prompt)
                .await
                .context("failed to execute ACP prompt")?;

            let _ = connection
                .cancel(CancelNotification::new(session_id.clone()))
                .await;
            let _ = child.start_kill();
            let _ = child.wait().await;

            Ok(AcpPromptResult {
                runtime_id: request.runtime_id,
                session_id,
                agent_info: initialize_response
                    .agent_info
                    .as_ref()
                    .map(AcpImplementationInfo::from),
                stop_reason: prompt_response.stop_reason,
                notifications: runtime_client.take_notifications(),
            })
        })
        .await
}
