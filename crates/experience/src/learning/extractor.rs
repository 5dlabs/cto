//! Task extractor - parses tool calls and progress from sessions.

use anyhow::Result;
use tracing::{debug, info};

use crate::models::{MessageRecord, SessionRecord, TaskRecord, TaskStatus, ToolCallRecord};

/// Task extractor that analyzes sessions to extract structured task data.
pub struct TaskExtractor;

impl TaskExtractor {
    /// Create a new task extractor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extract tasks from a session's message history.
    ///
    /// This analyzes the conversation to identify:
    /// - Planned tasks from agent responses
    /// - Tool calls made during execution
    /// - Progress updates
    /// - User preferences
    pub fn extract_tasks(&self, session: &SessionRecord) -> Result<Vec<TaskRecord>> {
        info!(
            play_id = %session.play_id,
            messages = session.messages.len(),
            "Extracting tasks from session"
        );

        // If session already has tasks, enhance them with tool call data
        if !session.tasks.is_empty() {
            return Ok(self.enhance_existing_tasks(session));
        }

        // Otherwise, try to extract tasks from messages
        self.extract_tasks_from_messages(session)
    }

    /// Enhance existing tasks with tool call information from messages.
    fn enhance_existing_tasks(&self, session: &SessionRecord) -> Vec<TaskRecord> {
        let tool_calls = self.extract_tool_calls(&session.messages);
        let preferences = self.extract_preferences(&session.messages);
        let progress = self.extract_progress(&session.messages);

        session
            .tasks
            .iter()
            .map(|task| {
                let mut enhanced = task.clone();

                // Add tool calls (simple assignment - in production would correlate by time/context)
                if enhanced.tool_calls.is_empty() {
                    enhanced.tool_calls = tool_calls.clone();
                }

                // Add preferences if not already present
                if enhanced.user_preferences.is_empty() {
                    enhanced.user_preferences = preferences.clone();
                }

                // Add progress if not already present
                if enhanced.progresses.is_empty() {
                    enhanced.progresses = progress.clone();
                }

                enhanced
            })
            .collect()
    }

    /// Extract tasks from message history when no structured tasks exist.
    fn extract_tasks_from_messages(&self, session: &SessionRecord) -> Result<Vec<TaskRecord>> {
        let tool_calls = self.extract_tool_calls(&session.messages);
        let preferences = self.extract_preferences(&session.messages);
        let progress = self.extract_progress(&session.messages);

        // If we have tool calls, create a synthetic task
        if !tool_calls.is_empty() {
            let mut task = TaskRecord::new(
                format!("session-{}", session.play_id),
                format!("Work performed in session {}", session.play_id),
            );

            task.tool_calls = tool_calls;
            task.user_preferences = preferences;
            task.progresses = progress;
            task.started_at = session.started_at;
            task.completed_at = session.completed_at;

            // Determine status from session
            task.status = match session.status {
                crate::models::SessionStatus::Completed => TaskStatus::Success,
                crate::models::SessionStatus::Failed => TaskStatus::Failed,
                crate::models::SessionStatus::Cancelled => TaskStatus::Cancelled,
                crate::models::SessionStatus::Active => TaskStatus::Running,
            };

            debug!(
                task_id = %task.id,
                tool_calls = task.tool_calls.len(),
                "Created synthetic task from session"
            );

            Ok(vec![task])
        } else {
            Ok(Vec::new())
        }
    }

    /// Extract tool calls from messages.
    fn extract_tool_calls(&self, messages: &[MessageRecord]) -> Vec<ToolCallRecord> {
        let mut tool_calls = Vec::new();

        for msg in messages {
            if let Some(ref tool_name) = msg.tool_name {
                let tool_call = ToolCallRecord::new(
                    msg.tool_call_id.clone().unwrap_or_default(),
                    tool_name.clone(),
                    "{}", // Arguments would need to be extracted from content
                );
                tool_calls.push(tool_call);
            }

            // Also look for tool results
            if msg.role == "tool" {
                if let Some(ref call_id) = msg.tool_call_id {
                    // Find and update the corresponding tool call
                    for tc in &mut tool_calls {
                        if tc.id == *call_id && tc.result.is_none() {
                            tc.result = Some(truncate_result(&msg.content, 500));
                            tc.success = !msg.content.to_lowercase().contains("error");
                            break;
                        }
                    }
                }
            }
        }

        tool_calls
    }

