---
name: Intake SDK Migration
overview: Replace the problematic CLI-based Claude integration in the intake crate with the claude-agent-sdk-rs Rust SDK, eliminating JSON parsing issues while preserving the existing AIProvider abstraction.
todos:
  - id: add-sdk-dep
    content: Add claude-agent-sdk-rs = "0.6" to crates/intake/Cargo.toml
    status: pending
  - id: create-sdk-provider
    content: Create crates/intake/src/ai/sdk_provider.rs with SdkProvider struct implementing AIProvider trait
    status: pending
  - id: impl-generate-text
    content: Implement generate_text() using claude_agent_sdk_rs::query() with PermissionMode::BypassPermissions
    status: pending
  - id: impl-response-extraction
    content: Implement extract_text_from_messages() to convert SDK Message/ContentBlock to text
    status: pending
  - id: update-registry
    content: Update crates/intake/src/ai/registry.rs to use SdkProvider as default instead of CLITextGenerator
    status: pending
  - id: remove-cli-provider
    content: Remove crates/intake/src/ai/cli_provider.rs (1300+ lines of problematic CLI code)
    status: pending
  - id: update-mod
    content: Update crates/intake/src/ai/mod.rs to export SdkProvider and remove cli_provider
    status: pending
  - id: test-integration
    content: Run intake with SDK provider against AlertHub PRD to verify task generation works
    status: pending
isProject: false
---

# Intake SDK Migration Plan

## Background

The current intake CLI mode has persistent JSON parsing failures when Claude returns prose instead of structured JSON. The `claude-agent-sdk-rs` crate (v0.6.2) provides a more reliable Rust-native integration with 100% feature parity with the official Python SDK.

**SDK Choice**: [`claude-agent-sdk-rs`](https://docs.rs/claude-agent-sdk-rs/0.6.2/claude_agent_sdk_rs/) over `claude-agent-sdk` because:
- More mature (v0.6.2 vs v0.1.1)
- Better streaming support
- Built-in permission bypass mode
- Includes `query()` one-shot function (perfect for intake)

## Architecture

```mermaid
flowchart TB
    subgraph Current["Current Architecture"]
        AIProviderTrait[AIProvider Trait]
        CLIProvider[CLITextGenerator]
        AnthropicProvider[AnthropicProvider]
        JSONParser[JSON Parser Utils]
        
        AIProviderTrait --> CLIProvider
        AIProviderTrait --> AnthropicProvider
        CLIProvider --> JSONParser
        AnthropicProvider --> JSONParser
    end
    
    subgraph New["New Architecture"]
        AIProviderTrait2[AIProvider Trait]
        SdkProvider[SdkProvider]
        AnthropicProvider2[AnthropicProvider]
        JSONParser2[JSON Parser Utils]
        
        AIProviderTrait2 --> SdkProvider
        AIProviderTrait2 --> AnthropicProvider2
        SdkProvider --> JSONParser2
        AnthropicProvider2 --> JSONParser2
    end
    
    Current --> New
```

## Key Files

| File | Action |
|------|--------|
| [`crates/intake/Cargo.toml`](crates/intake/Cargo.toml) | Add `claude-agent-sdk-rs` dependency |
| [`crates/intake/src/ai/sdk_provider.rs`](crates/intake/src/ai/sdk_provider.rs) | **NEW** - SDK implementation |
| [`crates/intake/src/ai/cli_provider.rs`](crates/intake/src/ai/cli_provider.rs) | Remove (1300+ lines) |
| [`crates/intake/src/ai/registry.rs`](crates/intake/src/ai/registry.rs) | Update to use SDK provider |
| [`crates/intake/src/ai/provider.rs`](crates/intake/src/ai/provider.rs) | Keep JSON parsing utils |

## SDK Provider Implementation

Core integration using `query()` for one-shot task generation:

```rust
use claude_agent_sdk_rs::{query, ClaudeAgentOptions, Message, ContentBlock, PermissionMode};
use crate::ai::provider::{AIProvider, AIMessage, AIResponse, GenerateOptions};

pub struct SdkProvider {
    model: String,
}

#[async_trait]
impl AIProvider for SdkProvider {
    async fn generate_text(
        &self,
        model: &str,
        messages: &[AIMessage],
        options: &GenerateOptions,
    ) -> TasksResult<AIResponse> {
        let prompt = self.messages_to_prompt(messages);
        
        let sdk_options = ClaudeAgentOptions::builder()
            .model(model)
            .permission_mode(PermissionMode::BypassPermissions)
            .max_turns(1)  // One-shot for task generation
            .build();
        
        let result = query(&prompt, Some(sdk_options)).await?;
        
        // Extract text from response
        let text = self.extract_text_from_messages(&result);
        
        Ok(AIResponse {
            text,
            usage: self.extract_usage(&result),
            model: model.to_string(),
            provider: "sdk".to_string(),
        })
    }
}
```

## Preserve Existing Components

**Keep unchanged:**
- [`crates/intake/src/ai/provider.rs`](crates/intake/src/ai/provider.rs) - JSON extraction logic:
  - `extract_json_continuation()` (lines 236-326)
  - `validate_json_continuation()` (lines 328-404)  
  - `parse_ai_response()` (lines 406-519)
- [`crates/intake/src/domain/ai.rs`](crates/intake/src/domain/ai.rs) - Retry logic with `MAX_PRD_PARSE_RETRIES = 3`
- [`crates/intake/src/ai/anthropic.rs`](crates/intake/src/ai/anthropic.rs) - API fallback provider
- Prompt template system (Handlebars)
- Prefill technique for JSON output

## Parallel Execution Strategy

Tasks can be executed by swarm agents in parallel where dependencies allow:

```mermaid
flowchart LR
    subgraph Phase1["Phase 1 - Parallel"]
        T1[Add SDK Dependency]
        T2[Create Provider Skeleton]
    end
    
    subgraph Phase2["Phase 2 - Sequential"]
        T3[Implement generate_text]
        T4[Add Response Extraction]
    end
    
    subgraph Phase3["Phase 3 - Parallel"]
        T5[Remove CLI Provider]
        T6[Update Registry]
        T7[Update Tests]
    end
    
    Phase1 --> Phase2
    Phase2 --> Phase3
```

## Environment Variables

**Remove:**
- `TASKS_USE_CLI` - No longer needed
- `TASKS_CLI` - No longer needed

**Keep:**
- `TASKS_MODEL` - Model selection
- `TASKS_EXTENDED_THINKING` - Enable via SDK options
- `TASKS_THINKING_BUDGET` - Map to `max_thinking_tokens`

## Testing

1. Unit tests for `SdkProvider` message conversion
2. Integration test with actual SDK call
3. Verify JSON parsing works with SDK responses
4. Test retry logic still functions

## Rollback

If SDK fails, the `AnthropicProvider` API mode remains available as fallback via the registry.