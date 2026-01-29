//! Claude Agent SDK Provider - Calls compiled TypeScript binary.
//!
//! This provider spawns the `intake-agent` TypeScript binary and communicates
//! via JSON over stdin/stdout. The TypeScript binary uses the official Claude
//! Agent SDK for AI operations with MCP support.
//!
//! # Design Philosophy
//!
//! - Single provider, no fallbacks (fallbacks mask issues)
//! - Clean JSON protocol over subprocess (no CLI output parsing)
//! - Full MCP integration via official Claude Agent SDK
//! - Type-safe request/response with proper error handling

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::{TasksError, TasksResult};

use super::provider::{AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage};

/// Default model for Claude Agent SDK.
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Supported Claude models.
const SUPPORTED_MODELS: &[&str] = &[
    "claude-opus-4-20250514",
    "claude-opus-4-5-20251101",
    "claude-sonnet-4-20250514",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
    "claude-3-5-sonnet-latest",
    "claude-3-5-haiku-latest",
];

/// Binary name for the intake agent.
const BINARY_NAME: &str = "intake-agent";

// =============================================================================
// JSON Protocol Types (must match TypeScript definitions)
// =============================================================================

/// Request sent to the TypeScript agent.
#[derive(Debug, Serialize)]
struct AgentRequest<'a> {
    operation: &'static str,
    model: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<AgentOptions>,
    payload: AgentPayload<'a>,
}

/// Options for generation.
#[derive(Debug, Serialize)]
struct AgentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_enabled: Option<bool>,
}

/// Payload varies by operation - we use a generic text generation payload.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AgentPayload<'a> {
    /// Generic text generation with messages.
    TextGeneration {
        system_prompt: &'a str,
        user_prompt: &'a str,
        prefill: Option<&'a str>,
    },
    /// Multi-model generation with critic.
    MultiModelGeneration {
        system_prompt: &'a str,
        user_prompt: &'a str,
        prefill: Option<&'a str>,
        critic_context: Option<&'a str>,
        config: Option<MultiModelConfig>,
    },
}

/// Configuration for multi-model collaboration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModelConfig {
    /// Provider to use for generation (claude, minimax, codex).
    #[serde(default = "default_generator")]
    pub generator: String,
    /// Provider to use for critique/validation.
    #[serde(default = "default_critic")]
    pub critic: String,
    /// Maximum refinement iterations.
    #[serde(default = "default_max_refinements")]
    pub max_refinements: u32,
    /// Confidence threshold for approval (0.0-1.0).
    #[serde(default = "default_critic_threshold")]
    pub critic_threshold: f32,
    /// Specific model for generator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator_model: Option<String>,
    /// Specific model for critic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critic_model: Option<String>,
}

fn default_generator() -> String {
    "claude".to_string()
}

fn default_critic() -> String {
    "minimax".to_string()
}

fn default_max_refinements() -> u32 {
    2
}

fn default_critic_threshold() -> f32 {
    0.8
}

impl Default for MultiModelConfig {
    fn default() -> Self {
        Self {
            generator: default_generator(),
            critic: default_critic(),
            max_refinements: default_max_refinements(),
            critic_threshold: default_critic_threshold(),
            generator_model: None,
            critic_model: None,
        }
    }
}

/// Result from multi-model critic validation.
#[derive(Debug, Clone, Deserialize)]
pub struct CriticResult {
    /// Whether the content is approved.
    pub approved: bool,
    /// List of issues found.
    pub issues: Vec<CriticIssue>,
    /// General suggestions for improvement.
    pub suggestions: Vec<String>,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Raw reasoning from the critic.
    #[serde(default)]
    pub reasoning: Option<String>,
}

/// A single issue identified by the critic.
#[derive(Debug, Clone, Deserialize)]
pub struct CriticIssue {
    /// Severity of the issue (critical, major, minor).
    pub severity: String,
    /// Location/context where the issue was found.
    pub location: String,
    /// Description of the issue.
    pub description: String,
    /// Suggested fix.
    pub suggestion: String,
}

/// Response from multi-model generation.
#[derive(Debug, Clone, Deserialize)]
pub struct MultiModelResponse {
    /// Generated text.
    pub text: String,
    /// Number of refinement iterations performed.
    pub refinements: u32,
    /// Final critic result.
    pub critic_result: CriticResult,
    /// Usage breakdown by provider.
    pub usage_by_provider: std::collections::HashMap<String, AgentUsage>,
}

/// Response from the TypeScript agent.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AgentResponse {
    Success(AgentSuccessResponse),
    Error(AgentErrorResponse),
}

