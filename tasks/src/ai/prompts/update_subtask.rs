//! Update subtask prompt template.
//!
//! Appends timestamped notes to a subtask.

use serde::Serialize;

use super::PromptTemplate;

/// Context for update-subtask prompt.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSubtaskContext {
    /// The subtask to update (as JSON)
    pub subtask: serde_json::Value,
    /// The parent task for context
    pub parent_task: serde_json::Value,
    /// The information to append
    pub update_prompt: String,
    /// Use research mode
    pub use_research: bool,
    /// Current subtask details
    pub current_details: String,
    /// Additional project context
    pub gathered_context: String,
    /// Project root path
    pub project_root: String,
}

impl Default for UpdateSubtaskContext {
    fn default() -> Self {
        Self {
            subtask: serde_json::json!({}),
            parent_task: serde_json::json!({}),
            update_prompt: String::new(),
            use_research: false,
            current_details: "(No existing details)".to_string(),
            gathered_context: String::new(),
            project_root: String::new(),
        }
    }
}

/// Get the update-subtask template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new(
        "update-subtask",
        SYSTEM_PROMPT,
        USER_PROMPT,
    ).with_description("Append timestamped notes to a subtask")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping to append additional information to a software development subtask. You will be provided with the subtask's existing details, context, and a user request string.

Your Goal: Based *only* on the user's request and all the provided context (including existing details if relevant to the request), GENERATE the new text content that should be added to the subtask's details.
Focus *only* on generating the substance of the update.

Output Requirements:
1. Return *only* the newly generated text content as a plain string. Do NOT return a JSON object or any other structured data.
2. Your string response should NOT include any of the subtask's original details, unless the user's request explicitly asks to rephrase, summarize, or directly modify existing text.
3. Do NOT include any timestamps, XML-like tags, markdown, or any other special formatting in your string response.
4. Ensure the generated text is concise yet complete for the update based on the user request. Avoid conversational fillers or explanations about what you are doing (e.g., do not start with "Okay, here's the update...")."#;

const USER_PROMPT: &str = r#"Subtask Context:

Parent Task: {{{json parent_task}}}
Subtask: {{{json subtask}}}
Current Subtask Details (for context only):
{{current_details}}

User Request: "{{update_prompt}}"

Based on the User Request and all the Subtask Context (including current subtask details provided above), what is the new information or text that should be appended to this subtask's details? Return this new text as a plain string.
{{#if gathered_context}}

# Additional Project Context

{{gathered_context}}
{{/if}}"#;

