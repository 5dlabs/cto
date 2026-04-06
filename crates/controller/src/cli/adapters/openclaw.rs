//! OpenClaw Agent Harness Adapter Implementation
//!
//! Implements the `CliAdapter` trait for the OpenClaw agent harness.
//! OpenClaw is provider-agnostic and runs as a one-shot embedded harness
//! inside K8s Jobs with native tools, skills, and structured JSON output.

use crate::cli::adapter::{
    AdapterError, AdapterResult, AgentConfig, AuthMethod, CliAdapter, CliCapabilities,
    ConfigFormat, ContainerContext, FinishReason, HealthState, HealthStatus, MemoryStrategy,
    ParsedResponse, ResponseMetadata,
};
use crate::cli::base_adapter::{AdapterConfig, BaseAdapter};
use crate::cli::types::CLIType;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

#[derive(Debug)]
pub struct OpenClawAdapter {
    base: Arc<BaseAdapter>,
}

#[derive(Debug, Clone)]
struct OpenClawProviderConfig {
    provider_id: &'static str,
    api: &'static str,
    api_key_env: &'static str,
    base_url: &'static str,
    builtin: bool,
    provider_model_id: String,
    provider_model_alias: Option<String>,
}

impl OpenClawAdapter {
    fn normalize_agent_id(github_app: &str) -> String {
        github_app
            .strip_prefix("5DLabs-")
            .unwrap_or(github_app)
            .to_lowercase()
    }

    fn resolve_provider(model: &str) -> OpenClawProviderConfig {
        let normalized = model.trim().to_ascii_lowercase();

        if normalized.starts_with("openai-codex/") {
            let provider_model_id = model
                .split_once('/')
                .map(|(_, model_id)| model_id.to_string())
                .unwrap_or_else(|| model.to_string());
            return OpenClawProviderConfig {
                provider_id: "openai-codex",
                api: "",
                api_key_env: "",
                base_url: "",
                builtin: true,
                provider_model_id,
                provider_model_alias: Some(model.to_string()),
            };
        }

        if normalized.starts_with("openai/") {
            let provider_model_id = model
                .split_once('/')
                .map(|(_, model_id)| model_id.to_string())
                .unwrap_or_else(|| model.to_string());
            return OpenClawProviderConfig {
                provider_id: "openai",
                api: "openai-responses",
                api_key_env: "OPENAI_API_KEY",
                base_url: "https://api.openai.com/v1",
                builtin: false,
                provider_model_id,
                provider_model_alias: Some(model.to_string()),
            };
        }

        if normalized.starts_with("anthropic/") {
            let provider_model_id = model
                .split_once('/')
                .map(|(_, model_id)| model_id.to_string())
                .unwrap_or_else(|| model.to_string());
            return OpenClawProviderConfig {
                provider_id: "anthropic",
                api: "anthropic-messages",
                api_key_env: "ANTHROPIC_API_KEY",
                base_url: "https://api.anthropic.com",
                builtin: false,
                provider_model_id,
                provider_model_alias: Some(model.to_string()),
            };
        }

        if normalized.starts_with("gpt-")
            || normalized.starts_with("o1")
            || normalized.starts_with("o3")
            || normalized.starts_with("o4")
            || normalized.starts_with("text-embedding-")
        {
            return OpenClawProviderConfig {
                provider_id: "openai",
                api: "openai-responses",
                api_key_env: "OPENAI_API_KEY",
                base_url: "https://api.openai.com/v1",
                builtin: false,
                provider_model_id: model.to_string(),
                provider_model_alias: None,
            };
        }

        OpenClawProviderConfig {
            provider_id: "anthropic",
            api: "anthropic-messages",
            api_key_env: "ANTHROPIC_API_KEY",
            base_url: "https://api.anthropic.com",
            builtin: false,
            provider_model_id: model.to_string(),
            provider_model_alias: None,
        }
    }

    pub fn new() -> AdapterResult<Self> {
        Self::with_config(AdapterConfig::new(CLIType::OpenClaw))
    }

    pub fn with_config(config: AdapterConfig) -> AdapterResult<Self> {
        info!("Initializing OpenClaw adapter");
        let base = Arc::new(BaseAdapter::new(config)?);
        let adapter = Self { base };
        info!("OpenClaw adapter initialized successfully");
        Ok(adapter)
    }
}

#[async_trait]
impl CliAdapter for OpenClawAdapter {
    #[instrument(skip(self))]
    async fn validate_model(&self, model: &str) -> Result<bool> {
        debug!(model = %model, "Validating OpenClaw model");
        let is_valid = !model.trim().is_empty();
        info!(model = %model, is_valid = is_valid, "OpenClaw model validation completed");
        Ok(is_valid)
    }

    #[instrument(skip(self, agent_config))]
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        debug!(
            github_app = %agent_config.github_app,
            model = %agent_config.model,
            "Generating OpenClaw configuration"
        );

        self.base.validate_base_config(agent_config)?;

        let agent_id = Self::normalize_agent_id(&agent_config.github_app);
        let provider = Self::resolve_provider(&agent_config.model);
        let openclaw_state_dir = "/workspace/.openclaw";
        let workspace_dir = "/workspace";
        let agent_dir = format!("{openclaw_state_dir}/agents/{agent_id}/agent");
        let session_store = format!("{openclaw_state_dir}/agents/{agent_id}/sessions/sessions.json");
        let provider_model_id = provider.provider_model_id.clone();
        let mut provider_models = vec![json!({
            "id": provider_model_id,
            "name": agent_config.model,
        })];

        if let Some(alias) = provider.provider_model_alias {
            if alias != provider_model_id {
                provider_models.push(json!({
                    "id": alias,
                    "name": agent_config.model,
                }));
            }
        }

