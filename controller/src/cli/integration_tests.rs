//! Integration tests for the CLI Adapter Trait System
//!
//! These tests demonstrate the complete functionality of the adapter system
//! including trait compliance, factory management, and adapter interoperability.

#[cfg(test)]
mod tests {
    use crate::cli::trait_adapter::*;
    use crate::cli::base_adapter::{BaseAdapter, AdapterConfig, AdapterMetrics};
    use crate::cli::factory::{AdapterFactory, FactoryConfig};
    use crate::cli::adapters::{ClaudeAdapter, CodexAdapter};
    use crate::cli::types::CLIType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use futures::future;

    // Mock container implementation for testing
    #[derive(Debug)]
    struct TestContainer {
        id: String,
        working_dir: String,
        env_vars: HashMap<String, String>,
        files: Arc<RwLock<HashMap<String, String>>>,
    }

    impl TestContainer {
        fn new() -> Self {
            Self {
                id: "test-container-123".to_string(),
                working_dir: "/workspace/test".to_string(),
                env_vars: {
                    let mut env = HashMap::new();
                    env.insert("CLAUDE_API_KEY".to_string(), "test-key".to_string());
                    env.insert("OPENAI_API_KEY".to_string(), "test-key".to_string());
                    env
                },
                files: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl Container for TestContainer {
        fn id(&self) -> &str {
            &self.id
        }

        fn working_dir(&self) -> &str {
            &self.working_dir
        }

        fn env_vars(&self) -> &HashMap<String, String> {
            &self.env_vars
        }

        async fn execute(&self, command: &[String]) -> Result<ExecutionResult, String> {
            // Simulate command execution
            let cmd = command.join(" ");

            let (exit_code, stdout, stderr) = match cmd.as_str() {
                c if c.contains("claude-code --version") => {
                    (Some(0), "claude-code v1.0.0".to_string(), "".to_string())
                }
                c if c.contains("codex --version") => {
                    (Some(0), "codex v0.7.2".to_string(), "".to_string())
                }
                _ => (Some(0), "OK".to_string(), "".to_string()),
            };

            Ok(ExecutionResult {
                exit_code,
                stdout,
                stderr,
                duration: Duration::from_millis(100),
            })
        }

        async fn create_file(&self, path: &str, content: &str) -> Result<(), String> {
            let mut files = self.files.write().await;
            files.insert(path.to_string(), content.to_string());
            Ok(())
        }

        async fn read_file(&self, path: &str) -> Result<String, String> {
            let files = self.files.read().await;
            files.get(path).cloned().ok_or_else(|| "File not found".to_string())
        }
    }

    #[tokio::test]
    async fn test_complete_adapter_lifecycle() {
        // Test the complete adapter lifecycle from factory creation to cleanup

        // 1. Create factory
        let factory = AdapterFactory::new().await.unwrap();
        assert!(factory.supports_cli(CLIType::Claude).await == false); // No adapters registered yet

        // 2. Create and register Claude adapter
        let claude_adapter = Arc::new(ClaudeAdapter::new().await.unwrap());
        factory.register_adapter(CLIType::Claude, claude_adapter.clone()).await.unwrap();
        assert!(factory.supports_cli(CLIType::Claude).await);

        // 3. Create adapter instance from factory
        let adapter_instance = factory.create(CLIType::Claude).await.unwrap();
        assert_eq!(adapter_instance.get_executable_name(), "claude-code");

        // 4. Test adapter capabilities
        let capabilities = adapter_instance.get_capabilities();
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_function_calling);
        assert_eq!(capabilities.max_context_tokens, 200_000);

        // 5. Test configuration generation
        let agent_config = AgentConfig {
            model: "claude-3-5-sonnet".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            ..Default::default()
        };

        let config = adapter_instance.generate_config(&agent_config).await.unwrap();
        assert!(config.contains("claude-3-5-sonnet"));
        assert!(config.contains("4096"));

        // 6. Test container lifecycle
        let container = TestContainer::new();
        adapter_instance.initialize(&container).await.unwrap();

        // Verify CLAUDE.md was created
        let claude_md = container.read_file("/workspace/test/CLAUDE.md").await.unwrap();
        assert!(claude_md.contains("# Claude Code Project Memory"));

        adapter_instance.cleanup(&container).await.unwrap();

        // 7. Test health check
        let health = adapter_instance.health_check().await.unwrap();
        assert!(matches!(
            health,
            HealthStatus::Healthy | HealthStatus::Degraded(_) | HealthStatus::Unhealthy(_)
        ));
    }

    #[tokio::test]
    async fn test_multi_cli_factory() {
        // Test factory with multiple CLI adapters

        let factory = AdapterFactory::new().await.unwrap();

        // Register multiple adapters
        let claude_adapter = Arc::new(ClaudeAdapter::new().await.unwrap());
        let codex_adapter = Arc::new(CodexAdapter::new().await.unwrap());

        factory.register_adapter(CLIType::Claude, claude_adapter).await.unwrap();
        factory.register_adapter(CLIType::Codex, codex_adapter).await.unwrap();

        // Test that both are supported
        assert!(factory.supports_cli(CLIType::Claude).await);
        assert!(factory.supports_cli(CLIType::Codex).await);

        let supported_clis = factory.get_supported_clis().await;
        assert!(supported_clis.contains(&CLIType::Claude));
        assert!(supported_clis.contains(&CLIType::Codex));

        // Test different adapter characteristics
        let claude = factory.create(CLIType::Claude).await.unwrap();
        let codex = factory.create(CLIType::Codex).await.unwrap();

        assert_eq!(claude.get_memory_filename(), "CLAUDE.md");
        assert_eq!(codex.get_memory_filename(), "AGENTS.md");

        assert_eq!(claude.get_executable_name(), "claude-code");
        assert_eq!(codex.get_executable_name(), "codex");

        // Test different configuration formats
        let claude_caps = claude.get_capabilities();
        let codex_caps = codex.get_capabilities();

        assert!(matches!(claude_caps.config_format, ConfigFormat::Json));
        assert!(matches!(codex_caps.config_format, ConfigFormat::Toml));
    }

    #[tokio::test]
    async fn test_model_validation_across_adapters() {
        // Test model validation for different CLI types

        let claude_adapter = ClaudeAdapter::new().await.unwrap();
        let codex_adapter = CodexAdapter::new().await.unwrap();

        // Test Claude model validation
        assert!(claude_adapter.validate_model("claude-3-5-sonnet").await.unwrap());
        assert!(claude_adapter.validate_model("opus").await.unwrap());
        assert!(claude_adapter.validate_model("claude-anything").await.unwrap());

        // Test invalid Claude models
        let result = claude_adapter.validate_model("gpt-4").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Claude model"));

        // Test Codex model validation (stub implementation)
        assert!(codex_adapter.validate_model("gpt-4").await.unwrap());
        assert!(codex_adapter.validate_model("o3").await.unwrap());
        assert!(!codex_adapter.validate_model("claude-3-opus").await.unwrap());
    }

    #[tokio::test]
    async fn test_response_parsing() {
        // Test response parsing across different adapters

        let claude_adapter = ClaudeAdapter::new().await.unwrap();
        let codex_adapter = CodexAdapter::new().await.unwrap();

        let test_response = "This is a test response from the AI assistant.";

        // Test Claude response parsing
        let claude_parsed = claude_adapter.parse_response(test_response).await.unwrap();
        assert_eq!(claude_parsed.content, test_response);
        assert_eq!(claude_parsed.finish_reason, FinishReason::Stop);
        assert!(claude_parsed.metadata.id.is_some());
        assert!(claude_parsed.metadata.usage.is_some());

        // Test Codex response parsing (stub)
        let codex_parsed = codex_adapter.parse_response(test_response).await.unwrap();
        assert_eq!(codex_parsed.content, test_response);
        assert_eq!(codex_parsed.finish_reason, FinishReason::Stop);
        assert!(codex_parsed.metadata.id.is_some());
    }

    #[tokio::test]
    async fn test_prompt_formatting() {
        // Test prompt formatting for different CLI styles

        let claude_adapter = ClaudeAdapter::new().await.unwrap();
        let codex_adapter = CodexAdapter::new().await.unwrap();

        let raw_prompt = "Please help me with this coding task.";

        // Test Claude formatting
        let claude_formatted = claude_adapter.format_prompt(raw_prompt);
        assert!(claude_formatted.contains("Human:"));
        assert!(claude_formatted.contains("Assistant:"));

        // Test Codex formatting (stub - should be pass-through)
        let codex_formatted = codex_adapter.format_prompt(raw_prompt);
        assert_eq!(codex_formatted, raw_prompt);
    }

    #[tokio::test]
    async fn test_configuration_generation() {
        // Test configuration generation for different formats

        let claude_adapter = ClaudeAdapter::new().await.unwrap();
        let codex_adapter = CodexAdapter::new().await.unwrap();

        let agent_config = AgentConfig {
            model: "test-model".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.8),
            tools: ToolConfiguration {
                remote: vec!["memory_create_entities".to_string()],
                local_servers: {
                    let mut servers = HashMap::new();
                    servers.insert("filesystem".to_string(), LocalServerConfig {
                        enabled: true,
                        tools: vec!["read_file".to_string(), "write_file".to_string()],
                        config: HashMap::new(),
                    });
                    servers
                },
                mcp_servers: HashMap::new(),
            },
            ..Default::default()
        };

        // Test Claude config (JSON)
        let claude_config = claude_adapter.generate_config(&agent_config).await;
        // Claude should reject invalid model
        assert!(claude_config.is_err());

        // Test with valid Claude model
        let mut claude_agent_config = agent_config.clone();
        claude_agent_config.model = "claude-3-5-sonnet".to_string();
        let claude_config = claude_adapter.generate_config(&claude_agent_config).await.unwrap();
        assert!(claude_config.contains("claude-3-5-sonnet"));
        assert!(claude_config.contains("4096"));
        assert!(claude_config.contains("0.8"));

        // Test Codex config (TOML stub)
        let codex_config = codex_adapter.generate_config(&agent_config).await.unwrap();
        assert!(codex_config.contains("model = \"gpt-4\"")); // Stub always uses gpt-4
        assert!(codex_config.contains("toml"));
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        // Test health monitoring across adapters

        let factory = AdapterFactory::with_config(FactoryConfig {
            enable_health_monitoring: true,
            health_check_interval: Duration::from_secs(1),
            ..Default::default()
        }).await.unwrap();

        let claude_adapter = Arc::new(ClaudeAdapter::new().await.unwrap());
        let codex_adapter = Arc::new(CodexAdapter::new().await.unwrap());

        factory.register_adapter(CLIType::Claude, claude_adapter).await.unwrap();
        factory.register_adapter(CLIType::Codex, codex_adapter).await.unwrap();

        // Run health checks
        let health_results = factory.health_check_all().await;

        assert!(health_results.contains_key(&CLIType::Claude));
        assert!(health_results.contains_key(&CLIType::Codex));

        // Claude should be healthy or degraded
        let claude_health = health_results.get(&CLIType::Claude).unwrap();
        assert!(matches!(
            claude_health,
            HealthStatus::Healthy | HealthStatus::Degraded(_)
        ));

        // Codex (stub) should be degraded
        let codex_health = health_results.get(&CLIType::Codex).unwrap();
        assert!(matches!(codex_health, HealthStatus::Degraded(_)));
    }

