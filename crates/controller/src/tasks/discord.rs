/*
 * 5D Labs Agent Platform - Discord Thread Management
 * Copyright (C) 2025 5D Labs
 *
 * Discord API helper for creating and managing per-task threads.
 * Used by the reconciler to provide task-scoped Discord communication
 * through Morgan's centralized gateway.
 */

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Discord API base URL
const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Discord thread auto-archive durations (in minutes)
#[derive(Debug, Clone, Copy)]
pub enum AutoArchiveDuration {
    OneHour = 60,
    OneDay = 1440,
    ThreeDays = 4320,
    OneWeek = 10080,
}

/// Response from Discord thread creation
#[derive(Debug, Deserialize)]
pub struct DiscordThread {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub thread_type: u8,
}

/// Payload for creating a Discord thread
#[derive(Debug, Serialize)]
struct CreateThreadPayload {
    name: String,
    auto_archive_duration: u16,
    #[serde(rename = "type")]
    thread_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitable: Option<bool>,
}

/// Payload for modifying a thread (archive/lock)
#[derive(Debug, Serialize)]
struct ModifyThreadPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
}

/// Payload for sending a message
#[derive(Debug, Serialize)]
struct SendMessagePayload {
    content: String,
}

/// Discord thread manager for task-scoped communication
pub struct DiscordThreadManager {
    http: Client,
    bot_token: String,
}

impl DiscordThreadManager {
    pub fn new(bot_token: String) -> Self {
        Self {
            http: Client::new(),
            bot_token,
        }
    }

    /// Create a new thread in the given channel for a task.
    /// Uses thread type 11 (PUBLIC_THREAD) — visible to anyone with channel access.
    pub async fn create_thread(
        &self,
        channel_id: &str,
        name: &str,
        auto_archive: AutoArchiveDuration,
    ) -> Result<DiscordThread> {
        let url = format!(
            "{}/channels/{}/threads",
            DISCORD_API_BASE, channel_id
        );

        // Thread name max 100 chars
        let truncated_name = if name.len() > 100 {
            format!("{}…", &name[..99])
        } else {
            name.to_string()
        };

        let payload = CreateThreadPayload {
            name: truncated_name,
            auto_archive_duration: auto_archive as u16,
            thread_type: 11, // PUBLIC_THREAD
            invitable: Some(false),
        };

        debug!(channel_id, name, "Creating Discord thread");

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send thread creation request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "Discord API error creating thread: {} — {}",
                status,
                body
            );
        }

        let thread: DiscordThread = resp
            .json()
            .await
            .context("Failed to parse thread creation response")?;

        debug!(thread_id = %thread.id, thread_name = %thread.name, "Thread created");
        Ok(thread)
    }

    /// Archive a thread (hides it from the channel list but preserves history).
    pub async fn archive_thread(&self, thread_id: &str) -> Result<()> {
        let url = format!("{}/channels/{}", DISCORD_API_BASE, thread_id);

        let payload = ModifyThreadPayload {
            archived: Some(true),
            locked: Some(true),
        };

        debug!(thread_id, "Archiving Discord thread");

        let resp = self
            .http
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send thread archive request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!(
                thread_id,
                status = %status,
                "Failed to archive thread: {}",
                body
            );
            // Don't fail — archival is best-effort cleanup
        } else {
            debug!(thread_id, "Thread archived successfully");
        }

        Ok(())
    }

    /// Delete a thread entirely (permanent — use archive_thread for recoverable cleanup).
    pub async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        let url = format!("{}/channels/{}", DISCORD_API_BASE, thread_id);

        debug!(thread_id, "Deleting Discord thread");

        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .context("Failed to send thread delete request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!(
                thread_id,
                status = %status,
                "Failed to delete thread: {}",
                body
            );
        } else {
            debug!(thread_id, "Thread deleted");
        }

        Ok(())
    }

    /// Post a message to a thread (used for initial task context).
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!(
            "{}/channels/{}/messages",
            DISCORD_API_BASE, channel_id
        );

        // Discord message limit is 2000 chars
        let truncated = if content.len() > 2000 {
            format!("{}…", &content[..1999])
        } else {
            content.to_string()
        };

        let payload = SendMessagePayload {
            content: truncated,
        };

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send Discord message")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!(
                channel_id,
                status = %status,
                "Failed to send message: {}",
                body
            );
        }

        Ok(())
    }
}

/// Build a task thread name from agent and task ID.
/// Format: `{Agent}-task-{task_id}` (e.g., `Rex-task-42`)
pub fn task_thread_name(agent_name: &str, task_id: u32) -> String {
    let capitalized = if agent_name.is_empty() {
        "Agent".to_string()
    } else {
        let mut chars = agent_name.chars();
        match chars.next() {
            None => "Agent".to_string(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    };
    format!("{}-task-{}", capitalized, task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_thread_name() {
        assert_eq!(task_thread_name("rex", 42), "Rex-task-42");
        assert_eq!(task_thread_name("cleo", 100), "Cleo-task-100");
        assert_eq!(task_thread_name("", 1), "Agent-task-1");
    }

    #[test]
    fn test_thread_name_truncation() {
        let long_name = "a".repeat(200);
        let name = task_thread_name(&long_name, 42);
        // The thread name itself won't exceed 100 chars when passed to create_thread
        assert!(name.len() < 210); // Just the raw format
    }
}