        let providers = if provider.builtin {
            json!({})
        } else {
            json!({
                (provider.provider_id): {
                    "api": provider.api,
                    "apiKey": format!("${{{}}}", provider.api_key_env),
                    "baseUrl": provider.base_url,
                    "auth": "api-key",
                    "authHeader": true,
                    "headers": {},
                    "models": provider_models
                }
            })
        };

        let config = json!({
            "agents": {
                "defaults": {
                    "workspace": workspace_dir,
                    "model": {
                        "primary": agent_config.model,
                        "fallbacks": []
                    }
                },
                "list": [{
                    "agentDir": agent_dir,
                    "id": agent_id,
                    "name": agent_config.github_app,
                    "model": agent_config.model,
                    "sandbox": {
                        "mode": "off"
                    },
                    "workspace": workspace_dir
                }]
            },
            "models": {
                "providers": providers
            },
            "gateway": {
                "mode": "local"
            },
            "commands": {
                "native": "auto",
                "nativeSkills": "auto"
            },
            "messages": {
                "ackReactionScope": "group-mentions"
            },
            "session": {
                "store": session_store
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "generated_by": "cli_adapter_openclaw",
            "version": "1.0",
        });

        let config_str = serde_json::to_string_pretty(&config).map_err(|e| {
            AdapterError::ConfigGenerationError(format!(
                "Failed to serialize OpenClaw configuration: {e}"
            ))
        })?;

        info!(
            config_length = config_str.len(),
            "OpenClaw configuration generated successfully"
        );

        Ok(config_str)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        prompt.to_string()
    }

    #[instrument(skip(self, response))]
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        debug!(response_length = response.len(), "Parsing OpenClaw response");

        Ok(ParsedResponse {
            content: response.to_string(),
            tool_calls: vec![],
            metadata: ResponseMetadata {
                input_tokens: None,
                output_tokens: None,
                duration_ms: None,
                model: None,
                extra: HashMap::new(),
            },
            finish_reason: FinishReason::Stop,
            streaming_delta: None,
        })
    }

    fn get_memory_filename(&self) -> &'static str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &'static str {
        "openclaw"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 200_000,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        }
    }

    #[instrument(skip(self, container))]
    async fn initialize(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Initializing OpenClaw adapter for container"
        );
        self.base.base_initialize(container).await?;
        info!("OpenClaw adapter initialization completed");
        Ok(())
    }

    #[instrument(skip(self, container))]
    async fn cleanup(&self, container: &ContainerContext) -> Result<()> {
        info!(
            container_name = %container.container_name,
            "Cleaning up OpenClaw adapter"
        );
        self.base.base_cleanup(container).await?;
        debug!("OpenClaw adapter cleanup completed");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing OpenClaw adapter health check");

        let container = ContainerContext {
            pod: None,
            container_name: "health-check".to_string(),
            working_dir: "/tmp".to_string(),
            env_vars: HashMap::new(),
            namespace: "default".to_string(),
        };

        let mut base_health = self.base.base_health_check(&container).await?;
        let mut openclaw_checks = HashMap::new();

        let model_validation_test = self.validate_model("claude-sonnet-4-20250514").await;
        openclaw_checks.insert(
            "model_validation".to_string(),
            json!(model_validation_test.is_ok()),
        );

        let test_config = AgentConfig {
            github_app: "test-app".to_string(),
            cli: "openclaw".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            tools: None,
            cli_config: None,
        };

        let config_generation_test = self.generate_config(&test_config).await;
        openclaw_checks.insert(
            "config_generation".to_string(),
            json!(config_generation_test.is_ok()),
        );

        let response_parsing_test = self.parse_response("{}").await;
        openclaw_checks.insert(
            "response_parsing".to_string(),
            json!(response_parsing_test.is_ok()),
        );

        for (key, value) in openclaw_checks {
            base_health.details.insert(key, value);
        }

        let all_checks_passed = base_health
            .details
            .values()
            .all(|v| v.as_bool().unwrap_or(false));

        if !all_checks_passed && base_health.status == HealthState::Healthy {
            base_health.status = HealthState::Warning;
            base_health.message = Some("Some OpenClaw-specific checks failed".to_string());
        }

        info!(status = ?base_health.status, "OpenClaw adapter health check completed");
        Ok(base_health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openclaw_adapter_creation() {
        let adapter = OpenClawAdapter::new().unwrap();
        assert_eq!(adapter.get_executable_name(), "openclaw");
        assert_eq!(adapter.get_memory_filename(), "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_openclaw_capabilities() {
        let adapter = OpenClawAdapter::new().unwrap();
        let capabilities = adapter.get_capabilities();

        assert!(!capabilities.supports_streaming);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_system_prompts);
        assert_eq!(capabilities.max_context_tokens, 200_000);
        assert_eq!(capabilities.config_format, ConfigFormat::Json);
        assert!(capabilities
            .authentication_methods
            .contains(&AuthMethod::ApiKey));
    }

    #[tokio::test]
    async fn test_model_validation_accepts_all() {
        let adapter = OpenClawAdapter::new().unwrap();

        assert!(adapter
            .validate_model("claude-sonnet-4-20250514")
            .await
            .unwrap());
        assert!(adapter.validate_model("gpt-5.2").await.unwrap());
        assert!(adapter
            .validate_model("gemini-3.1-pro-preview")
            .await
            .unwrap());
        assert!(adapter.validate_model("any-model").await.unwrap());

        assert!(!adapter.validate_model("").await.unwrap());
        assert!(!adapter.validate_model("   ").await.unwrap());
    }
}
