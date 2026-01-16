//! Context editing strategies.
//!
//! Declarative strategies for managing LLM context windows.

use serde::{Deserialize, Serialize};

/// Parameters for edit operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditParams {
    /// Session ID to edit.
    pub session_id: uuid::Uuid,

    /// Strategies to apply (in order).
    pub strategies: Vec<EditStrategy>,
}

/// Result of an edit operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResult {
    /// Original token count.
    pub original_tokens: u32,

    /// Token count after editing.
    pub edited_tokens: u32,

    /// Number of messages before editing.
    pub original_messages: usize,

    /// Number of messages after editing.
    pub edited_messages: usize,

    /// Strategies that were applied.
    pub strategies_applied: Vec<String>,
}

impl EditResult {
    /// Create a new edit result.
    #[must_use]
    pub fn new(original_tokens: u32, original_messages: usize) -> Self {
        Self {
            original_tokens,
            edited_tokens: original_tokens,
            edited_messages: original_messages,
            original_messages,
            strategies_applied: Vec::new(),
        }
    }

    /// Calculate the reduction ratio.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn reduction_ratio(&self) -> f32 {
        if self.original_tokens == 0 {
            0.0
        } else {
            1.0 - (self.edited_tokens as f32 / self.original_tokens as f32)
        }
    }
}

/// Declarative editing strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditStrategy {
    /// Limit total tokens in context.
    TokenLimit {
        /// Maximum tokens to keep.
        limit: u32,
    },

    /// Remove tool results, keeping only recent ones.
    RemoveToolResult {
        /// Number of recent tool results to keep.
        keep_recent: usize,
    },

    /// Summarize older messages.
    SummarizeOlder {
        /// Messages older than this threshold get summarized.
        older_than_messages: usize,
    },

    /// Remove messages matching a pattern.
    RemovePattern {
        /// Regex pattern to match against message content.
        pattern: String,
    },

    /// Keep only the most recent N messages.
    KeepRecent {
        /// Number of recent messages to keep.
        count: usize,
    },

    /// Truncate long message contents.
    TruncateLong {
        /// Maximum characters per message.
        max_chars: usize,
    },
}

impl EditStrategy {
    /// Get a description of this strategy.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::TokenLimit { limit } => format!("Limit to {limit} tokens"),
            Self::RemoveToolResult { keep_recent } => {
                format!("Remove tool results except {keep_recent} recent")
            }
            Self::SummarizeOlder {
                older_than_messages,
            } => {
                format!("Summarize messages older than {older_than_messages}")
            }
            Self::RemovePattern { pattern } => format!("Remove messages matching '{pattern}'"),
            Self::KeepRecent { count } => format!("Keep {count} recent messages"),
            Self::TruncateLong { max_chars } => format!("Truncate messages to {max_chars} chars"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_result_reduction() {
        let mut result = EditResult::new(1000, 10);
        result.edited_tokens = 500;

        assert!((result.reduction_ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_strategy_descriptions() {
        let strategy = EditStrategy::TokenLimit { limit: 10000 };
        assert!(strategy.description().contains("10000"));

        let strategy = EditStrategy::KeepRecent { count: 5 };
        assert!(strategy.description().contains('5'));
    }
}
