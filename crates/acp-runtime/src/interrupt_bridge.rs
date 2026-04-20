//! File → ACP interrupt bridge.
//!
//! A narrator sidecar running alongside the agent container writes interrupt
//! events as JSON lines to a shared file (default
//! `/workspace/.narrator/interrupt.jsonl`). Since sidecars cannot write to the
//! agent process's stdin, this bridge tails the file and forwards each line as
//! an ACP interrupt: `session/cancel` on the current turn followed by a new
//! `session/prompt` with the user's text.
//!
//! # Architecture note (Phase C)
//!
//! The original design sketch passed an `AgentSideConnection` to this bridge.
//! In ACP v0.10.0, however, `AgentSideConnection` implements `Client` only
//! (it receives cancel/prompt, it does not send them). `cancel` and `prompt`
//! are `Agent`-trait methods, and `Agent` is implemented by
//! `ClientSideConnection` — i.e. by the process that spawned the agent. The
//! bridge therefore runs on the **client** side of the ACP link (inside the
//! same wrapper binary that launches the agent subprocess).
//!
//! To keep Phase D (controller wiring) decoupled from Phase C, the public API
//! takes an `AcpInterruptSink` trait object. A blanket impl is provided for
//! `ClientSideConnection`; other transports can supply their own adapter.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::{
    Agent, CancelNotification, ClientSideConnection, ContentBlock, PromptRequest, TextContent,
};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Default location of the interrupt log on the shared workspace PV.
pub const DEFAULT_INTERRUPT_PATH: &str = "/workspace/.narrator/interrupt.jsonl";

/// Polling interval used by the file tailer when the `notify` crate is not
/// available as a workspace dependency.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// A single JSON line written by the narrator sidecar.
#[derive(Debug, Clone, Deserialize)]
pub struct InterruptEvent {
    /// ACP session the interrupt targets.
    pub session_id: String,
    /// Source of the interrupt (`"voice"` or `"text"`). Not acted on today.
    #[serde(default)]
    pub source: Option<String>,
    /// The user-provided text that should become the new prompt.
    pub text: String,
    /// Optional RFC3339 timestamp captured by the sidecar.
    #[serde(default)]
    pub ts: Option<String>,
}

/// Abstraction over an ACP client capable of sending cancel + prompt on a
/// session. Implemented for `ClientSideConnection`; a test double can be
/// supplied by implementing this trait on any `Send + Sync` handle.
#[async_trait(?Send)]
pub trait AcpInterruptSink: 'static {
    /// Cancel the currently running turn on `session_id`.
    async fn cancel_session(&self, session_id: &str) -> anyhow::Result<()>;

    /// Start a new prompt turn on `session_id` with the given text content.
    async fn prompt_session(&self, session_id: &str, text: &str) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
impl AcpInterruptSink for ClientSideConnection {
    async fn cancel_session(&self, session_id: &str) -> anyhow::Result<()> {
        let notification = CancelNotification::new(session_id.to_string());
        self.cancel(notification)
            .await
            .map_err(|err| anyhow::anyhow!("ACP cancel failed: {err}"))
    }

    async fn prompt_session(&self, session_id: &str, text: &str) -> anyhow::Result<()> {
        let request = PromptRequest::new(
            session_id.to_string(),
            vec![ContentBlock::Text(TextContent::new(text.to_string()))],
        );
        self.prompt(request)
            .await
            .map(|_| ())
            .map_err(|err| anyhow::anyhow!("ACP prompt failed: {err}"))
    }
}

/// Spawn a task that tails `path` and forwards each JSON line as an ACP
/// interrupt (cancel + new prompt) via `sink`.
///
/// The task:
/// * creates the parent directory if missing,
/// * seeks to end-of-file on startup (pre-existing lines are ignored — the
///   sidecar is per-session and pre-existing content would be stale),
/// * polls for new bytes every 100 ms and processes each complete newline
///   terminated record,
/// * logs errors via `tracing` without aborting the task.
///
/// The task stops when the returned `JoinHandle` is aborted/dropped. It must
/// be invoked from inside a `tokio::task::LocalSet`, because `ClientSideConnection`
/// is `!Send`.
///
/// # Errors
///
/// Returns an error only for setup failures (e.g. the parent directory could
/// not be created). Runtime tailing errors are logged and the task keeps
/// running.
pub fn spawn_interrupt_bridge<S>(
    sink: Arc<S>,
    path: impl Into<PathBuf>,
) -> anyhow::Result<JoinHandle<()>>
where
    S: AcpInterruptSink,
{
    let path = path.into();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| {
                anyhow::anyhow!(
                    "failed to create interrupt bridge parent dir {}: {err}",
                    parent.display()
                )
            })?;
        }
    }

    info!(
        path = %path.display(),
        "spawning ACP interrupt bridge (poll = {POLL_INTERVAL:?})"
    );

    let handle = tokio::task::spawn_local(async move {
        if let Err(err) = tail_loop(sink.as_ref(), &path).await {
            warn!(error = %err, path = %path.display(), "ACP interrupt bridge terminated");
        }
    });

    Ok(handle)
}

