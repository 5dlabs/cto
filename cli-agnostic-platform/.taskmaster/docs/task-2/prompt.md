# AI Agent Prompt: Simplify Model Validation for Multi-CLI Support

You are a senior Rust engineer focused on removing blockers and enabling rapid multi-CLI integration. Your mission: eliminate the hard-coded Claude-only model validation that prevents multi-CLI support with a minimal, flexible solution.

## Your Mission
The current `validate_model_name()` function in `/mcp/src/main.rs` blocks all non-Claude models. You need to replace this with a simple, permissive validation approach that allows rapid CLI integration without complex model catalogs or validation frameworks.

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

This blocks models like `gpt-4o`, `gpt-5-codex`, `gemini-pro`, `qwen-max`, etc.

## Simple Solution Approach

### Replace Hard-Coded Validation
Replace the restrictive validation with a permissive approach that allows any reasonable model name:

```rust
fn validate_model_name(model: &str) -> Result<()> {
    // Simple validation: reject empty or obviously invalid names
    if model.trim().is_empty() {
        return Err(anyhow!("Model name cannot be empty"));
    }
    
    // Allow any non-empty model name - let the CLI handle model-specific validation
    Ok(())
}
```

### Key Principles
- **Permissive by default**: Accept any reasonable model name
- **Fail fast**: Let individual CLIs handle their specific validation
- **No hardcoding**: Avoid maintaining model catalogs or complex validation rules
- **Future-proof**: New models work automatically without code changes

## Implementation Steps

### Step 1: Replace Current Validation
1. Locate the `validate_model_name()` function in `/mcp/src/main.rs`
2. Replace the restrictive Claude-only logic with permissive validation
3. Test that the function accepts various model names from different providers

### Step 2: Verify Integration Points
1. Check all callers of `validate_model_name()` 
2. Ensure the simplified validation works throughout the system
3. Test with sample model names from each CLI type

### Step 3: Update Tests
1. Update existing unit tests to reflect new permissive validation
2. Add tests for edge cases (empty strings, whitespace-only)
3. Remove tests that enforce Claude-specific model name patterns

## Test Cases to Validate

Test the updated function accepts these model names:
- **Claude**: `claude-3-opus`, `claude-3.5-sonnet`, `opus`, `sonnet`, `haiku`
- **OpenAI**: `gpt-4o`, `gpt-4-turbo`, `gpt-5-codex`, `o1-preview`, `o3-mini`
- **Google**: `gemini-1.5-pro`, `gemini-pro`, `gemini-pro-vision`
- **Qwen**: `qwen-max`, `qwen-turbo`, `qwen-plus`
- **Grok**: `grok-beta`, `grok-1`, `grok-2`
- **Custom**: Any reasonable custom model names

Test the updated function rejects these:
- **Empty**: `""`, `"   "` (whitespace only)
- **Invalid**: Potentially add basic format validation if needed

## Success Criteria
- ✅ All non-Claude model names are accepted
- ✅ Empty/whitespace-only names are rejected  
- ✅ Existing Claude models continue to work
- ✅ No complex validation frameworks or model catalogs
- ✅ Multi-CLI integration is unblocked
- ✅ All tests pass

## Constraints
- **Keep it simple**: Avoid over-engineering or complex validation systems
- **No breaking changes**: Existing Claude model names must continue to work
- **Performance**: Validation should be fast and lightweight
- **Maintainability**: Solution should be easy to understand and modify

Remember: The goal is to **unblock multi-CLI support quickly** by removing artificial restrictions, not to build a comprehensive model validation system. Let individual CLIs handle their own model-specific validation.