/// Successful response.
#[derive(Debug, Deserialize)]
struct AgentSuccessResponse {
    #[allow(dead_code)] // Used for JSON parsing discrimination
    success: bool,
    data: serde_json::Value,
    usage: AgentUsage,
    model: String,
    provider: String,
}

/// Error response.
#[derive(Debug, Deserialize)]
struct AgentErrorResponse {
    #[allow(dead_code)] // Used for JSON parsing discrimination
    success: bool,
    error: String,
    error_type: String,
    #[serde(default)]
    details: Option<String>,
}

/// Token usage from agent.
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::struct_field_names)] // Matches TypeScript SDK naming
struct AgentUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

// =============================================================================
// Provider Implementation
// =============================================================================

/// Claude Agent SDK Provider - Calls compiled TypeScript binary.
///
/// Uses the `intake-agent` binary for AI operations with MCP support.
/// This is the ONLY provider - no fallbacks.
pub struct AgentSdkProvider {
    /// Path to the intake-agent binary.
    binary_path: PathBuf,
    /// Default model to use if not specified.
    default_model: String,
}

impl AgentSdkProvider {
    /// Create a new provider with explicit binary path.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary is not found or not executable.
    pub fn new(binary_path: PathBuf) -> TasksResult<Self> {
        if !binary_path.exists() {
            return Err(TasksError::Ai(format!(
                "intake-agent binary not found at: {}",
                binary_path.display()
            )));
        }

        let default_model =
            std::env::var("TASKS_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Ok(Self {
            binary_path,
            default_model,
        })
    }

    /// Create provider by searching for the binary.
    ///
    /// Search order:
    /// 1. `INTAKE_AGENT_PATH` environment variable
    /// 2. `tools/intake-agent/dist/intake-agent` relative to crate root
    /// 3. `intake-agent` in PATH
    ///
    /// # Errors
    ///
    /// Returns an error if the binary cannot be found.
    pub fn from_env() -> TasksResult<Self> {
        // 1. Check environment variable
        if let Ok(path) = std::env::var("INTAKE_AGENT_PATH") {
            let binary_path = PathBuf::from(&path);
            if binary_path.exists() {
                tracing::info!(path = %path, "Using intake-agent from INTAKE_AGENT_PATH");
                return Self::new(binary_path);
            }
            tracing::warn!(
                path = %path,
                "INTAKE_AGENT_PATH is set but binary not found"
            );
        }

        // 2. Check relative path from workspace root
        // Try multiple possible locations
        let relative_paths = [
            "tools/intake-agent/dist/intake-agent",
            "../tools/intake-agent/dist/intake-agent",
            "../../tools/intake-agent/dist/intake-agent",
        ];

        for rel_path in relative_paths {
            let binary_path = PathBuf::from(rel_path);
            if binary_path.exists() {
                let canonical = binary_path.canonicalize().unwrap_or(binary_path.clone());
                tracing::info!(path = %canonical.display(), "Using intake-agent from relative path");
                return Self::new(canonical);
            }
        }

        // 3. Check PATH using `which`
        if let Ok(path) = which::which(BINARY_NAME) {
            tracing::info!(path = %path.display(), "Using intake-agent from PATH");
            return Self::new(path);
        }

        Err(TasksError::Ai(
            "intake-agent binary not found. Either:\n\
             1. Set INTAKE_AGENT_PATH environment variable\n\
             2. Build it with: cd tools/intake-agent && bun run build\n\
             3. Add it to PATH"
                .to_string(),
        ))
    }

    /// Execute a request against the agent binary.
    async fn execute_request(&self, request: &AgentRequest<'_>) -> TasksResult<AgentResponse> {
        let request_json = serde_json::to_string(request)
            .map_err(|e| TasksError::Ai(format!("Failed to serialize request to JSON: {e}")))?;

        tracing::debug!(
            binary = %self.binary_path.display(),
            request_len = request_json.len(),
            "Executing intake-agent request"
        );

        // Spawn the process
        let mut child = Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                TasksError::Ai(format!(
                    "Failed to spawn intake-agent at {}: {e}",
                    self.binary_path.display()
                ))
            })?;

        // Write request to stdin
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| TasksError::Ai("Failed to open stdin for intake-agent".to_string()))?;

        stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to write to intake-agent stdin: {e}")))?;

        // Close stdin to signal end of input
        drop(stdin);

        // Wait for output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to wait for intake-agent output: {e}")))?;

        // Check stderr for any warnings/errors
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::debug!(stderr = %stderr, "intake-agent stderr output");
        }

        // Parse stdout as JSON response
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.is_empty() {
            return Err(TasksError::Ai(format!(
                "intake-agent returned empty output. Exit code: {:?}",
                output.status.code()
            )));
        }

        tracing::debug!(
            stdout_len = stdout.len(),
            exit_code = ?output.status.code(),
            "intake-agent response received"
        );

        serde_json::from_str(&stdout).map_err(|e| {
            TasksError::Ai(format!(
                "Failed to parse intake-agent response as JSON: {e}. Output: {}",
                &stdout[..stdout.len().min(500)]
            ))
        })
    }
}

