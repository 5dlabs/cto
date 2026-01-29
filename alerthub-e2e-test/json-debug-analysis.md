# JSON Debug Analysis: Taskmaster vs Our Implementation

## Executive Summary

After analyzing Taskmaster-AI's approach to JSON parsing and comparing it with our implementation, I've identified key differences and patterns that can improve our JSON handling reliability.

## Taskmaster Patterns Identified

### 1. JSON Repair Library (`jsonrepair`)
**Location:** `src/ai-providers/base-provider.js:2`

Taskmaster uses the `jsonrepair` npm package to fix malformed JSON:
```javascript
import { jsonrepair } from 'jsonrepair';

// In generateObject catch block:
if (NoObjectGeneratedError.isInstance(error) && error.cause instanceof JSONParseError) {
    const repairedJson = jsonrepair(error.cause.text);
    const parsed = JSON.parse(repairedJson);
    // Return repaired result
}
```

**Key insight:** They catch JSON parse errors at the provider level and attempt repair before failing.

### 2. Markdown Code Block Stripping
**Location:** `src/utils/stream-parser.js:291-296`

```javascript
_cleanJsonText(text) {
    // Remove markdown code block wrappers and trim whitespace
    return text
        .replace(/^```(?:json)?\s*\n?/i, '')
        .replace(/\n?```\s*$/i, '')
        .trim();
}
```

**Key insight:** They handle both `\`\`\`json` and plain `\`\`\`` blocks with case-insensitive matching.

### 3. Structured Prompting for JSON-Only Output
**Location:** `src/prompts/parse-prd.json:60`

```
IMPORTANT: Your response must be a JSON object with a "tasks" property containing
an array of task objects. You may optionally include a "metadata" object.
Do not include any other properties.
```

**Key insight:** Explicit JSON-only instruction at the END of the user prompt.

### 4. Vercel AI SDK Integration
**Location:** `src/ai-providers/base-provider.js`

They use the Vercel AI SDK's `generateObject` with schema validation:
- Uses `zodSchema(params.schema)` for type validation
- Sets `mode: 'json'` for explicit JSON mode
- Provides `schemaName` and `schemaDescription` for better context

### 5. Stream Fallback Parsing
**Location:** `src/utils/stream-parser.js:250-322`

The `FallbackParser` class handles cases where streaming JSON parsing fails:
1. Checks if we got fewer items than expected
2. Attempts to parse accumulated text as complete JSON
3. Uses `_cleanJsonText()` to strip code blocks first
4. Extracts items using `fallbackItemExtractor` callback

## Our Current Implementation

### Strengths
1. **Prefill technique** - We add `{"tasks":[` as assistant message to force JSON continuation
2. **JSON extraction** - `extract_json_continuation()` handles prose before/after JSON
3. **Validation** - `validate_json_continuation()` checks for valid task structure
4. **Markdown code block handling** - Already strips `\`\`\`json` blocks

### Gaps Identified

1. **No JSON repair** - We retry on failure but don't attempt to repair malformed JSON
2. **Incomplete code block regex** - Missing case-insensitive flag and edge cases
3. **No structured schema validation** - We parse JSON but don't validate against schema before attempting deserialization
4. **Missing explicit JSON instruction** - Our prompts don't have the explicit "IMPORTANT: Your response must be JSON" at the end

## Test Cases

### Test 1: Pure JSON Array
```json
[{"id":1,"title":"Task 1"},{"id":2,"title":"Task 2"}]
```
**Expected:** Parse directly - BOTH PASS

### Test 2: JSON in Markdown Fences
```
Here's the JSON:

\`\`\`json
{"id":1,"title":"Task 1"}
\`\`\`
```
**Expected:** Strip fences, extract JSON - BOTH PASS

### Test 3: Prose with Embedded JSON
```
I'll generate tasks based on the PRD. Here's my analysis:

{"id":1,"title":"Task 1"},{"id":2,"title":"Task 2"}]}

This covers the main requirements.
```
**Expected:** Extract JSON portion - OUR IMPL PASSES, but could be more robust

### Test 4: Invalid JSON (trailing comma)
```json
{"tasks":[{"id":1,"title":"Task 1"},]}
```
**Expected:** Taskmaster repairs with jsonrepair, we fail and retry

### Test 5: Echoed Prefill
```
{"tasks":[{"id":1,"title":"Task 1"}]}
```
**Expected:** Both should handle - OUR IMPL correctly strips echoed prefill

## Recommended Fixes

### Fix 1: Add JSON Repair (Rust equivalent)
Add a JSON repair function using serde_json's error location to fix common issues:
- Trailing commas
- Missing closing brackets
- Unescaped quotes in strings

### Fix 2: Strengthen JSON-Only Prompting
Add explicit instruction at the end of user prompts:
```
CRITICAL: Your response must be ONLY a JSON object. Do not include any
explanatory text, markdown formatting, or code blocks. Output raw JSON directly.
```

### Fix 3: Model-Specific Handling for Opus 4.5
Opus 4.5 with extended thinking tends to include explanations. Ensure `force_disable_thinking: true` is set for all JSON parsing operations.

### Fix 4: Improve Markdown Fence Stripping
Make regex case-insensitive and handle more edge cases:
```rust
// Current
text.find("```json")

// Improved
text.to_lowercase().find("```json") // or use regex with (?i) flag
```

## Implementation Status

| Fix | Status | Notes |
|-----|--------|-------|
| JSON fence stripping | IMPLEMENTED | Now uses case-insensitive regex |
| Prose extraction | IMPLEMENTED | `extract_json_continuation` works |
| Echoed prefill | IMPLEMENTED | Strips `{"tasks":[` prefix |
| JSON repair | IMPLEMENTED | Added `jsonrepair` crate v0.1 |
| Model-specific handling | IMPLEMENTED | `force_disable_thinking: true` |
| Explicit JSON prompting | PARTIAL | Could be stronger |
| `try_repair_json` helper | IMPLEMENTED | Exported for use in domain layer |

## Changes Made

### 1. Added `jsonrepair` Crate (Cargo.toml)
```toml
jsonrepair = "0.1"
```

### 2. Updated `provider.rs`
- Added case-insensitive regex patterns for markdown code block detection
- Implemented `try_repair_json()` helper function for manual JSON repair
- Updated `extract_json_continuation()` to use regex for `\`\`\`json` (case-insensitive)
- Updated `parse_ai_response()` to try JSON repair before failing
- Removed duplicate code blocks

### 3. Key Functions

**`try_repair_json(json_text: &str) -> Result<String, String>`**
- Attempts to repair malformed JSON using the `jsonrepair` crate
- Returns Ok(repaired) if repair succeeded and validates
- Returns Err(original) if repair failed

**`extract_json_from_markdown(text: &str) -> Option<String>`** (internal)
- Uses case-insensitive regex for `\`\`\`json` detection
- Handles both `\`\`\`json` and plain `\`\`\`` blocks

**`parse_ai_response<T>(response: &AIResponse) -> TasksResult<T>`**
- Step 1: Extract JSON from markdown code blocks (case-insensitive)
- Step 2: Try direct JSON parsing
- Step 3: If parsing fails, try `jsonrepair` to fix malformed JSON
- Step 4: Return error with truncated response if all attempts fail

## Test Results

All 53 existing tests pass after the changes:
```
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Sources Referenced

- [jsonrepair crate](https://crates.io/crates/jsonrepair)
- [llm_json crate](https://github.com/oramasearch/llm_json)
- Taskmaster-AI source code (base-provider.js, stream-parser.js)
