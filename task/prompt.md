# AI Agent Prompt: Design and Implement CLI Adapter Trait System

You are a senior Rust architect specializing in trait-based system design, async programming, and multi-provider abstraction layers. Your mission is to create the foundational CLI adapter system that will unify interactions with 8 different CLI tools while preserving their unique capabilities.

## Your Critical Mission
Design and implement the core abstraction layer that will serve as the foundation for the entire Multi-CLI Agent Platform. This trait system must elegantly handle the diverse requirements of different CLI providers while providing a consistent interface for the platform.

## Architectural Challenge
You must create a system that accommodates radically different CLI characteristics:
- **Claude**: JSON config, CLAUDE.md memory, streaming support, native MCP
- **Codex**: TOML config, AGENTS.md memory, STDIO-only MCP, Rust-based
- **Opencode**: JSON/JSONC config, TypeScript-based, multi-package architecture
- **Gemini**: JSON config, GEMINI.md memory, multimodal support, Google auth
- **Grok**: JSON config, .grok/GROK.md subdirectory pattern
- **Qwen**: Gemini fork, similar patterns but different models
- **Cursor**: Python-based, session memory, OAuth authentication
- **OpenHands**: Python framework, virtualenv, different tool patterns

## Core Technical Requirements

### 1. CliAdapter Trait Design
Create a comprehensive trait that abstracts CLI operations:
```rust
#[async_trait]
pub trait CliAdapter: Send + Sync + std::fmt::Debug {
    // Core validation and configuration
    async fn validate_model(&self, model: &str) -> Result<bool>;
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String>;

    // Prompt and response handling
    fn format_prompt(&self, prompt: &str) -> String;
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse>;

    // CLI-specific metadata
    fn get_memory_filename(&self) -> &str;
    fn get_executable_name(&self) -> &str;
    fn get_capabilities(&self) -> CliCapabilities;

    // Container lifecycle
    async fn initialize(&self, container: &Container) -> Result<()>;
    async fn cleanup(&self, container: &Container) -> Result<()>;
    async fn health_check(&self) -> Result<HealthStatus>;
}
```

### 2. Supporting Type System
Design comprehensive types that capture CLI diversity:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CLIType {
    Claude, Codex, Opencode, Gemini, Grok, Qwen, Cursor, OpenHands,
}

pub struct ParsedResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: ResponseMetadata,
    pub finish_reason: FinishReason,
    pub streaming_delta: Option<StreamingDelta>,
}

pub struct CliCapabilities {
    pub supports_streaming: bool,
    pub supports_multimodal: bool,
    pub supports_function_calling: bool,
    pub supports_system_prompts: bool,
    pub max_context_tokens: u32,
    pub memory_strategy: MemoryStrategy,
    pub config_format: ConfigFormat,
    pub authentication_methods: Vec<AuthMethod>,
}

pub enum MemoryStrategy {
    MarkdownFile(String),      // CLAUDE.md, AGENTS.md, GEMINI.md
    Subdirectory(String),      // .grok/GROK.md
    SessionBased,              // Cursor, OpenHands
    ConfigurationBased,        // Some CLIs use config for persistence
}
```

### 3. Base Adapter Implementation
Create shared functionality that all adapters can leverage:
```rust
pub struct BaseAdapter {
    pub cli_type: CLIType,
    pub config: AdapterConfig,
    pub metrics: Arc<AdapterMetrics>,
    pub logger: slog::Logger,
    pub tracer: Tracer,
}

impl BaseAdapter {
    // Common logging with structured data
    pub fn log_operation(&self, operation: &str, context: &HashMap<String, String>) {
        info!(self.logger, "CLI adapter operation";
            "cli_type" => ?self.cli_type,
            "operation" => operation,
            "context" => ?context
        );
    }

    // Metrics recording with OpenTelemetry
    pub async fn record_metrics(&self, operation: &str, duration: Duration, success: bool) {
        self.metrics.record_operation(self.cli_type, operation, duration, success).await;
    }

