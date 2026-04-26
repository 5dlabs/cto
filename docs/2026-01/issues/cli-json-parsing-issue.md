# CLI JSON Parsing Issue - Intake Task Generation

## Summary

When using the Claude Code CLI in subprocess mode for intake task generation, we encounter persistent JSON parsing failures. The CLI outputs mixed content (prose + JSON) that is difficult to reliably parse.

## The Architecture

```
┌─────────────┐     spawn      ┌─────────────┐     stdout     ┌─────────────┐
│   Intake    │ ──────────────▶│ Claude CLI  │ ──────────────▶│   Parser    │
│   (Rust)    │                │ (subprocess)│                │   Logic     │
└─────────────┘                └─────────────┘                └─────────────┘
                                     │
                                     ▼
                               Mixed Output:
                               - Prose text
                               - JSON objects
                               - Markdown blocks
                               - Prefill echoes
```

## The Problem

### 1. Mixed Content Output

The CLI is designed for interactive use, not programmatic JSON extraction. Output can look like:

```
I'll continue from where I was cut off, completing the task generation.

{"id":35,"title":"Implement gRPC Service Handlers","description":"..."}
```

Or with markdown code blocks:

```
Here are the tasks:

```json
{"tasks":[{"id":1,"title":"Setup project"}]}
```
```

### 2. Prefill Echo Problem

When using the "prefill" technique (starting assistant message with `{"tasks":[`), the CLI sometimes echoes this back:

```
{"tasks":[I'll generate the following tasks based on the PRD...

{"id":1,"title":"First task"}]}
```

This creates malformed JSON with prose embedded inside the structure.

### 3. Extended Thinking Variations

With extended thinking enabled, Claude may output substantial reasoning before the JSON:

```
Based on the PRD requirements, I need to consider:
1. The authentication system needs...
2. The API layer should...
[... 500 lines of reasoning ...]

{"id":1,"title":"Setup authentication",...}
```

### 4. Hallucinated Content

Sometimes Claude outputs valid JSON but with wrong structure:

```json
{"expo":{"name":"AlertHub","slug":"alerthub"}}
```

Instead of the expected task objects:

```json
{"id":1,"title":"Setup project","description":"..."}
```

## Current Parsing Logic

Located in `crates/intake/src/ai/provider.rs`:

### `extract_json_continuation()`

Attempts to extract JSON from mixed content:
- Strips `{"tasks":[` prefill if echoed
- Looks for markdown code blocks (`\`\`\`json`)
- Finds first `{"id":` pattern
- Falls back to first `{` character

### `validate_json_continuation()`

Validates extracted content:
- Checks first character is `{` or `]`
- Verifies `"id"` is the first key (for task objects)
- Rejects prose-only responses

## Error Messages

Common errors from this issue:

```
AI returned invalid content
```

```
AI response does not contain valid task objects. Expected JSON array of tasks with 'id' as the first field
```

```
AI returned a summary or explanation instead of JSON task data
```

## Reproduction Steps

1. Run intake with a large PRD (50+ tasks expected)
2. Use extended thinking mode
3. Observe retry loop:
   ```json
   {"type":"retry","step":1,"attempt":1,"max":3,"reason":"AI returned invalid content"}
   {"type":"retry","step":1,"attempt":2,"max":3,"reason":"Extended thinking disabled, retrying"}
   ```

## Why This Worked Before

This is unclear and needs investigation. Possibilities:

1. **CLI version change** - Claude CLI may have changed output format
2. **Model behavior change** - Newer models may be more verbose
3. **Prompt changes** - Changes to system prompts affecting output
4. **Extended thinking** - Recent addition of thinking mode may have changed behavior

## Attempted Solutions

### 1. JSON Repair (jsonrepair crate)
- Added `jsonrepair` crate to fix malformed JSON
- Helps with minor issues but can't fix prose embedded in JSON

### 2. More Aggressive Extraction
- Added patterns to find `{"id":` specifically
- Strip markdown code blocks
- Handle prefill echoes
- Still fails with edge cases

### 3. Disable Extended Thinking on Retry
- First attempt with thinking enabled
- Retry with thinking disabled
- Sometimes works, sometimes doesn't

## Proposed Solutions

### Option A: Direct HTTP API (anthropic-sdk-rust)

Use direct HTTP calls to `api.anthropic.com`:
- Clean structured JSON responses
- No CLI output parsing
- Token usage in response

**Status:** Currently implementing

### Option B: Official Claude Agent SDK (Python/TypeScript)

Use the official SDK with MCP support:
- Built-in tool handling
- Skills support
- Clean response format

**Tradeoff:** Requires Python/TypeScript runtime

### Option C: Fix CLI Parsing

Improve extraction logic to handle all edge cases:
- More robust regex patterns
- Multi-pass extraction
- Streaming JSON parser

**Tradeoff:** Complex, fragile, still subject to CLI changes

## Files Involved

| File | Purpose |
|------|---------|
| `crates/intake/src/ai/provider.rs` | JSON extraction and validation |
| `crates/intake/src/ai/cli_provider.rs` | CLI subprocess management (deleted) |
| `crates/intake/src/ai/sdk_provider.rs` | New SDK-based provider |
| `crates/intake/src/ai/registry.rs` | Provider selection |
| `crates/intake/src/workflows/generate.rs` | Task generation workflow |

## Key Questions for Investigation

1. What changed in the Claude CLI output format between working and broken states?
2. Are there CLI flags to force structured output mode?
3. Is there a JSON-only output mode for the CLI?
4. Did the model's tendency to add explanatory text increase?
5. Are we using the correct API version header?

## Test Case

PRD: `alerthub-e2e-test/prd.json`
Expected: 50+ tasks generated
Actual: Retry loop with "AI returned invalid content"

## Related Code

### Extract JSON Continuation (simplified)

```rust
pub fn extract_json_continuation(text: &str) -> String {
    // Strip prefill echo
    let text = text.strip_prefix(r#"{"tasks":["#).unwrap_or(text);
    
    // Look for markdown code blocks
    if let Some(json) = extract_from_code_block(text) {
        return json;
    }
    
    // Find first task object
    if let Some(pos) = text.find(r#"{"id":"#) {
        return text[pos..].to_string();
    }
    
    // Fallback to first brace
    if let Some(pos) = text.find('{') {
        return text[pos..].to_string();
    }
    
    text.to_string()
}
```

### Validate JSON Continuation (simplified)

```rust
pub fn validate_json_continuation(content: &str) -> TasksResult<()> {
    let content = content.trim();
    
    if content.is_empty() {
        return Err(TasksError::Ai("Empty response".into()));
    }
    
    let first_char = content.chars().next().unwrap();
    
    if first_char == '{' {
        // Must start with {"id":
        let after_brace = content.trim_start_matches('{').trim_start();
        if !after_brace.starts_with("\"id\"") {
            return Err(TasksError::Ai("Invalid task structure".into()));
        }
    }
    
    Ok(())
}
```

## Contact

For questions about this issue, contact the CTO platform team.
