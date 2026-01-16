//! Session collector - receives events from Healer's `SessionStore`.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::models::{SessionRecord, SessionStatus};

/// Event types emitted by the session collector.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// A session was started.
    Started(SessionRecord),
    /// A session was updated.
    Updated(SessionRecord),
    /// A session was completed (success or failure).
    Completed(SessionRecord),
}

/// Trait for receiving session events.
#[async_trait]
pub trait SessionEventHandler: Send + Sync {
    /// Handle a session event.
    async fn handle(&self, event: SessionEvent) -> Result<()>;
}

/// Session collector that receives events and dispatches to handlers.
pub struct SessionCollector {
    /// Channel sender for incoming events.
    sender: mpsc::Sender<SessionEvent>,
    /// Handlers for session events.
    handlers: Vec<Arc<dyn SessionEventHandler>>,
}

impl SessionCollector {
    /// Create a new session collector.
    #[must_use]
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<SessionEvent>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (
            Self {
                sender,
                handlers: Vec::new(),
            },
            receiver,
        )
    }

    /// Add an event handler.
    pub fn add_handler(&mut self, handler: Arc<dyn SessionEventHandler>) {
        self.handlers.push(handler);
    }

    /// Get a sender clone for submitting events.
    #[must_use]
    pub fn sender(&self) -> mpsc::Sender<SessionEvent> {
        self.sender.clone()
    }

    /// Submit a session started event.
    pub async fn on_session_started(&self, session: SessionRecord) -> Result<()> {
        debug!(play_id = %session.play_id, "Session started event");
        self.sender
            .send(SessionEvent::Started(session))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send session started event: {e}"))?;
        Ok(())
    }

    /// Submit a session updated event.
    pub async fn on_session_updated(&self, session: SessionRecord) -> Result<()> {
        debug!(play_id = %session.play_id, "Session updated event");
        self.sender
            .send(SessionEvent::Updated(session))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send session updated event: {e}"))?;
        Ok(())
    }

    /// Submit a session completed event.
    pub async fn on_session_completed(&self, session: SessionRecord) -> Result<()> {
        info!(
            play_id = %session.play_id,
            status = %session.status,
            "Session completed event"
        );
        self.sender
            .send(SessionEvent::Completed(session))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send session completed event: {e}"))?;
        Ok(())
    }

    /// Process events from the receiver channel.
    pub async fn process_events(
        handlers: Vec<Arc<dyn SessionEventHandler>>,
        mut receiver: mpsc::Receiver<SessionEvent>,
    ) {
        info!("Starting session event processor");

        while let Some(event) = receiver.recv().await {
            let event_type = match &event {
                SessionEvent::Started(_) => "started",
                SessionEvent::Updated(_) => "updated",
                SessionEvent::Completed(_) => "completed",
            };

            debug!(event_type, "Processing session event");

            for handler in &handlers {
                if let Err(e) = handler.handle(event.clone()).await {
                    warn!(
                        error = %e,
                        event_type,
                        "Handler failed to process event"
                    );
                }
            }
        }

        info!("Session event processor stopped");
    }
}

/// Handler that queues successful sessions for learning.
pub struct LearningQueueHandler {
    /// Queue for sessions to be processed for learning.
    queue: mpsc::Sender<SessionRecord>,
}

impl LearningQueueHandler {
    /// Create a new learning queue handler.
    #[must_use]
    #[allow(dead_code)]
    pub fn new(queue: mpsc::Sender<SessionRecord>) -> Self {
        Self { queue }
    }
}

#[async_trait]
impl SessionEventHandler for LearningQueueHandler {
    async fn handle(&self, event: SessionEvent) -> Result<()> {
        if let SessionEvent::Completed(session) = event {
            // Only queue successful sessions for learning
            if session.status == SessionStatus::Completed {
                info!(
                    play_id = %session.play_id,
                    tasks = session.tasks.len(),
                    "Queueing successful session for learning"
                );
                self.queue
                    .send(session)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to queue session for learning: {e}"))?;
            } else {
                debug!(
                    play_id = %session.play_id,
                    status = %session.status,
                    "Skipping non-successful session"
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_session_collector_creation() {
        let (collector, _receiver) = SessionCollector::new(100);
        assert!(collector.handlers.is_empty());
    }

    #[tokio::test]
    async fn test_event_sending() {
        let (collector, mut receiver) = SessionCollector::new(100);

        let space_id = Uuid::new_v4();
        let session = SessionRecord::new("play-123", space_id);

        collector.on_session_started(session.clone()).await.unwrap();

        let event = receiver.recv().await.unwrap();
        match event {
            SessionEvent::Started(s) => {
                assert_eq!(s.play_id, "play-123");
            }
            _ => panic!("Expected Started event"),
        }
    }
}