    #[tokio::test]
    async fn test_factory_statistics() {
        // Test comprehensive factory statistics

        let factory = AdapterFactory::new().await.unwrap();

        // Register multiple adapters with different health states
        let claude_adapter = Arc::new(ClaudeAdapter::new().await.unwrap());
        let codex_adapter = Arc::new(CodexAdapter::new().await.unwrap());

        factory.register_adapter(CLIType::Claude, claude_adapter).await.unwrap();
        factory.register_adapter(CLIType::Codex, codex_adapter).await.unwrap();

        // Get factory stats
        let stats = factory.get_factory_stats().await;

        assert_eq!(stats.total_adapters, 2);
        assert!(stats.supported_clis.contains(&CLIType::Claude));
        assert!(stats.supported_clis.contains(&CLIType::Codex));

        // After health checks, we should have some health data
        factory.health_check_all().await;
        let updated_stats = factory.get_factory_stats().await;

        // Codex should be degraded (stub), Claude might be healthy/degraded
        assert!(updated_stats.degraded_adapters >= 1);
        assert_eq!(updated_stats.total_adapters, 2);
    }

    #[tokio::test]
    async fn test_adapter_error_handling() {
        // Test error handling and recovery

        let claude_adapter = ClaudeAdapter::new().await.unwrap();

        // Test invalid model validation
        let result = claude_adapter.validate_model("invalid-model").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AdapterError::ModelValidation(msg) => {
                assert!(msg.contains("Invalid Claude model"));
            }
            _ => panic!("Expected ModelValidation error"),
        }