    /// Extract user preferences from messages.
    fn extract_preferences(&self, messages: &[MessageRecord]) -> Vec<String> {
        let mut preferences = Vec::new();

        // Look for preference indicators in user messages
        let preference_indicators = [
            "prefer",
            "please use",
            "make sure to",
            "i want",
            "should use",
            "always",
            "never",
            "must",
        ];

        for msg in messages {
            if msg.role == "user" {
                let content_lower = msg.content.to_lowercase();
                for indicator in &preference_indicators {
                    if content_lower.contains(indicator) {
                        // Extract the sentence containing the preference
                        for sentence in msg.content.split(['.', '!', '?']) {
                            if sentence.to_lowercase().contains(indicator) {
                                let trimmed = sentence.trim();
                                if !trimmed.is_empty() && trimmed.len() < 200 {
                                    preferences.push(trimmed.to_string());
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Deduplicate
        preferences.sort();
        preferences.dedup();
        preferences
    }

    /// Extract progress updates from assistant messages.
    fn extract_progress(&self, messages: &[MessageRecord]) -> Vec<String> {
        let mut progress = Vec::new();

        let progress_indicators = [
            "i have",
            "i've",
            "completed",
            "finished",
            "done with",
            "successfully",
            "implemented",
            "created",
            "added",
            "updated",
            "fixed",
        ];

        for msg in messages {
            if msg.role == "assistant" {
                let content_lower = msg.content.to_lowercase();
                for indicator in &progress_indicators {
                    if content_lower.contains(indicator) {
                        // Extract first sentence as progress
                        if let Some(first_sentence) = msg.content.split(['.', '!', '?']).next() {
                            let trimmed = first_sentence.trim();
                            if !trimmed.is_empty() && trimmed.len() < 300 {
                                progress.push(trimmed.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        progress
    }
}

impl Default for TaskExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate a result string to a maximum length.
fn truncate_result(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_extractor_creation() {
        let extractor = TaskExtractor::new();
        assert!(std::mem::size_of_val(&extractor) > 0);
    }

    #[test]
    fn test_extract_empty_session() {
        let extractor = TaskExtractor::new();
        let session = SessionRecord::new("play-123", Uuid::new_v4());

        let tasks = extractor.extract_tasks(&session).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_extract_preferences() {
        let extractor = TaskExtractor::new();

        let messages = vec![
            MessageRecord::user("Please use async/await for all IO operations."),
            MessageRecord::user("I prefer using the Result type for error handling."),
            MessageRecord::assistant("I'll implement it with async/await."),
        ];

        let preferences = extractor.extract_preferences(&messages);
        assert!(!preferences.is_empty());
        assert!(preferences.iter().any(|p| p.contains("async/await")));
    }

    #[test]
    fn test_extract_progress() {
        let extractor = TaskExtractor::new();

        let messages = vec![
            MessageRecord::assistant("I have completed the initial setup."),
            MessageRecord::assistant("Successfully implemented the handler."),
            MessageRecord::user("Great, continue."),
        ];

        let progress = extractor.extract_progress(&messages);
        assert!(!progress.is_empty());
    }

    #[test]
    fn test_truncate_result() {
        let short = "short result";
        assert_eq!(truncate_result(short, 100), short);

        let long = "a".repeat(200);
        let truncated = truncate_result(&long, 50);
        assert!(truncated.len() <= 50);
        assert!(truncated.ends_with("..."));
    }
}
