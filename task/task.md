# Task 3: Design and Implement CLI Adapter Trait System

## Overview
Create the core abstraction layer with CliAdapter trait and base implementations for unified CLI interaction patterns. This task establishes the foundational interface that all CLI providers will implement, enabling consistent behavior across 8 different CLI tools while handling their unique requirements.

## Context
With the model validation framework in place (Task 2), we now need the core abstraction layer that will unify interactions with different CLI providers. Each CLI has unique characteristics - Codex uses TOML configuration, Claude uses markdown memory, Gemini supports multimodal inputs - but our platform must provide a consistent interface for all of them.

## Technical Specification

### 1. Core CliAdapter Trait
```rust
#[async_trait]
pub trait CliAdapter: Send + Sync + std::fmt::Debug {
    // Model and validation
    async fn validate_model(&self, model: &str) -> Result<bool>;

    // Configuration management
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String>;

    // Prompt formatting
    fn format_prompt(&self, prompt: &str) -> String;

    // Response handling
    async fn parse_response(&self, response: &str) -> Result<ParsedResponse>;

    // CLI-specific information
    fn get_memory_filename(&self) -> &str;
    fn get_executable_name(&self) -> &str;

    // Lifecycle management
    async fn initialize(&self, container: &Container) -> Result<()>;
    async fn cleanup(&self, container: &Container) -> Result<()>;

    // Health and status
    async fn health_check(&self) -> Result<HealthStatus>;

    // Capabilities
    fn get_capabilities(&self) -> CliCapabilities;
}
```

### 2. Supporting Types and Enums

#### CLI Type Enumeration
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CLIType {
    Claude,
    Codex,
    Opencode,
    Gemini,
    Grok,
    Qwen,
    Cursor,
    OpenHands,
}
```

#### Parsed Response Structure
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub metadata: ResponseMetadata,
    pub finish_reason: FinishReason,
}
```

#### CLI Capabilities
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CliCapabilities {
    pub supports_streaming: bool,
    pub supports_multimodal: bool,
    pub supports_function_calling: bool,
    pub supports_system_prompts: bool,
    pub max_context_tokens: u32,
    pub memory_strategy: MemoryStrategy,
    pub config_format: ConfigFormat,
}
```

### 3. Base Adapter Implementation
```rust
pub struct BaseAdapter {
    pub cli_type: CLIType,
    pub config: AdapterConfig,
    pub metrics: Arc<AdapterMetrics>,
    pub logger: slog::Logger,
}

impl BaseAdapter {
    // Shared functionality for all adapters
    fn log_request(&self, operation: &str, model: &str) {
        info!(self.logger, "CLI operation";
            "operation" => operation,
            "cli_type" => ?self.cli_type,
            "model" => model
        );
    }

    async fn record_metrics(&self, operation: &str, duration: Duration, success: bool) {
        self.metrics.record_operation(
            self.cli_type,
            operation,
            duration,
            success
        ).await;
    }

    fn validate_config(&self, config: &AgentConfig) -> Result<()> {
        // Common validation logic
    }
}
```

### 4. Adapter Factory Pattern
```rust
pub struct AdapterFactory {
    adapters: HashMap<CLIType, Arc<dyn CliAdapter>>,
    config_registry: ConfigRegistry,
}

