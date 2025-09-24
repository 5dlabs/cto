# Task 2: Implement CLI-Aware Model Validation Framework

## Overview
Refactor the hard-coded `validate_model_name()` function to support all CLI providers with extensible validation patterns and comprehensive model catalog. This is a critical blocker that must be resolved before any multi-CLI integration work can proceed.

## Context
The current validation system in `/mcp/src/main.rs:validate_model_name()` is hard-coded for Claude models only, immediately rejecting any non-Claude models like `gpt-5-codex` or `o3`. This creates a complete blocker for integrating other CLI providers and must be addressed as the highest priority technical risk.

## Technical Specification

### 1. Current Validation Problem
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

### 2. New Trait-Based Architecture
```rust
pub trait ModelValidator {
    async fn validate_model(&self, model: &str, cli_type: CLIType) -> Result<ValidatedModel>;
    fn get_supported_models(&self) -> Vec<String>;
    fn suggest_correction(&self, invalid_model: &str) -> Option<String>;
    fn get_model_capabilities(&self, model: &str) -> Option<ModelCapabilities>;
}
```

### 3. Provider-Specific Validators

#### Claude Model Validator
- **Patterns**: `claude-3-opus`, `claude-3.5-sonnet`, `claude-3-haiku`
- **Legacy Support**: `opus`, `sonnet`, `haiku`
- **Version Handling**: Semantic versioning support
- **Context Windows**: 200K-1M tokens depending on model

#### OpenAI Model Validator
- **Patterns**: `gpt-4o`, `gpt-4-turbo`, `gpt-3.5-turbo`
- **Codex Models**: `gpt-5-codex`, `o1-preview`, `o3-mini`
- **Version Support**: Model version suffixes
- **Context Windows**: 8K-128K tokens

#### Google Model Validator
- **Patterns**: `gemini-1.5-pro`, `gemini-2.0-flash`, `gemini-pro-vision`
- **Multimodal Support**: Vision and text capabilities
- **Context Windows**: Up to 2M tokens
- **Streaming Support**: Real-time responses

### 4. Model Catalog System
```rust
pub struct ModelCatalog {
    providers: HashMap<CLIType, Box<dyn ModelValidator>>,
    cache: Arc<RwLock<LruCache<String, ValidatedModel>>>,
    last_updated: Instant,
}
```

Features:
- Registry of valid models per CLI
- Version support with deprecation warnings
- Capability metadata (context window, multimodal, streaming)
- Performance caching with TTL
- Fuzzy matching for typo correction

### 5. Enhanced Validation Features

#### Fuzzy Matching
- Levenshtein distance algorithm for typo detection
- Common abbreviation expansions
- Case-insensitive matching
- Suggestion system for invalid models

#### Model Capabilities
```rust
pub struct ModelCapabilities {
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_multimodal: bool,
    pub supports_tools: bool,
    pub cost_per_token: Option<f64>,
    pub deprecated: bool,
}
```

## Implementation Steps

### Phase 1: Core Framework
1. Create `controller/src/cli/validation.rs` module
2. Define `ModelValidator` trait with all required methods
3. Implement base validation functionality
4. Add comprehensive error types for validation failures

### Phase 2: Provider Validators
1. Implement `ClaudeModelValidator` with existing patterns
2. Create `OpenAIModelValidator` for GPT and Codex models
3. Build `GoogleModelValidator` for Gemini models
4. Add extensible framework for future providers

### Phase 3: Model Catalog
1. Design `ModelCatalog` struct with provider registry
2. Implement caching layer with LRU cache
3. Add model capability metadata system
4. Create fuzzy matching for error suggestions

### Phase 4: Integration
1. Replace existing `validate_model_name()` function
2. Update MCP server to use new validation system
3. Add configuration for model catalog updates
4. Implement telemetry for validation metrics

### Phase 5: Testing & Optimization
1. Comprehensive unit tests for all validators
2. Performance testing with concurrent validation
3. Integration testing with mock CLI responses
4. Load testing with 1000+ concurrent requests

## Dependencies
- Current MCP server codebase
- CLI type definitions (from Task 3)
- Error handling framework
- Caching library (LRU cache)
- Async runtime (tokio)

## Success Criteria
- All CLI models validate correctly
- Performance: <10ms per validation
- Fuzzy matching suggests correct alternatives
- Cache hit ratio >80% for repeated validations
- Zero false positives/negatives in validation
- Backward compatibility with existing Claude models

## Files Created/Modified
```
controller/src/cli/
├── validation.rs (new)
├── model_catalog.rs (new)
└── validators/
    ├── claude.rs (new)
    ├── openai.rs (new)
    ├── google.rs (new)
    └── mod.rs (new)

mcp/src/
├── main.rs (modified - replace validate_model_name)
└── validation/ (new)
    ├── types.rs (new)
    └── error.rs (new)
```

## Risk Mitigation

### Backward Compatibility
- Maintain exact same behavior for existing Claude models
- Add deprecation warnings before removing legacy support
- Provide migration path for configurations

### Performance
- Implement comprehensive caching strategy
- Use async validation where possible
- Add circuit breakers for external API calls
- Profile and optimize critical paths

### Extensibility
- Plugin-based architecture for new providers
- Configuration-driven model definitions
- Version-aware validation rules
- Hot-reloading of model catalogs

## Testing Strategy
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_claude_models() {
        // Test all Claude model patterns
    }

    #[tokio::test]
    async fn test_openai_models() {
        // Test GPT and Codex models
    }

    #[tokio::test]
    async fn test_fuzzy_matching() {
        // Test typo correction
    }

    #[tokio::test]
    async fn test_concurrent_validation() {
        // Performance under load
    }
}
```

## Next Steps
After completion enables:
- Task 3: CLI Adapter Trait System (can use validated models)
- All subsequent CLI integrations (Codex, Opencode, Gemini)
- Model capability-based routing
- Cost optimization based on model pricing

This task is the foundation that unblocks the entire multi-CLI platform development.