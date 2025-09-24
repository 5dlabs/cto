# Task 7: Implement Claude and Codex CLI Adapters

## Overview
Create production-ready adapters for Claude (reference implementation) and Codex (TOML configuration) to validate multi-CLI architecture. These adapters serve as the foundation and validation for the entire CLI adapter system.

## Technical Specification

### 1. Claude Adapter (Reference)
```rust
pub struct ClaudeAdapter {
    base: BaseAdapter,
    model_validator: Arc<ClaudeModelValidator>,
    config_template: Template,
}

impl CliAdapter for ClaudeAdapter {
    async fn generate_config(&self, config: &AgentConfig) -> Result<String> {
        let context = json!({
            "model": config.model,
            "max_tokens": config.max_tokens.unwrap_or(4096),
            "tools": self.generate_mcp_config(&config.tools)?,
        });

        self.base.render_template("claude/config.json.hbs", &context)
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            max_context_tokens: 200_000,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
        }
    }
}
```

### 2. Codex Adapter (TOML Configuration)
```rust
pub struct CodexAdapter {
    base: BaseAdapter,
    model_validator: Arc<OpenAIModelValidator>,
    oauth_client: Arc<OAuthClient>,
}

impl CliAdapter for CodexAdapter {
    async fn generate_config(&self, config: &AgentConfig) -> Result<String> {
        let context = json!({
            "model": config.model,
            "provider": "openai",
            "sandbox_preset": config.sandbox_preset.unwrap_or("workspace-write"),
            "model_reasoning_effort": config.reasoning_effort.unwrap_or("medium"),
        });

        self.base.render_template("codex/config.toml.hbs", &context)
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: false, // STDIO-based
            supports_multimodal: false,
            supports_function_calling: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Toml,
        }
    }
}
```

## Implementation Steps

### Phase 1: Claude Adapter
1. Implement complete CliAdapter trait for Claude
2. Maintain 100% backward compatibility
3. Add comprehensive test coverage
4. Performance optimization and caching

### Phase 2: Codex Adapter
1. Implement Codex-specific features (TOML, OAuth)
2. Add sandbox presets and reasoning effort
3. AGENTS.md memory file support
4. Authentication flow integration

### Phase 3: Integration and Testing
1. Integration tests with real CLI binaries
2. Configuration generation validation
3. Error handling and recovery testing
4. Performance benchmarking

## Success Criteria
- Claude adapter maintains exact backward compatibility
- Codex adapter supports all TOML configuration features
- Both adapters pass comprehensive integration tests
- Authentication flows work end-to-end
- Memory file operations (read/write/update) function correctly
- Performance matches or exceeds direct CLI usage