#[async_trait]
impl AIProvider for AgentSdkProvider {
    fn name(&self) -> &'static str {
        "claude-agent-sdk"
    }

    fn api_key_env_var(&self) -> &'static str {
        // The TypeScript agent reads ANTHROPIC_API_KEY
        "ANTHROPIC_API_KEY"
    }

    fn is_configured(&self) -> bool {
        // Check that the binary exists
        // Note: The Claude CLI handles API key authentication from its own config (~/.claude/)
        // so we don't require ANTHROPIC_API_KEY in the environment
        self.binary_path.exists()
    }

    fn supported_models(&self) -> Vec<&str> {
        SUPPORTED_MODELS.to_vec()
    }

    fn supports_model(&self, model: &str) -> bool {
        // Support any claude model
        model.starts_with("claude-") || SUPPORTED_MODELS.contains(&model)
    }

    async fn generate_text(
        &self,
        model: &str,
        messages: &[AIMessage],
        options: &GenerateOptions,
    ) -> TasksResult<AIResponse> {
        let model = if model.is_empty() {
            &self.default_model
        } else {
            model
        };

        // Extract system and user messages
        let system_prompt = messages
            .iter()
            .find(|m| m.role == AIRole::System)
            .map(|m| m.content.as_str())
            .unwrap_or("");

        // Combine user messages and any prefill (assistant) message
        let user_messages: Vec<&str> = messages
            .iter()
            .filter(|m| m.role == AIRole::User)
            .map(|m| m.content.as_str())
            .collect();

        let user_prompt = user_messages.join("\n\n");

        let prefill = messages
            .iter()
            .find(|m| m.role == AIRole::Assistant)
            .map(|m| m.content.as_str());

        tracing::debug!(
            model = %model,
            system_len = system_prompt.len(),
            user_len = user_prompt.len(),
            has_prefill = prefill.is_some(),
            "Generating text via Claude Agent SDK"
        );

        // Build request
        // Note: The TypeScript agent currently implements specific operations
        // (parse_prd, expand_task, analyze_complexity). For generic text generation,
        // we pass the prompts and let it handle them appropriately.
        //
        // TODO: Add a generic "generate" operation to the TypeScript agent for
        // cases that don't fit the specific operations.
        let request = AgentRequest {
            operation: "generate",
            model,
            options: Some(AgentOptions {
                temperature: options.temperature,
                max_tokens: options.max_tokens,
                mcp_enabled: if options.disable_mcp {
                    Some(false)
                } else {
                    None
                },
            }),
            payload: AgentPayload::TextGeneration {
                system_prompt,
                user_prompt: &user_prompt,
                prefill,
            },
        };

        // Execute request
        let response = self.execute_request(&request).await?;

        match response {
            AgentResponse::Success(success) => {
                // Extract text from response data
                let text = if let Some(text) = success.data.get("text") {
                    text.as_str().unwrap_or("").to_string()
                } else if let Some(result) = success.data.get("result") {
                    result.as_str().unwrap_or("").to_string()
                } else {
                    // Fallback: serialize the data as the response
                    serde_json::to_string(&success.data).unwrap_or_default()
                };

                if text.is_empty() {
                    return Err(TasksError::Ai(
                        "Claude Agent SDK returned empty response".to_string(),
                    ));
                }

                let usage = TokenUsage {
                    input_tokens: success.usage.input_tokens,
                    output_tokens: success.usage.output_tokens,
                    total_tokens: success.usage.total_tokens,
                };

                tracing::debug!(
                    response_len = text.len(),
                    input_tokens = usage.input_tokens,
                    output_tokens = usage.output_tokens,
                    "Claude Agent SDK response received"
                );

                Ok(AIResponse {
                    text,
                    usage,
                    model: success.model,
                    provider: success.provider,
                })
            }
            AgentResponse::Error(error) => Err(TasksError::Ai(format!(
                "Claude Agent SDK error ({}): {}{}",
                error.error_type,
                error.error,
                error
                    .details
                    .map(|d| format!("\nDetails: {d}"))
                    .unwrap_or_default()
            ))),
        }
    }
}