    // Common configuration validation
    pub fn validate_base_config(&self, config: &AgentConfig) -> Result<()> {
        // Shared validation logic
    }

    // Template rendering utilities
    pub fn render_template(&self, template: &str, context: &serde_json::Value) -> Result<String> {
        // Handlebars rendering with helpers
    }
}
```

### 4. Adapter Factory Pattern
Design a factory that manages adapter lifecycle and registration:
```rust
pub struct AdapterFactory {
    adapters: HashMap<CLIType, Arc<dyn CliAdapter>>,
    config_registry: ConfigRegistry,
    health_monitor: HealthMonitor,
}

impl AdapterFactory {
    pub async fn new() -> Result<Self> {
        let mut factory = Self {
            adapters: HashMap::new(),
            config_registry: ConfigRegistry::new(),
            health_monitor: HealthMonitor::new(),
        };

        // Register built-in adapters
        factory.register_adapter(CLIType::Claude, Arc::new(ClaudeAdapter::new().await?)).await?;

        Ok(factory)
    }

    pub async fn create(&self, cli_type: CLIType) -> Result<Arc<dyn CliAdapter>> {
        let adapter = self.adapters.get(&cli_type)
            .ok_or_else(|| anyhow!("Unsupported CLI type: {:?}", cli_type))?
            .clone();

        // Verify health before returning
        adapter.health_check().await?;

        Ok(adapter)
    }

    pub async fn register_adapter(&mut self, cli_type: CLIType, adapter: Arc<dyn CliAdapter>) -> Result<()> {
        // Validate adapter before registration
        self.validate_adapter(&adapter).await?;
        self.adapters.insert(cli_type, adapter);
        Ok(())
    }

    async fn validate_adapter(&self, adapter: &Arc<dyn CliAdapter>) -> Result<()> {
        // Comprehensive adapter validation
    }
}
```

## Implementation Strategy

### Phase 1: Trait Foundation
1. Design the `CliAdapter` trait with all required methods
2. Create comprehensive supporting types (CLIType, ParsedResponse, CliCapabilities)
3. Implement error types with detailed context for debugging
4. Add trait bounds and lifetime annotations for thread safety

### Phase 2: Base Infrastructure
1. Implement `BaseAdapter` with shared functionality
2. Add logging integration with structured data
3. Implement metrics collection with OpenTelemetry
4. Create template rendering system with Handlebars

### Phase 3: Factory System
1. Design `AdapterFactory` with registration and discovery
2. Add health monitoring and validation
3. Implement adapter lifecycle management
4. Create configuration management system

### Phase 4: Reference Implementation
1. Implement `ClaudeAdapter` as the gold standard
2. Ensure 100% backward compatibility with existing behavior
3. Add comprehensive documentation and examples
4. Create extensive test coverage

### Phase 5: Advanced Features
1. Add lifecycle hooks for pre/post execution
2. Implement distributed tracing with spans
3. Create circuit breaker pattern for resilience
4. Add configuration hot-reloading

## Critical Requirements

### Performance Specifications
- **Adapter Creation**: <50ms for any CLI type
- **Configuration Generation**: <100ms per agent config
- **Memory Usage**: <10MB per adapter instance
- **Concurrent Safety**: Support 1000+ simultaneous operations
- **Error Recovery**: Graceful degradation when adapters fail

### Compatibility Requirements
- **Claude Adapter**: Must behave identically to existing implementation
- **Future CLIs**: Extensible architecture for 8 different CLI types
- **Thread Safety**: All adapters must be Send + Sync
- **Async Support**: Full async/await patterns throughout

### Quality Standards
- **Error Handling**: Comprehensive error types with actionable messages
- **Logging**: Structured logging with correlation IDs
- **Testing**: >95% code coverage with integration tests
- **Documentation**: Complete rustdoc with examples

## Specific Adapter Implementations

### ClaudeAdapter (Reference)
```rust
#[derive(Debug)]
pub struct ClaudeAdapter {
    base: BaseAdapter,
    model_validator: Arc<ClaudeModelValidator>,
    config_template: Template,
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        self.model_validator.validate(model).await
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        let context = json!({
            "model": agent_config.model,
            "max_tokens": agent_config.max_tokens.unwrap_or(4096),
            "temperature": agent_config.temperature.unwrap_or(0.7),
            "tools": agent_config.tools,
            "mcp_servers": self.generate_mcp_config(&agent_config.tools)?,
        });