impl AdapterFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            adapters: HashMap::new(),
            config_registry: ConfigRegistry::new(),
        };

        // Register built-in adapters
        factory.register_adapter(CLIType::Claude, Arc::new(ClaudeAdapter::new()));
        factory.register_adapter(CLIType::Codex, Arc::new(CodexAdapter::new()));
        factory.register_adapter(CLIType::Opencode, Arc::new(OpencodeAdapter::new()));
        factory.register_adapter(CLIType::Gemini, Arc::new(GeminiAdapter::new()));

        factory
    }

    pub fn create(&self, cli_type: CLIType) -> Result<Arc<dyn CliAdapter>> {
        self.adapters.get(&cli_type)
            .cloned()
            .ok_or_else(|| anyhow!("Unsupported CLI type: {:?}", cli_type))
    }

    pub fn register_adapter(&mut self, cli_type: CLIType, adapter: Arc<dyn CliAdapter>) {
        self.adapters.insert(cli_type, adapter);
    }

    pub fn get_supported_clis(&self) -> Vec<CLIType> {
        self.adapters.keys().cloned().collect()
    }
}
```

### 5. CLI-Specific Adapter Examples

#### Claude Adapter (Reference Implementation)
```rust
#[derive(Debug)]
pub struct ClaudeAdapter {
    base: BaseAdapter,
    model_validator: ClaudeModelValidator,
}

#[async_trait]
impl CliAdapter for ClaudeAdapter {
    async fn validate_model(&self, model: &str) -> Result<bool> {
        self.model_validator.validate(model).await
    }

    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        let template = include_str!("../templates/claude-config.json.hbs");
        let context = json!({
            "model": agent_config.model,
            "max_tokens": agent_config.max_tokens,
            "temperature": agent_config.temperature,
            "tools": agent_config.tools,
        });
        render_template(template, &context)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        // Claude-specific prompt formatting
        format!("Human: {}\n\nAssistant: ", prompt)
    }

    fn get_memory_filename(&self) -> &str {
        "CLAUDE.md"
    }

    fn get_executable_name(&self) -> &str {
        "claude"
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
        }
    }
}
```

#### Codex Adapter (TOML Configuration)
```rust
#[derive(Debug)]
pub struct CodexAdapter {
    base: BaseAdapter,
    model_validator: OpenAIModelValidator,
}

#[async_trait]
impl CliAdapter for CodexAdapter {
    async fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        let template = include_str!("../templates/codex-config.toml.hbs");
        let context = json!({
            "model": agent_config.model,
            "provider": "openai",
            "max_tokens": agent_config.max_tokens,
            "temperature": agent_config.temperature,
        });
        render_template(template, &context)
    }

    fn get_memory_filename(&self) -> &str {
        "AGENTS.md"
    }

    fn get_executable_name(&self) -> &str {
        "codex"
    }

    fn get_capabilities(&self) -> CliCapabilities {
        CliCapabilities {
            supports_streaming: false, // STDIO-based
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".to_string()),
            config_format: ConfigFormat::Toml,
        }
    }
}
```

### 6. Lifecycle Management and Telemetry

#### Lifecycle Hooks
```rust
pub trait LifecycleHooks {
    async fn pre_execution(&self, context: &ExecutionContext) -> Result<()>;
    async fn post_execution(&self, context: &ExecutionContext, result: &ExecutionResult) -> Result<()>;
    async fn on_error(&self, context: &ExecutionContext, error: &AdapterError) -> Result<()>;
}
```

#### Telemetry Integration
```rust
pub struct AdapterTelemetry {
    tracer: Tracer,
    meter: Meter,
    counters: AdapterCounters,
    histograms: AdapterHistograms,
}

impl AdapterTelemetry {
    pub async fn record_operation(&self, cli_type: CLIType, operation: &str, duration: Duration, success: bool) {
        // OpenTelemetry metrics recording
        self.counters.operations_total
            .add(1, &[KeyValue::new("cli_type", cli_type.as_str()),
                     KeyValue::new("operation", operation.to_string()),
                     KeyValue::new("success", success.to_string())]);

        self.histograms.operation_duration
            .record(duration.as_millis() as f64, &[
                KeyValue::new("cli_type", cli_type.as_str()),
                KeyValue::new("operation", operation.to_string())
            ]);
    }