impl AgentSdkProvider {
    /// Generate text with multi-model critic validation.
    ///
    /// Uses the critic/validator pattern where one model generates content
    /// and another reviews/critiques it for quality assurance.
    ///
    /// # Arguments
    ///
    /// * `system_prompt` - System prompt for the generator
    /// * `user_prompt` - User prompt (main input)
    /// * `config` - Multi-model configuration
    /// * `critic_context` - Optional context for the critic
    ///
    /// # Returns
    ///
    /// Result containing the generated text, critic result, and usage information.
    pub async fn generate_with_critic(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        config: Option<MultiModelConfig>,
        critic_context: Option<&str>,
    ) -> TasksResult<(String, CriticResult, TokenUsage)> {
        let config = config.unwrap_or_default();

        tracing::info!(
            generator = %config.generator,
            critic = %config.critic,
            max_refinements = config.max_refinements,
            "Starting multi-model generation with critic"
        );

        let request = AgentRequest {
            operation: "generate_with_critic",
            model: config.generator_model.as_deref().unwrap_or(&self.default_model),
            options: None,
            payload: AgentPayload::MultiModelGeneration {
                system_prompt,
                user_prompt,
                prefill: None,
                critic_context,
                config: Some(config.clone()),
            },
        };

        let response = self.execute_request(&request).await?;

        match response {
            AgentResponse::Success(success) => {
                // Parse the multi-model response
                let multi_response: MultiModelResponse = serde_json::from_value(success.data)
                    .map_err(|e| TasksError::Ai(format!("Failed to parse multi-model response: {e}")))?;

                let usage = TokenUsage {
                    input_tokens: success.usage.input_tokens,
                    output_tokens: success.usage.output_tokens,
                    total_tokens: success.usage.total_tokens,
                };

                tracing::info!(
                    refinements = multi_response.refinements,
                    approved = multi_response.critic_result.approved,
                    confidence = multi_response.critic_result.confidence,
                    issues = multi_response.critic_result.issues.len(),
                    "Multi-model generation complete"
                );

                Ok((multi_response.text, multi_response.critic_result, usage))
            }
            AgentResponse::Error(error) => Err(TasksError::Ai(format!(
                "Multi-model generation error ({}): {}{}",
                error.error_type,
                error.error,
                error
                    .details
                    .map(|d| format!("\nDetails: {d}"))
                    .unwrap_or_default()
            ))),
        }
    }

    /// Check provider availability status.
    ///
    /// Returns information about which providers (Claude, Minimax, Codex) are available.
    pub async fn get_provider_status(&self) -> TasksResult<std::collections::HashMap<String, bool>> {
        let request_json = r#"{"operation":"provider_status","payload":{}}"#;

        tracing::debug!("Checking provider status");

        let mut child = Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                TasksError::Ai(format!(
                    "Failed to spawn intake-agent at {}: {e}",
                    self.binary_path.display()
                ))
            })?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| TasksError::Ai("Failed to open stdin for intake-agent".to_string()))?;

        stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to write to intake-agent stdin: {e}")))?;

        drop(stdin);

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to wait for intake-agent output: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(Deserialize)]
        struct ProviderStatusResponse {
            success: bool,
            data: ProviderStatusData,
        }

        #[derive(Deserialize)]
        struct ProviderStatusData {
            providers: std::collections::HashMap<String, ProviderInfo>,
        }

        #[derive(Deserialize)]
        struct ProviderInfo {
            available: bool,
            #[allow(dead_code)]
            model: String,
        }

        let response: ProviderStatusResponse = serde_json::from_str(&stdout)
            .map_err(|e| TasksError::Ai(format!("Failed to parse provider status: {e}")))?;

        let mut status = std::collections::HashMap::new();
        for (name, info) in response.data.providers {
            status.insert(name, info.available);
        }

        Ok(status)
    }
}

// Keep the old name as an alias for compatibility
pub type AnthropicProvider = AgentSdkProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_claude_models() {
        assert!(SUPPORTED_MODELS.contains(&"claude-sonnet-4-20250514"));
        assert!(SUPPORTED_MODELS.contains(&"claude-3-5-sonnet-20241022"));
        assert!(SUPPORTED_MODELS.contains(&"claude-3-5-sonnet-latest"));
    }

    #[test]
    fn test_default_model() {
        assert_eq!(DEFAULT_MODEL, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_request_serialization() {
        let request = AgentRequest {
            operation: "generate",
            model: "claude-sonnet-4-20250514",
            options: Some(AgentOptions {
                temperature: Some(0.7),
                max_tokens: Some(1000),
                mcp_enabled: None,
            }),
            payload: AgentPayload::TextGeneration {
                system_prompt: "You are a helpful assistant.",
                user_prompt: "Hello!",
                prefill: None,
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"operation\":\"generate\""));
        assert!(json.contains("\"model\":\"claude-sonnet-4-20250514\""));
        assert!(json.contains("\"temperature\":0.7"));
    }
}