        // Test empty model validation
        let result = claude_adapter.validate_model("").await;
        assert!(result.is_err());

        // Test invalid configuration
        let invalid_config = AgentConfig {
            model: "".to_string(), // Invalid empty model
            ..Default::default()
        };
        let result = claude_adapter.generate_config(&invalid_config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_adapter_operations() {
        // Test concurrent access to adapters

        let factory = Arc::new(AdapterFactory::new().await.unwrap());
        let claude_adapter = Arc::new(ClaudeAdapter::new().await.unwrap());

        factory.register_adapter(CLIType::Claude, claude_adapter).await.unwrap();

        // Spawn multiple concurrent operations
        let mut handles = Vec::new();

        for i in 0..10 {
            let factory = factory.clone();
            handles.push(tokio::spawn(async move {
                let adapter = factory.create(CLIType::Claude).await.unwrap();
                let result = adapter.validate_model(&format!("claude-3-test-{}", i)).await;
                result
            }));
        }

        // Wait for all to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should complete successfully (though validation may fail)
        for result in results {
            assert!(result.is_ok()); // The task itself should succeed
        }
    }

    #[tokio::test]
    async fn test_adapter_trait_compliance() {
        // Test that all adapters properly implement the CliAdapter trait

        let claude_adapter: Box<dyn CliAdapter> = Box::new(ClaudeAdapter::new().await.unwrap());
        let codex_adapter: Box<dyn CliAdapter> = Box::new(CodexAdapter::new().await.unwrap());

        let adapters = vec![claude_adapter, codex_adapter];

        for adapter in adapters {
            // Test that all trait methods are callable
            let capabilities = adapter.get_capabilities();
            assert!(capabilities.max_context_tokens > 0);
            assert!(!adapter.get_executable_name().is_empty());
            assert!(!adapter.get_memory_filename().is_empty() ||
                   matches!(capabilities.memory_strategy, MemoryStrategy::SessionBased));

            // Test health check
            let health = adapter.health_check().await.unwrap();
            assert!(matches!(
                health,
                HealthStatus::Healthy | HealthStatus::Degraded(_) | HealthStatus::Unhealthy(_)
            ));

            // Test prompt formatting
            let formatted = adapter.format_prompt("test prompt");
            assert!(!formatted.is_empty());

            // Test response parsing
            let parsed = adapter.parse_response("test response").await.unwrap();
            assert!(!parsed.content.is_empty());
        }
    }
}