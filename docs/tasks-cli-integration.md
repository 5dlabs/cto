# Tasks Module CLI Integration

## Overview

The `tasks` module has been refactored to replace direct LLM API calls with a unified CLI-based provider system. This allows the module to leverage any of the 6 supported AI CLIs (Claude, Codex, OpenCode, Cursor, Factory, Gemini) instead of being hardcoded to specific API implementations.

## Architecture

### Key Components

```
crates/tasks/src/ai/
├── provider.rs       # AIProvider trait and GenerateOptions
├── cli_provider.rs   # CLITextGenerator - unified CLI adapter
├── registry.rs       # ProviderRegistry for managing providers
├── anthropic.rs      # Legacy API provider (still available)
├── openai.rs         # Legacy API provider (still available)
└── prompts/          # Prompt templates (Handlebars)
```

### CLITextGenerator

The `CLITextGenerator` in `cli_provider.rs` implements the `AIProvider` trait and provides:

- Automatic CLI detection via `which` crate
- Model validation per CLI
- Extended thinking support (Claude, Cursor, Factory)
- MCP config integration
- JSONL/JSON output parsing for each CLI's format

## Supported CLIs

| CLI | Executable | Model Format | JSON Output | JSON Input | Extended Thinking |
|-----|------------|--------------|-------------|------------|-------------------|
| **Claude** | `claude` | `claude-opus-4-5-20251101` | ✅ `--output-format json` | ✅ `--input-format stream-json` | ✅ `--settings '{"alwaysThinkingEnabled":true}'` |
| **Codex** | `codex` | `gpt-5.1-codex` | ✅ `--json` (JSONL) | ❌ | ❌ (built-in reasoning) |
| **OpenCode** | `opencode` | `provider/model` (e.g., `anthropic/claude-opus-4-5`) | ✅ `--format json` | ❌ | ❌ |
| **Cursor** | `cursor` | `opus-4.5-thinking` | ✅ `--output-format json` | ❌ | ✅ (model name suffix) |
| **Factory** | `droid` | `claude-opus-4-5-20251101` | ✅ `--output-format json` | ✅ `--input-format stream-json` | ✅ `--reasoning-effort high` |
| **Gemini** | `gemini` | `gemini-2.5-flash` | ✅ `--output-format json` | ❌ | ❌ |

## CLI Command Patterns

### Claude
```bash
claude --print --output-format json --model claude-opus-4-5-20251101 \
  --max-output-tokens 4096 \
  --settings '{"alwaysThinkingEnabled": true}' \
  --thinking-budget 10000 \
  -- "prompt"
```

### Codex
```bash
codex exec --json -m gpt-5.1-codex \
  -c max_output_tokens=16000 \
  --skip-git-repo-check \
  -- "prompt"
```

**Important:** Codex requires `-c max_output_tokens=16000` for detailed task breakdowns to avoid truncation.

### OpenCode
```bash
opencode run --format json -m anthropic/claude-opus-4-5 -- "prompt"
```

**Note:** OpenCode uses `provider/model` format (e.g., `anthropic/claude-opus-4-5`, `openai/gpt-4.1`).

### Cursor
```bash
cursor agent --print --output-format json --model opus-4.5-thinking -- "prompt"
```

### Factory (droid)
```bash
droid exec --output-format json -m claude-opus-4-5-20251101 \
  --reasoning-effort high \
  -- "prompt"
```

### Gemini
```bash
gemini -m gemini-2.5-flash -p "prompt"
# Or with JSON output:
gemini -m gemini-2.5-flash --output-format json "prompt"
```

## Output Parsing

Each CLI has a different output format that `CLITextGenerator::parse_cli_output()` handles:

### Claude/Cursor JSON
```json
{"type":"result","result":"<content>","cost_usd":0.05,...}
```

### Codex JSONL
```jsonl
{"type":"thread.started","thread_id":"..."}
{"type":"turn.started"}
{"type":"item.completed","item":{"type":"agent_message","text":"<content>"}}
{"type":"turn.completed","usage":{"input_tokens":5000,"output_tokens":1500}}
```

### OpenCode JSONL
```jsonl
{"type":"step_start",...}
{"type":"text","part":{"text":"<content>"}}
{"type":"step_finish","part":{"tokens":{"input":1000,"output":500}}}
```

