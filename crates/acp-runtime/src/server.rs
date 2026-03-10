use agent_client_protocol::{Agent, AgentSideConnection, Error, Meta, Result};
use serde::{Deserialize, Serialize};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::warn;

/// Internal caller information extracted from ACP metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CallerContext {
    /// Stable caller identifier, for example `openclaw`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "callerId")]
    pub caller_id: Option<String>,

    /// Optional bearer token mirrored into ACP metadata.
    #[serde(skip_serializing_if = "Option::is_none", rename = "bearerToken")]
    pub bearer_token: Option<String>,
}

/// Extract caller metadata from an ACP request `_meta` field.
#[must_use]
pub fn caller_from_meta(meta: Option<&Meta>) -> CallerContext {
    let caller_id = meta
        .and_then(|meta| meta.get("caller").or_else(|| meta.get("callerId")))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let bearer_token = meta
        .and_then(|meta| meta.get("token").or_else(|| meta.get("bearerToken")))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);

    CallerContext {
        caller_id,
        bearer_token,
    }
}

/// Enforce a caller allowlist using ACP metadata.
///
/// Services can call this from `new_session`/`load_session` handlers to keep
/// internal-only ACP servers scoped to known callers such as OpenClaw.
pub fn ensure_allowed_caller(
    meta: Option<&Meta>,
    allowed_callers: &[String],
) -> Result<CallerContext> {
    let caller = caller_from_meta(meta);

    if allowed_callers.is_empty() {
        return Ok(caller);
    }

    if caller
        .caller_id
        .as_ref()
        .is_some_and(|caller_id| allowed_callers.iter().any(|allowed| allowed == caller_id))
    {
        Ok(caller)
    } else {
        Err(Error::auth_required().data(format!(
            "caller must be one of: {}",
            allowed_callers.join(", ")
        )))
    }
}

/// Serve an ACP agent over stdio using the upstream ACP transport.
///
/// This is the standard hosting mode for harness-managed ACP agents such as
/// OpenClaw-spawned helpers.
pub async fn serve_stdio_agent(agent: impl Agent + 'static) -> anyhow::Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            let (_connection, io_task) =
                AgentSideConnection::new(agent, stdout.compat_write(), stdin.compat(), |future| {
                    tokio::task::spawn_local(future);
                });

            if let Err(error) = io_task.await {
                warn!(error = %error, "ACP agent stdio task exited with error");
                return Err(anyhow::Error::new(error));
            }

            Ok(())
        })
        .await
}
