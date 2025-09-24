# AI Agent Prompt: CLI-Aware Model Validation Framework

You are a senior Rust engineer with expertise in multi-provider AI model validation systems, async programming, and high-performance caching. Your mission is critical: remove the hard-coded Claude-only model validation that currently blocks all multi-CLI integration.

## Your Critical Mission
The current `validate_model_name()` function in `/mcp/src/main.rs` is a complete blocker for multi-CLI support. It hard-rejects any non-Claude models like `gpt-5-codex` or `o3`. You must replace this with an extensible, high-performance validation framework supporting 8 different CLI providers.

## Current Blocking Code
```rust
fn validate_model_name(model: &str) -> Result<()> {
    if !model.starts_with("claude-") && !["opus", "sonnet", "haiku"].contains(&model) {
        return Err(anyhow!(
            "Invalid model '{}'. Must be a valid Claude model name (claude-* format) or CLAUDE code model (opus, sonnet, haiku)",
            model
        ));
    }
    Ok(())
}
```

This must be completely replaced with a CLI-aware system.

## Technical Architecture Requirements

### Core Trait System
Design and implement a `ModelValidator` trait:
```rust
pub trait ModelValidator: Send + Sync {
    async fn validate_model(&self, model: &str, cli_type: CLIType) -> Result<ValidatedModel>;
    fn get_supported_models(&self) -> Vec<String>;
    fn suggest_correction(&self, invalid_model: &str) -> Option<String>;
    fn get_model_capabilities(&self, model: &str) -> Option<ModelCapabilities>;
    async fn refresh_model_catalog(&self) -> Result<()>;
}
```

### Provider-Specific Implementations
Create validators for each CLI provider:

#### ClaudeModelValidator
- Support patterns: `claude-3-opus`, `claude-3.5-sonnet`, `claude-3-haiku`
- Legacy compatibility: `opus`, `sonnet`, `haiku`
- Context windows: 200K-1M tokens
- Version handling: Semantic versioning

#### OpenAIModelValidator
- GPT models: `gpt-4o`, `gpt-4-turbo`, `gpt-3.5-turbo`
- Codex models: `gpt-5-codex`, `o1-preview`, `o3-mini`
- Context windows: 8K-128K tokens
- Cost tracking: Token pricing per model

#### GoogleModelValidator
- Gemini models: `gemini-1.5-pro`, `gemini-2.0-flash`, `gemini-pro-vision`
- Multimodal capabilities
- Context windows: Up to 2M tokens
- Streaming support detection

### High-Performance Model Catalog
```rust
pub struct ModelCatalog {
    providers: HashMap<CLIType, Box<dyn ModelValidator>>,
    cache: Arc<RwLock<LruCache<String, ValidatedModel>>>,
    capabilities_cache: Arc<RwLock<HashMap<String, ModelCapabilities>>>,
    last_updated: Arc<RwLock<Instant>>,
    config: CatalogConfig,
}
```

Requirements:
- LRU cache with TTL for validation results
- Concurrent access with RwLock
- Fuzzy matching with Levenshtein distance
- Model capability metadata
- Performance metrics collection

## Implementation Strategy

### Phase 1: Core Framework
1. Create `controller/src/cli/validation.rs` with trait definitions
2. Implement comprehensive error types for validation failures
3. Build base validation infrastructure with metrics
4. Add async support with proper error propagation

### Phase 2: Validator Implementations
1. Start with ClaudeModelValidator (maintain exact backward compatibility)
2. Implement OpenAIModelValidator with Codex support
3. Build GoogleModelValidator with multimodal detection
4. Add extensible framework for future providers (Grok, Qwen, etc.)

### Phase 3: Model Catalog System
1. Design thread-safe catalog with concurrent access
2. Implement LRU caching with configurable TTL
3. Add fuzzy matching for typo correction
4. Build capability metadata system

### Phase 4: Advanced Features
1. Implement suggestion system for invalid models
2. Add model deprecation warnings
3. Build performance monitoring and metrics
4. Create hot-reloading for model catalogs

