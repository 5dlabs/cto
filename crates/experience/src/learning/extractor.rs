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
            return Ok(Self::enhance_existing_tasks(session));
        }

        // Otherwise, try to extract tasks from messages
        Ok(Self::extract_tasks_from_messages(session))
    }

    /// Enhance existing tasks with tool call information from messages.
    fn enhance_existing_tasks(session: &SessionRecord) -> Vec<TaskRecord> {
        session
            .tasks
            .iter()
            .map(|task| {
                let mut enhanced = task.clone();
                let scoped_messages = messages_for_task(&session.messages, task, session);

                // Add tool calls (simple assignment - in production would correlate by time/context)
                if enhanced.tool_calls.is_empty() {
                    enhanced.tool_calls = Self::extract_tool_calls(&scoped_messages);
                }

                // Add preferences if not already present
                if enhanced.user_preferences.is_empty() {
                    enhanced.user_preferences = Self::extract_preferences(&scoped_messages);
                }

                // Add progress if not already present
                if enhanced.progresses.is_empty() {
                    enhanced.progresses = Self::extract_progress(&scoped_messages);
                }

                enhanced
            })
            .collect()
    }

    /// Extract tasks from message history when no structured tasks exist.
    fn extract_tasks_from_messages(session: &SessionRecord) -> Vec<TaskRecord> {
        let tool_calls = Self::extract_tool_calls(&session.messages);
        let preferences = Self::extract_preferences(&session.messages);
        let progress = Self::extract_progress(&session.messages);

        // If we have tool calls, create a synthetic task
        if tool_calls.is_empty() {
            Vec::new()
        } else {
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

            vec![task]
        }
    }

    /// Extract tool calls from messages.
    fn extract_tool_calls(messages: &[MessageRecord]) -> Vec<ToolCallRecord> {
        let mut tool_calls = Vec::new();
        let mut missing_id_counter = 0usize;

        for msg in messages {
            if let Some(ref tool_name) = msg.tool_name {
                let tool_call_id = normalize_tool_call_id(msg.tool_call_id.as_ref())
                    .unwrap_or_else(|| {
                        missing_id_counter += 1;
                        format!("missing-{missing_id_counter}")
                    });
                let tool_call = ToolCallRecord::new(
                    tool_call_id,
                    tool_name.clone(),
                    "{}", // Arguments would need to be extracted from content
                );
                tool_calls.push(tool_call);
            }

            // Also look for tool results
            if msg.role == "tool" {
                if let Some(call_id) = normalize_tool_call_id(msg.tool_call_id.as_ref()) {
                    // Find and update the corresponding tool call
                    for tc in &mut tool_calls {
                        if tc.id == call_id && tc.result.is_none() {
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
    fn extract_preferences(messages: &[MessageRecord]) -> Vec<String> {
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
    fn extract_progress(messages: &[MessageRecord]) -> Vec<String> {
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

/// Truncate a result string to a maximum length (UTF-8 safe).
///
/// This function ensures truncation occurs at a valid UTF-8 character boundary
/// to avoid panics when the string contains multi-byte characters (emoji, CJK, etc.).
fn truncate_result(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Reserve 3 bytes for "..."
        let truncate_at = max_len.saturating_sub(3);
        // Find a valid UTF-8 boundary at or before truncate_at
        let mut boundary = truncate_at.min(s.len());
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        format!("{}...", &s[..boundary])
    }
}

fn normalize_tool_call_id(id: Option<&String>) -> Option<String> {
    id.map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn messages_for_task(
    messages: &[MessageRecord],
    task: &TaskRecord,
    session: &SessionRecord,
) -> Vec<MessageRecord> {
    let start = task.started_at;
    let mut end = task
        .completed_at
        .or(session.completed_at)
        .unwrap_or_else(|| {
            messages
                .iter()
                .map(|message| message.timestamp)
                .max()
                .unwrap_or(start)
        });

    if end < start {
        end = start;
    }

    messages
        .iter()
        .filter(|message| message.timestamp >= start && message.timestamp <= end)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_extractor_creation() {
        let extractor = TaskExtractor::new();
        // TaskExtractor is a unit struct (ZST), just verify it can be created
        let _ = extractor;
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
        let messages = vec![
            MessageRecord::user("Please use async/await for all IO operations."),
            MessageRecord::user("I prefer using the Result type for error handling."),
            MessageRecord::assistant("I'll implement it with async/await."),
        ];

        let preferences = TaskExtractor::extract_preferences(&messages);
        assert!(!preferences.is_empty());
        assert!(preferences.iter().any(|p| p.contains("async/await")));
    }

    #[test]
    fn test_extract_progress() {
        let messages = vec![
            MessageRecord::assistant("I have completed the initial setup."),
            MessageRecord::assistant("Successfully implemented the handler."),
            MessageRecord::user("Great, continue."),
        ];

        let progress = TaskExtractor::extract_progress(&messages);
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

    #[test]
    fn test_truncate_result_utf8_safe() {
        // Test with emoji (4-byte UTF-8 characters)
        // Each emoji like 🎉 is 4 bytes
        let emoji_string = "🎉🎊🎈🎁🎂"; // 5 emojis = 20 bytes
                                         // Truncating at 10 bytes would fall in the middle of an emoji without proper handling
        let truncated = truncate_result(emoji_string, 10);
        assert!(truncated.ends_with("..."));
        // Should be valid UTF-8 (this would panic if truncation was invalid)
        assert!(truncated.chars().count() > 0);

        // Test with CJK characters (3-byte UTF-8 characters)
        let cjk_string = "你好世界测试"; // 6 chars = 18 bytes
        let truncated_cjk = truncate_result(cjk_string, 10);
        assert!(truncated_cjk.ends_with("..."));
        // Verify it's valid UTF-8
        assert!(truncated_cjk.chars().count() > 0);

        // Test mixed ASCII and multi-byte
        let mixed = "Hello 世界! 🌍";
        let truncated_mixed = truncate_result(mixed, 12);
        assert!(truncated_mixed.ends_with("..."));
        assert!(truncated_mixed.chars().count() > 0);

        // Test edge case: max_len smaller than "..."
        let tiny_truncate = truncate_result("hello", 2);
        assert_eq!(tiny_truncate, "...");

        // Test edge case: max_len exactly 3
        let exactly_three = truncate_result("hello", 3);
        assert_eq!(exactly_three, "...");
    }

    #[test]
    fn test_tool_results_without_ids_do_not_attach_to_missing_call_ids() {
        let messages = vec![
            MessageRecord {
                role: "assistant".to_string(),
                content: "Call tool A".to_string(),
                tool_call_id: None,
                tool_name: Some("tool_a".to_string()),
                token_count: 0,
                timestamp: chrono::Utc::now(),
            },
            MessageRecord {
                role: "assistant".to_string(),
                content: "Call tool B".to_string(),
                tool_call_id: None,
                tool_name: Some("tool_b".to_string()),
                token_count: 0,
                timestamp: chrono::Utc::now(),
            },
            MessageRecord::tool("", "result one"),
            MessageRecord::tool("", "result two"),
        ];

        let tool_calls = TaskExtractor::extract_tool_calls(&messages);
        assert_eq!(tool_calls.len(), 2);
        assert!(tool_calls.iter().all(|tc| tc.result.is_none()));
    }

    #[test]
    #[allow(clippy::too_many_lines)] // Complex function not easily split
    fn test_enhance_existing_tasks_scopes_messages_by_task_time() {
        let extractor = TaskExtractor::new();

        let base_time = chrono::Utc::now();
        let task1_start = base_time - chrono::Duration::seconds(10);
        let task1_end = base_time - chrono::Duration::seconds(5);
        let task2_start = base_time - chrono::Duration::seconds(4);
        let task2_end = base_time - chrono::Duration::seconds(1);

        let mut task1 = TaskRecord::new("task-1", "First task");
        task1.started_at = task1_start;
        task1.completed_at = Some(task1_end);

        let mut task2 = TaskRecord::new("task-2", "Second task");
        task2.started_at = task2_start;
        task2.completed_at = Some(task2_end);

        let messages = vec![
            MessageRecord {
                role: "assistant".to_string(),
                content: "Calling tool for task 1".to_string(),
                tool_call_id: Some("call-1".to_string()),
                tool_name: Some("tool_one".to_string()),
                token_count: 0,
                timestamp: task1_start + chrono::Duration::seconds(1),
            },
            MessageRecord {
                role: "tool".to_string(),
                content: "result 1".to_string(),
                tool_call_id: Some("call-1".to_string()),
                tool_name: None,
                token_count: 0,
                timestamp: task1_start + chrono::Duration::seconds(2),
            },
            MessageRecord {
                role: "user".to_string(),
                content: "Please use serde for task 1.".to_string(),
                tool_call_id: None,
                tool_name: None,
                token_count: 0,
                timestamp: task1_start + chrono::Duration::seconds(3),
            },
            MessageRecord {
                role: "assistant".to_string(),
                content: "Completed task 1 setup.".to_string(),
                tool_call_id: None,
                tool_name: None,
                token_count: 0,
                timestamp: task1_start + chrono::Duration::seconds(4),
            },
            MessageRecord {
                role: "assistant".to_string(),
                content: "Calling tool for task 2".to_string(),
                tool_call_id: Some("call-2".to_string()),
                tool_name: Some("tool_two".to_string()),
                token_count: 0,
                timestamp: task2_start + chrono::Duration::seconds(1),
            },
            MessageRecord {
                role: "tool".to_string(),
                content: "result 2".to_string(),
                tool_call_id: Some("call-2".to_string()),
                tool_name: None,
                token_count: 0,
                timestamp: task2_start + chrono::Duration::seconds(2),
            },
            MessageRecord {
                role: "user".to_string(),
                content: "I prefer using tokio in task 2.".to_string(),
                tool_call_id: None,
                tool_name: None,
                token_count: 0,
                timestamp: task2_start + chrono::Duration::seconds(3),
            },
            MessageRecord {
                role: "assistant".to_string(),
                content: "Completed task 2 implementation.".to_string(),
                tool_call_id: None,
                tool_name: None,
                token_count: 0,
                timestamp: task2_end,
            },
        ];

        let mut session = SessionRecord::new("play-1", uuid::Uuid::new_v4());
        session.tasks = vec![task1, task2];
        session.messages = messages;

        let extracted = extractor.extract_tasks(&session).expect("tasks extracted");
        assert_eq!(extracted.len(), 2);

        let first_task = extracted.iter().find(|t| t.id == "task-1").unwrap();
        let second_task = extracted.iter().find(|t| t.id == "task-2").unwrap();

        assert_eq!(first_task.tool_calls.len(), 1);
        assert_eq!(first_task.tool_calls[0].tool_name, "tool_one");
        assert!(first_task
            .user_preferences
            .iter()
            .any(|pref| pref.contains("serde")));
        assert!(first_task
            .progresses
            .iter()
            .any(|progress| progress.contains("Completed task 1")));

        assert_eq!(second_task.tool_calls.len(), 1);
        assert_eq!(second_task.tool_calls[0].tool_name, "tool_two");
        assert!(second_task
            .user_preferences
            .iter()
            .any(|pref| pref.contains("tokio")));
        assert!(second_task
            .progresses
            .iter()
            .any(|progress| progress.contains("Completed task 2")));
    }
}
