//! MCP tool handlers for experience functionality.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::editing::{EditParams, EditResult, EditStrategy};
use crate::models::{SearchMode, Skill};

/// Request for searching skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSkillRequest {
    /// Query to search for.
    pub query: String,

    /// Space ID to search in.
    pub space_id: Uuid,

    /// Search mode.
    #[serde(default)]
    pub mode: SearchMode,

    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Response from skill search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSkillResponse {
    /// Found skills.
    pub skills: Vec<Skill>,

    /// Number of results.
    pub count: usize,
}

/// Request for editing context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditContextRequest {
    /// Session ID to edit.
    pub session_id: Uuid,

    /// Strategies to apply.
    pub strategies: Vec<EditStrategy>,
}

/// Request for token counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCountRequest {
    /// Session ID to count tokens for.
    pub session_id: Uuid,
}

/// Response with token counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCountResponse {
    /// Total tokens in the session.
    pub total_tokens: u32,

    /// Number of messages.
    pub message_count: usize,

    /// Breakdown by role.
    pub by_role: std::collections::HashMap<String, u32>,
}

/// Handlers for experience-related MCP tools.
pub struct ExperienceToolHandlers;

impl ExperienceToolHandlers {
    /// Create new handlers.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ExperienceToolHandlers {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle `search_skill` tool call.
pub async fn search_skill_handler(request: SearchSkillRequest) -> Result<SearchSkillResponse> {
    // TODO: Implement actual search using SkillSearcher
    let _ = request;
    Ok(SearchSkillResponse {
        skills: Vec::new(),
        count: 0,
    })
}

/// Handle `edit_context` tool call.
pub async fn edit_context_handler(request: EditContextRequest) -> Result<EditResult> {
    // TODO: Implement actual context editing
    let params = EditParams {
        session_id: request.session_id,
        strategies: request.strategies,
    };
    let _ = params;

    Ok(EditResult::new(0, 0))
}

/// Handle `get_token_counts` tool call.
pub async fn get_token_counts_handler(request: TokenCountRequest) -> Result<TokenCountResponse> {
    // TODO: Implement actual token counting from session
    let _ = request;
    Ok(TokenCountResponse {
        total_tokens: 0,
        message_count: 0,
        by_role: std::collections::HashMap::new(),
    })
}
