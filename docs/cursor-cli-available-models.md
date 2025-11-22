# Cursor Agent CLI - Available Models

**Last Updated:** November 22, 2025  
**Cursor Agent Version:** 2025.11.06-8fe8a63 (Latest - Verified)  
**Status:** ✅ Up to date

This document lists the exact model identifiers you can use with the `cursor agent --model <model>` command based on the latest Cursor CLI.

> **Note:** Gemini models are NOT currently available through Cursor CLI. Use the dedicated `gemini` CLI for Gemini model access.

## Available Models

| Model Name | CLI Identifier | Provider | Notes |
|------------|---------------|----------|-------|
| **Auto** | `auto` | Mixed | Automatically selects the best model for the task |
| **Cheetah** | `cheetah` | Cursor | Cursor's proprietary fast model |
| **Claude 4.5 Sonnet** | `sonnet-4.5` | Anthropic | Latest Claude Sonnet model |
| **Claude 4.5 Sonnet (Thinking)** | `sonnet-4.5-thinking` | Anthropic | Extended reasoning mode |
| **Claude 4.1 Opus** | `opus-4.1` | Anthropic | Most capable Claude model |
| **GPT-5** | `gpt-5` | OpenAI | Latest GPT-5 model |
| **GPT-5 Codex** | `gpt-5-codex` | OpenAI | GPT-5 optimized for code |
| **Grok** | `grok` | xAI | Grok model |

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
- **Recommended:** `sonnet-4.5` or `auto`
- Fast, balanced performance for most coding tasks

### For Complex Reasoning
- **Recommended:** `sonnet-4.5-thinking` or `opus-4.1`
- Extended thinking capabilities for complex problems

### For Code Generation
- **Recommended:** `gpt-5-codex` or `cheetah`
- Optimized for code-specific tasks

### For Speed
- **Recommended:** `cheetah` or `auto`
- Fastest response times

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

## Legacy Model Names

The following model names may still work but are deprecated:

- `sonnet-4` → Use `sonnet-4.5`
- `gpt-4o` → Use `gpt-5` or `gpt-5-codex`
- `claude-sonnet-4-5-20250929` → Use `sonnet-4.5`
- `claude-opus-4-1-20250805` → Use `opus-4.1`

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