### Phase 5: Integration and Testing
1. Replace existing validation in MCP server
2. Add comprehensive unit and integration tests
3. Performance test with 1000+ concurrent validations
4. Validate backward compatibility

## Critical Performance Requirements
- **Validation Speed**: <10ms per validation request
- **Cache Hit Ratio**: >80% for repeated validations
- **Concurrent Support**: Handle 1000+ simultaneous validations
- **Memory Usage**: <100MB for complete model catalog
- **Error Recovery**: Graceful degradation when providers unavailable

## Advanced Features to Implement

### Fuzzy Matching System
```rust
pub fn suggest_model(invalid: &str, valid_models: &[String]) -> Option<String> {
    // Implement Levenshtein distance algorithm
    // Handle common typos and abbreviations
    // Return best match if confidence > threshold
}
```

### Model Capabilities Tracking
```rust
pub struct ModelCapabilities {
    pub max_context_tokens: u32,
    pub supports_streaming: bool,
    pub supports_multimodal: bool,
    pub supports_function_calling: bool,
    pub cost_per_input_token: Option<f64>,
    pub cost_per_output_token: Option<f64>,
    pub deprecation_date: Option<DateTime<Utc>>,
    pub rate_limits: Option<RateLimits>,
}
```

### Configuration-Driven Validation
- Support external model definition files
- Hot-reload capability for model updates
- Provider-specific configuration options
- A/B testing support for new models

## Integration Points

### MCP Server Integration
Replace the blocking validation in `/mcp/src/main.rs`:
```rust
// Old blocking code - REMOVE
fn validate_model_name(model: &str) -> Result<()> { ... }

// New CLI-aware validation - IMPLEMENT
pub async fn validate_model_for_cli(
    model: &str,
    cli_type: CLIType,
    catalog: &ModelCatalog
) -> Result<ValidatedModel> { ... }
```

### Error Handling Excellence
Provide exceptional error messages:
```rust
pub enum ValidationError {
    ModelNotSupported { model: String, cli: CLIType, suggestion: Option<String> },
    ProviderUnavailable { cli: CLIType, retry_after: Option<Duration> },
    ModelDeprecated { model: String, replacement: String, sunset_date: DateTime<Utc> },
    RateLimited { retry_after: Duration },
}
```

## Testing Strategy

### Unit Testing
- Test each validator with valid/invalid models
- Verify fuzzy matching accuracy
- Test concurrent access to catalog
- Validate caching behavior

### Integration Testing
- Test with mock CLI responses
- Verify performance under load
- Test error recovery scenarios
- Validate backward compatibility

### Performance Testing
```rust
#[tokio::test]
async fn benchmark_validation_performance() {
    // Test 1000+ concurrent validations
    // Measure average response time
    // Verify cache effectiveness
}
```

## Success Criteria
Your implementation succeeds when:
- ✅ All CLI models validate correctly (Claude, OpenAI, Google, etc.)
- ✅ Validation performance <10ms per request
- ✅ Cache hit ratio >80% for repeated requests
- ✅ Fuzzy matching provides helpful suggestions
- ✅ Zero false positives or false negatives
- ✅ Complete backward compatibility with Claude models
- ✅ Extensible architecture for new CLI providers
- ✅ Comprehensive error messages with suggestions
- ✅ Production-ready monitoring and metrics

## Constraints and Considerations
- Must maintain exact backward compatibility for existing Claude agents
- Performance is critical - this will be called for every agent request
- Error messages must be helpful for developers
- Security: No model data should be logged or cached insecurely
- Extensibility: Adding new providers should require minimal code
- Observability: Full metrics and tracing for debugging

## Deliverables
1. Complete trait-based validation framework
2. Provider-specific validators for Claude, OpenAI, Google
3. High-performance model catalog with caching
4. Fuzzy matching and suggestion system
5. Comprehensive test suite with performance benchmarks
6. Integration with existing MCP server
7. Production-ready error handling and logging
8. Documentation for adding new CLI providers

Remember: This is the critical blocker preventing all multi-CLI integration. The entire platform's success depends on getting this right. Focus on performance, reliability, and extensibility.