    pub fn create_span(&self, operation: &str) -> Span {
        self.tracer.start(&format!("cli_adapter_{}", operation))
    }
}
```

## Implementation Steps

### Phase 1: Core Trait Design
1. Define the `CliAdapter` trait with all required methods
2. Create supporting types and enums (CLIType, ParsedResponse, etc.)
3. Implement error types for adapter operations
4. Add comprehensive documentation and examples

### Phase 2: Base Implementation
1. Create `BaseAdapter` with shared functionality
2. Implement logging, metrics, and common utilities
3. Add configuration validation and helper methods
4. Create adapter configuration system

### Phase 3: Adapter Factory
1. Design and implement the `AdapterFactory` pattern
2. Add adapter registration and discovery
3. Implement dynamic adapter creation
4. Add configuration management integration

### Phase 4: Reference Implementation
1. Implement `ClaudeAdapter` as the reference implementation
2. Ensure complete backward compatibility with existing behavior
3. Add comprehensive test coverage
4. Document implementation patterns

### Phase 5: Lifecycle and Telemetry
1. Add lifecycle hooks for pre/post execution
2. Implement telemetry with OpenTelemetry integration
3. Add health checking and status reporting
4. Create monitoring dashboards and alerts

## Dependencies
- Task 1: Project structure for module organization
- Task 2: Model validation framework for validator integration
- Kubernetes client libraries for container management
- OpenTelemetry for distributed tracing and metrics
- Handlebars for template rendering
- Async runtime (tokio)

## Success Criteria
- All trait methods are properly defined and documented
- BaseAdapter provides consistent shared functionality
- AdapterFactory creates correct adapter instances
- ClaudeAdapter maintains exact backward compatibility
- Comprehensive telemetry and monitoring integration
- Thread-safe concurrent adapter operations
- Extensible architecture for new CLI additions

## Files Created
```
controller/src/cli/
├── adapter.rs (main trait and types)
├── base_adapter.rs (shared implementation)
├── factory.rs (adapter factory pattern)
├── lifecycle.rs (lifecycle management)
├── telemetry.rs (metrics and tracing)
└── adapters/
    ├── claude.rs (reference implementation)
    ├── codex.rs (stub for Task 4)
    ├── opencode.rs (stub for future)
    ├── gemini.rs (stub for future)
    └── mod.rs

controller/src/cli/templates/
├── claude-config.json.hbs
├── codex-config.toml.hbs
└── base-template-helpers.rs

tests/
├── adapter_tests.rs
├── factory_tests.rs
└── integration/
    └── cli_adapter_integration.rs
```

## Risk Mitigation

### Backward Compatibility
- ClaudeAdapter must behave identically to existing code
- Comprehensive regression testing
- Feature flags for gradual rollout
- Rollback capabilities

### Performance
- Async-first design for non-blocking operations
- Efficient trait object dispatch
- Memory pool for frequent allocations
- Connection pooling where applicable

### Extensibility
- Plugin-based architecture for new CLI adapters
- Configuration-driven adapter behavior
- Version-aware adapter interfaces
- Hot-swappable adapter implementations

## Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_factory_creation() {
        let factory = AdapterFactory::new();
        let claude_adapter = factory.create(CLIType::Claude).unwrap();

        assert_eq!(claude_adapter.get_executable_name(), "claude");
    }

    #[tokio::test]
    async fn test_claude_adapter_config_generation() {
        let adapter = ClaudeAdapter::new();
        let config = AgentConfig::default();

        let result = adapter.generate_config(&config).await.unwrap();
        assert!(result.contains("claude"));
    }

    #[tokio::test]
    async fn test_lifecycle_hooks() {
        let adapter = ClaudeAdapter::new();
        let context = ExecutionContext::new();

        adapter.pre_execution(&context).await.unwrap();
        // Test execution
        adapter.post_execution(&context, &result).await.unwrap();
    }
}
```

## Next Steps
After completion, this task enables:
- Task 4: Codex CLI Integration (implements CodexAdapter)
- Task 5: Opencode CLI Integration (implements OpencodeAdapter)
- Task 6+: Additional CLI integrations
- Dynamic adapter loading and hot-swapping
- Advanced adapter features (streaming, multimodal)

This task establishes the architectural foundation that all subsequent CLI integrations will build upon.