        self.base.render_template(&self.config_template, &context)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        format!("Human: {}\n\nAssistant: ", prompt)
    }

    async fn parse_response(&self, response: &str) -> Result<ParsedResponse> {
        // Parse Claude's response format
        // Handle tool calls, streaming deltas, etc.
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 200_000,
            memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".to_string()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::SessionToken],
        }
    }
}
```

### Extensibility Patterns
Design patterns that make adding new CLI adapters straightforward:

1. **Template-Driven Configuration**: Each adapter can define its config template
2. **Capability Declaration**: Self-describing adapter capabilities
3. **Plugin Architecture**: Dynamic adapter loading and registration
4. **Configuration Validation**: Per-CLI validation rules
5. **Error Context**: Rich error information for debugging

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_trait_completeness() {
        let adapter = ClaudeAdapter::new().await.unwrap();

        // Test all trait methods
        assert!(adapter.validate_model("claude-3-opus").await.unwrap());
        assert!(adapter.generate_config(&default_config()).await.is_ok());
        assert!(!adapter.get_memory_filename().is_empty());
    }

    #[tokio::test]
    async fn test_factory_adapter_creation() {
        let factory = AdapterFactory::new().await.unwrap();
        let adapter = factory.create(CLIType::Claude).await.unwrap();

        assert_eq!(adapter.get_executable_name(), "claude");
    }

    #[tokio::test]
    async fn test_concurrent_adapter_operations() {
        let factory = Arc::new(AdapterFactory::new().await.unwrap());
        let mut handles = vec![];

        for _ in 0..100 {
            let factory = factory.clone();
            handles.push(tokio::spawn(async move {
                let adapter = factory.create(CLIType::Claude).await.unwrap();
                adapter.validate_model("claude-3-opus").await.unwrap()
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

### Integration Testing
- Test adapter behavior with real CLI tools (mocked)
- Verify configuration generation produces valid configs
- Test lifecycle hooks execute in correct order
- Validate telemetry data collection

## Success Criteria
Your implementation succeeds when:
- ✅ CliAdapter trait elegantly abstracts all CLI differences
- ✅ ClaudeAdapter maintains perfect backward compatibility
- ✅ AdapterFactory efficiently manages adapter lifecycle
- ✅ BaseAdapter provides valuable shared functionality
- ✅ All code is thread-safe and performance-optimized
- ✅ Comprehensive telemetry and observability
- ✅ Extensible architecture for easy CLI additions
- ✅ Production-ready error handling and recovery
- ✅ Complete test coverage with benchmarks

## Constraints and Considerations
- Maintain zero-cost abstractions where possible
- Use async/await patterns throughout
- Ensure all adapters are Send + Sync for thread safety
- Design for hot-swappable adapter implementations
- Plan for configuration changes without restarts
- Consider memory usage with many concurrent adapters
- Design error types for excellent developer experience

## Deliverables
1. Complete CliAdapter trait with all required methods
2. Comprehensive type system for CLI abstraction
3. BaseAdapter with shared functionality and utilities
4. AdapterFactory with lifecycle management
5. ClaudeAdapter reference implementation
6. Telemetry and observability integration
7. Comprehensive test suite with benchmarks
8. Documentation and examples for new adapter creation

This trait system is the architectural foundation that enables the entire Multi-CLI Agent Platform. Every subsequent CLI integration depends on getting this abstraction right. Focus on elegance, performance, and extensibility.