async fn tail_loop<S: AcpInterruptSink>(sink: &S, path: &Path) -> anyhow::Result<()> {
    let mut offset: u64 = 0;
    let mut pending = String::new();
    let mut file: Option<File> = None;
    let mut ticker = tokio::time::interval(POLL_INTERVAL);

    loop {
        ticker.tick().await;

        if file.is_none() {
            match OpenOptions::new().read(true).open(path).await {
                Ok(mut opened) => {
                    let end = opened.seek(SeekFrom::End(0)).await.unwrap_or(0);
                    offset = end;
                    debug!(
                        path = %path.display(),
                        start_offset = offset,
                        "interrupt log opened; tailing from end"
                    );
                    file = Some(opened);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    continue;
                }
                Err(err) => {
                    warn!(error = %err, path = %path.display(), "failed to open interrupt log");
                    continue;
                }
            }
        }

        let Some(active) = file.as_mut() else {
            continue;
        };

        let metadata = match active.metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                warn!(error = %err, "failed to stat interrupt log; reopening");
                file = None;
                pending.clear();
                continue;
            }
        };

        if metadata.len() < offset {
            debug!(
                new_len = metadata.len(),
                old_offset = offset,
                "interrupt log truncated; resetting tail"
            );
            offset = 0;
            pending.clear();
            if let Err(err) = active.seek(SeekFrom::Start(0)).await {
                warn!(error = %err, "failed to rewind interrupt log");
                file = None;
                continue;
            }
        }

        if metadata.len() == offset {
            continue;
        }

        let mut buf = Vec::new();
        match active.read_to_end(&mut buf).await {
            Ok(0) => continue,
            Ok(n) => {
                offset += n as u64;
            }
            Err(err) => {
                warn!(error = %err, "failed to read interrupt log; reopening");
                file = None;
                pending.clear();
                continue;
            }
        }

        match std::str::from_utf8(&buf) {
            Ok(chunk) => pending.push_str(chunk),
            Err(err) => {
                warn!(error = %err, "interrupt log contained invalid UTF-8; dropping chunk");
                pending.clear();
                continue;
            }
        }

        while let Some(newline) = pending.find('\n') {
            let line = pending[..newline].trim_end_matches('\r').to_string();
            pending.drain(..=newline);
            if line.trim().is_empty() {
                continue;
            }
            handle_line(sink, &line).await;
        }
    }
}

async fn handle_line<S: AcpInterruptSink>(sink: &S, line: &str) {
    let event: InterruptEvent = match serde_json::from_str(line) {
        Ok(event) => event,
        Err(err) => {
            warn!(error = %err, line, "malformed interrupt event; skipping");
            return;
        }
    };

    if event.session_id.is_empty() || event.text.is_empty() {
        warn!(
            ?event,
            "interrupt event missing session_id or text; skipping"
        );
        return;
    }

    info!(
        session_id = %event.session_id,
        source = event.source.as_deref().unwrap_or("unknown"),
        "forwarding interrupt: cancel + prompt"
    );

    if let Err(err) = sink.cancel_session(&event.session_id).await {
        warn!(
            error = %err,
            session_id = %event.session_id,
            "ACP cancel failed; continuing with prompt"
        );
    }

    if let Err(err) = sink.prompt_session(&event.session_id, &event.text).await {
        warn!(
            error = %err,
            session_id = %event.session_id,
            "ACP prompt failed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::io::AsyncWriteExt;

    #[derive(Default)]
    struct RecordingSink {
        calls: Mutex<Vec<String>>,
    }

    #[async_trait(?Send)]
    impl AcpInterruptSink for RecordingSink {
        async fn cancel_session(&self, session_id: &str) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("cancel:{session_id}"));
            Ok(())
        }

        async fn prompt_session(&self, session_id: &str, text: &str) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("prompt:{session_id}:{text}"));
            Ok(())
        }
    }

    async fn drive_once<S: AcpInterruptSink>(sink: &S, line: &str) {
        handle_line(sink, line).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handle_line_calls_cancel_then_prompt_in_order() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let sink = RecordingSink::default();
                let line = r#"{"session_id":"sess-1","source":"voice","text":"stop and start over","ts":"2025-01-01T00:00:00Z"}"#;
                drive_once(&sink, line).await;
                let calls = sink.calls.lock().unwrap().clone();
                assert_eq!(
                    calls,
                    vec![
                        "cancel:sess-1".to_string(),
                        "prompt:sess-1:stop and start over".to_string(),
                    ]
                );
            })
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handle_line_skips_malformed_json() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let sink = RecordingSink::default();
                drive_once(&sink, "this is not json").await;
                drive_once(&sink, r#"{"session_id":"","text":"x"}"#).await;
                drive_once(&sink, r#"{"session_id":"a","text":""}"#).await;
                assert!(sink.calls.lock().unwrap().is_empty());
            })
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn bridge_forwards_lines_appended_after_startup() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let tmp = tempfile::tempdir().unwrap();
                let path = tmp.path().join("interrupt.jsonl");
                // Pre-existing content that must be ignored.
                tokio::fs::write(&path, "{\"session_id\":\"old\",\"text\":\"ignored\"}\n")
                    .await
                    .unwrap();

                let sink = Arc::new(RecordingSink::default());
                let handle = spawn_interrupt_bridge(sink.clone(), path.clone()).unwrap();

                // Give the tailer a beat to seek to end before we append.
                tokio::time::sleep(Duration::from_millis(250)).await;

                let mut writer = OpenOptions::new().append(true).open(&path).await.unwrap();
                writer
                    .write_all(
                        b"{\"session_id\":\"sess-2\",\"source\":\"text\",\"text\":\"hello\"}\n",
                    )
                    .await
                    .unwrap();
                writer.flush().await.unwrap();
                drop(writer);

                // Wait up to ~1s for the call to appear.
                for _ in 0..20 {
                    if sink.calls.lock().unwrap().len() >= 2 {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }

                let calls = sink.calls.lock().unwrap().clone();
                handle.abort();

                assert_eq!(
                    calls,
                    vec![
                        "cancel:sess-2".to_string(),
                        "prompt:sess-2:hello".to_string(),
                    ],
                    "pre-existing line should be ignored; only appended line forwarded"
                );
            })
            .await;
    }
}
