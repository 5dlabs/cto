# Task 2: Implement Flexible CLI Model Configuration

## Overview
Remove hardcoded model validation and implement a flexible configuration system that allows users to specify any model for any CLI. Each CLI will handle its own model validation, eliminating the need for maintaining hardcoded model lists in our codebase.

## Context
The current system has hardcoded Claude-only model validation that blocks other CLI providers. Instead of expanding this validation to include more hardcoded models, we should eliminate the validation entirely and let each CLI handle model compatibility on its own.

## Technical Specification

### 1. Current Validation Problem
The system currently rejects any non-Claude models:
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

### 2. New Pass-Through Architecture
Replace validation with flexible configuration:
```rust
pub struct CLIModelConfig {
    pub model_name: String,
    pub cli_type: CLIType,
    pub custom_parameters: HashMap<String, String>,
}

impl CLIModelConfig {
    pub fn new(model_name: String, cli_type: CLIType) -> Self {
        Self {
            model_name,
            cli_type,
            custom_parameters: HashMap::new(),
        }
    }
    
    // No validation - just pass through to CLI
    pub fn validate_format(&self) -> Result<()> {
        // Only validate configuration structure, not model names
        if self.model_name.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }
        Ok(())
    }
}
```

### 3. CLI-Native Model Handling
Instead of provider-specific validators, let each CLI handle its own models:

#### Philosophy: Trust the CLI
- **No Hardcoded Lists**: Zero model names in source code  
- **CLI Responsibility**: Each CLI validates its own supported models
- **User Freedom**: Users can configure any model identifier
- **Runtime Validation**: Model validity checked when CLI executes

#### Implementation Approach
```rust
pub struct CLIModelHandler {
    pub cli_type: CLIType,
    pub model_config: String,
    pub environment: HashMap<String, String>,
}

impl CLIModelHandler {
    pub fn new(cli_type: CLIType, model_config: String) -> Self {
        Self {
            cli_type,
            model_config,
            environment: HashMap::new(),
        }
    }
    
    // Pass through model config without validation
    pub fn prepare_cli_args(&self) -> Vec<String> {
        match self.cli_type {
            CLIType::Claude => vec!["--model".to_string(), self.model_config.clone()],
            CLIType::Codex => vec!["--model".to_string(), self.model_config.clone()],
            CLIType::Gemini => vec!["--model".to_string(), self.model_config.clone()],
            // Each CLI gets its model config passed through unchanged
            _ => vec!["--model".to_string(), self.model_config.clone()],
        }
    }
}
```

### 4. Configuration Management
Simple configuration structure without hardcoded models:
```rust
pub struct CLIConfiguration {
    pub default_models: HashMap<CLIType, String>,
    pub environment_overrides: HashMap<String, String>,
    pub cli_specific_params: HashMap<CLIType, HashMap<String, String>>,
}

impl CLIConfiguration {
    pub fn get_model_for_cli(&self, cli_type: CLIType) -> Option<&String> {
        // Check environment override first
        let env_key = format!("{}_MODEL", cli_type.as_str().to_uppercase());
        if let Ok(env_model) = std::env::var(&env_key) {
            return Some(&env_model);
        }
        
        // Fall back to default
        self.default_models.get(&cli_type)
    }
}
```

## Implementation Steps

### Phase 1: Remove Validation Logic
1. Locate and remove `validate_model_name()` function
2. Update all callers to accept any model string
3. Remove hardcoded model constants and lists
4. Add simple format validation (non-empty string only)

### Phase 2: Configuration Pass-Through
1. Create `CLIModelConfig` structure for flexible configuration
2. Implement model parameter passing to CLI commands
3. Add environment variable override support
4. Update configuration loading to accept any model names

### Phase 3: Error Handling
1. Remove model validation errors from our code
2. Capture and forward CLI-specific model errors
3. Implement proper error propagation from CLI processes
4. Add helpful error context without model validation

### Phase 4: Integration
1. Replace existing `validate_model_name()` function with pass-through
2. Update MCP server to accept any model configuration
3. Remove hardcoded model constants and validation logic
4. Test with various CLI model configurations

### Phase 5: Testing & Verification
1. Test that any model string is accepted in configuration
2. Verify model parameters are passed to CLI unchanged
3. Test environment variable override functionality
4. Verify CLI-specific error handling works properly

## Dependencies
- Current MCP server codebase (Rust-based)
- CLI type definitions
- Basic error handling framework
- Standard Rust HashMap for configuration storage

## Success Criteria
- Any model name is accepted in configuration
- Model strings are passed to CLI without modification
- Environment variables can override default models
- CLI-specific errors are properly forwarded
- Zero hardcoded model names in source code
- Backward compatibility with existing configurations maintained

## Files Created/Modified
```
mcp/src/
├── main.rs (modified - remove validate_model_name function)
└── config.rs (modified - add flexible model configuration)

controller/src/cli/
└── model_config.rs (new - simple model pass-through logic)
```

## Risk Mitigation

### Backward Compatibility
- Ensure all existing configurations continue working unchanged
- No breaking changes to configuration APIs
- Preserve CLI-specific behavior patterns

### Error Handling
- Proper error propagation from CLI tools to users
- Clear distinction between configuration errors and runtime errors
- Graceful handling of CLI-specific authentication failures

### Configuration Security
- Input validation for configuration format (not content)
- Protection against command injection in model parameters
- Secure handling of environment variable overrides

## Testing Strategy
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_any_model_accepted() {
        // Test that any model string is accepted
        let config = CLIModelConfig::new("any-model".to_string(), CLIType::Claude);
        assert!(config.validate_format().is_ok());
    }

    #[tokio::test]
    async fn test_model_passthrough() {
        // Test model parameters are passed unchanged to CLI
        let handler = CLIModelHandler::new(CLIType::Gemini, "custom-model".to_string());
        let args = handler.prepare_cli_args();
        assert!(args.contains(&"custom-model".to_string()));
    }

    #[tokio::test]
    async fn test_env_variable_override() {
        // Test environment variable model overrides
        std::env::set_var("CLAUDE_MODEL", "test-model");
        let config = CLIConfiguration::new();
        assert_eq!(config.get_model_for_cli(CLIType::Claude), Some(&"test-model".to_string()));
    }
}
```

## Next Steps
After completion enables:
- Task 3: CLI Adapter Trait System (can use any configured models)
- All subsequent CLI integrations (Codex, Opencode, Gemini)
- Model capability-based routing
- Cost optimization based on model pricing

This task is the foundation that unblocks the entire multi-CLI platform development.