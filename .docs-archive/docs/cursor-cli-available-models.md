# Cursor Agent CLI - Available Models

**Last Updated:** November 25, 2025  
**Cursor Agent Version:** 2025.11.20-a4d3945 (Latest - Verified)  
**Status:** ✅ Up to date

This document lists the exact model identifiers you can use with the `cursor agent --model <model>` command based on the latest Cursor CLI.

## Available Models

| Model Name | CLI Identifier | Provider | Notes |
|------------|---------------|----------|-------|
| **Composer 1** | `composer-1` | Cursor | Cursor's internal model |
| **Auto** | `auto` | Mixed | Automatically selects the best model for the task |
| **Claude 4.5 Sonnet** | `sonnet-4.5` | Anthropic | Claude Sonnet 4.5 |
| **Claude 4.5 Sonnet (Thinking)** | `sonnet-4.5-thinking` | Anthropic | Extended reasoning mode |
| **Claude 4.5 Opus** | `opus-4.5` | Anthropic | Latest and most capable Claude model |
| **Claude 4.5 Opus (Thinking)** | `opus-4.5-thinking` | Anthropic | Opus with extended reasoning |
| **Claude 4.1 Opus** | `opus-4.1` | Anthropic | Previous Opus version (legacy) |
| **Gemini 3 Pro** | `gemini-3-pro` | Google | Google's latest Gemini model |
| **GPT-5** | `gpt-5` | OpenAI | GPT-5 base model |
| **GPT-5.1** | `gpt-5.1` | OpenAI | Latest GPT-5.1 model |
| **GPT-5 High** | `gpt-5-high` | OpenAI | GPT-5 with higher reasoning |
| **GPT-5.1 High** | `gpt-5.1-high` | OpenAI | GPT-5.1 with higher reasoning |
| **GPT-5 Codex** | `gpt-5-codex` | OpenAI | GPT-5 optimized for code |
| **GPT-5 Codex High** | `gpt-5-codex-high` | OpenAI | GPT-5 Codex with higher reasoning |
| **GPT-5.1 Codex** | `gpt-5.1-codex` | OpenAI | Latest GPT-5.1 optimized for code |
| **GPT-5.1 Codex High** | `gpt-5.1-codex-high` | OpenAI | GPT-5.1 Codex with higher reasoning |
| **Grok** | `grok` | xAI | xAI's Grok model |

## Usage Examples

### Basic Usage
```bash
cursor agent --model sonnet-4.5 "Write a hello world program"
```

### With Extended Thinking
```bash
cursor agent --model sonnet-4.5-thinking "Solve this complex algorithm problem"
```

### Using Auto Selection
```bash
cursor agent --model auto "Help me with this task"
```

### In Print Mode (Scripting)
```bash
cursor agent --model gpt-5-codex --print "Generate a REST API"
```

## Model Selection Guide

### For General Tasks
- **Recommended:** `opus-4.5` or `auto`
- Best balanced performance for most coding tasks

### For Complex Reasoning
- **Recommended:** `opus-4.5-thinking` or `sonnet-4.5-thinking`
- Extended thinking capabilities for complex problems

### For Code Generation
- **Recommended:** `gpt-5.1-codex` or `gpt-5.1-codex-high`
- Optimized for code-specific tasks

### For Speed
- **Recommended:** `sonnet-4.5` or `auto`
- Fastest response times

### For Multi-Provider Coverage
- **Recommended rotation:** `opus-4.5`, `gemini-3-pro`, `gpt-5.1-codex`
- Ensures fallback across providers

## Verifying Available Models

To see the current list of models available to your account:

### Method 1: Interactive CLI
```bash
cursor agent
# Then type: /model
```

### Method 2: API Query (Requires API Key)
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  https://api.cursor.com/v0/models
```

### Method 3: Check Help
```bash
cursor agent --help | grep -A 2 "model"
```

## Model Availability Notes

- Model availability may vary based on your Cursor subscription plan
- Some models may require specific API access or permissions
- The `auto` model intelligently routes to the best available model
- Model identifiers are case-sensitive in some contexts

## Configuration in CTO Platform

When using these models in the CTO platform's `cto-config.json`, use these exact identifiers:

```json
{
  "agents": {
    "rex": {
      "cli": "cursor",
      "model": "sonnet-4.5"
    }
  }
}
```

## Legacy/Removed Models

The following models have been removed or deprecated:

- ❌ `cheetah` - Removed from Cursor CLI
- `sonnet-4` → Use `sonnet-4.5`
- `gpt-4o` → Use `gpt-5.1` or `gpt-5.1-codex`
- `gpt-5` → Consider upgrading to `gpt-5.1`
- `gpt-5-codex` → Consider upgrading to `gpt-5.1-codex`
- `opus-4.1` → Use `opus-4.5`
- `claude-sonnet-4-5-20250929` → Use `sonnet-4.5`
- `claude-opus-4-5-20251101` → Use `opus-4.5`

## Additional Resources

- [Cursor CLI Documentation](https://docs.cursor.com/en/cli/using)
- [Cursor API Reference](https://docs.cursor.com/en/background-agent/api/list-models)
- [Cursor Forum](https://forum.cursor.com)

## Updating This Document

To update this list with the latest models:

1. Run `cursor agent` and use the `/model` command
2. Query the API endpoint: `https://api.cursor.com/v0/models`
3. Check the official Cursor documentation
4. Test model availability with your API key

---

**Note:** This list is based on publicly available information and web search results as of November 2025. The actual available models may vary based on your account type, region, and Cursor's latest updates.