### Factory JSON
```json
{"type":"result","subtype":"success","result":"<content>","duration_ms":7000,...}
```

## GenerateOptions

The `GenerateOptions` struct in `provider.rs` supports:

```rust
pub struct GenerateOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub json_mode: bool,
    pub schema_name: Option<String>,
    pub extended_thinking: bool,      // Enable thinking mode
    pub thinking_budget: Option<u32>, // Token budget for thinking
    pub mcp_config: Option<String>,   // Path to MCP config file
}
```

## JSON Response Parsing

The `parse_ai_response()` function in `provider.rs` handles:

1. **Markdown code blocks** - Extracts JSON from ` ```json ... ``` ` blocks
2. **Leading prose** - Handles cases where AI includes explanation before JSON
3. **Embedded code blocks** - Uses `rfind()` to find the last closing marker, avoiding embedded ` ``` ` in code examples
4. **Raw JSON** - Parses direct JSON output

## Known Issues & Fixes

### Codex Truncation
**Issue:** Codex was returning truncated JSON for large responses.
**Fix:** Added `-c max_output_tokens=16000` to Codex CLI arguments.

### Embedded Code Blocks
**Issue:** JSON responses containing code examples with ` ``` ` markers caused parsing failures.
**Fix:** Changed from `find("```")` to `rfind("\n```")` to find the actual closing marker.

### OpenCode Model Format
**Issue:** OpenCode uses `provider/model` format, not just model name.
**Fix:** Updated supported models list to use correct format (e.g., `anthropic/claude-opus-4-5`).

## Test Results (All 6 CLIs)

| CLI | Model | Duration | Tasks | Status |
|-----|-------|----------|-------|--------|
| Claude | claude-opus-4-5-20251101 | ~70s | 5 | ✅ |
| Codex | gpt-5.1-codex | ~20s | 5 | ✅ |
| OpenCode | anthropic/claude-opus-4-5 | ~160s | 5 | ✅ |
| Cursor | opus-4.5-thinking | ~80s | 5 | ✅ |
| Factory | claude-opus-4-5-20251101 | ~110s | 5 | ✅ |
| Gemini | gemini-2.5-flash | ~26s | 5 | ✅ |

## File Changes Summary

### New/Modified Files

- `crates/tasks/src/ai/provider.rs` - Added `extended_thinking`, `thinking_budget`, `mcp_config` fields, improved JSON parsing
- `crates/tasks/src/ai/cli_provider.rs` - New CLI adapter implementation
- `crates/tasks/src/ai/mod.rs` - Added `cli_provider` module export
- `crates/tasks/Cargo.toml` - Added `cli`, `which`, `tempfile` dependencies

### Deleted Files

- `crates/tasks/src/ai/comparison.rs` - Old comparison module
- `crates/tasks/src/bin/compare_providers.rs` - Old comparison binary

## Future Work

1. **Add `--ai-backend` flag to tasks CLI** - Allow users to select which CLI provider to use
2. **Implement JSON input streaming** - For Claude and Factory bidirectional support
3. **Add retry logic** - Handle transient CLI failures
4. **Parallel CLI execution** - Run multiple CLIs simultaneously for comparison
5. **Token usage tracking** - Aggregate token usage across CLI calls
6. **MCP integration** - Enable research mode via MCP tools

## Usage Example

```rust
use tasks::ai::cli_provider::CLITextGenerator;
use tasks::ai::provider::{AIProvider, AIMessage, GenerateOptions};
use cli::CLIType;

// Create a CLI provider for Claude
let provider = CLITextGenerator::new(
    CLIType::Claude,
    "claude-opus-4-5-20251101".to_string(),
    true,   // extended_thinking
    10000,  // thinking_budget
);

// Generate text
let messages = vec![
    AIMessage::system("You are a helpful assistant."),
    AIMessage::user("Generate a task breakdown for: Build a REST API"),
];

let options = GenerateOptions {
    max_tokens: Some(4096),
    json_mode: true,
    extended_thinking: true,
    ..Default::default()
};

let response = provider.generate_text("claude-opus-4-5-20251101", &messages, &options).await?;
```

## Related Crates

- `crates/cli` - Shared CLI types (`CLIType` enum, `CliAdapter` trait)
- `crates/controller` - Controller using CLI adapters for agent execution


