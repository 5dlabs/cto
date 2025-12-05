# CLI Reference Documentation

This directory contains source code documentation and reference materials for each
CLI tool supported by the CTO platform.

## Supported CLIs

| CLI | Package | Repository | Memory File |
|-----|---------|------------|-------------|
| [Claude Code](claude-code.md) | `@anthropic-ai/claude-code` | [anthropics/claude-code](https://github.com/anthropics/claude-code) | `CLAUDE.md` |
| [OpenAI Codex](codex.md) | `@openai/codex` | [openai/codex](https://github.com/openai/codex) | `AGENTS.md` |
| [Gemini CLI](gemini-cli.md) | `@google/gemini-cli` | [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) | `GEMINI.md` |
| [OpenCode](opencode.md) | `opencode-ai` | [sst/opencode](https://github.com/sst/opencode) | `AGENTS.md` |
| [Cursor Agent](cursor.md) | `cursor-agent` | (Proprietary) | N/A |
| [Aider](aider.md) | `aider-chat` (PyPI) | [Aider-AI/aider](https://github.com/Aider-AI/aider) | N/A |
| [Factory Droid](factory.md) | (Proprietary) | [Factory-AI/factory](https://github.com/Factory-AI/factory) | N/A |

## Quick Reference

### CTO Config Snippets

```json
// Claude Code
{ "cli": "claude", "model": "claude-sonnet-4-20250514" }

// OpenAI Codex
{ "cli": "codex", "model": "gpt-5-codex" }

// Cursor Agent
{ "cli": "cursor", "model": "gpt-5-cursor" }

// Factory Droid
{ "cli": "factory", "model": "gpt-5-factory-high" }

// OpenCode
{ "cli": "opencode", "model": "opencode-sonnet" }

// Gemini CLI
{ "cli": "gemini", "model": "gemini-2.5-pro" }
```

### Memory/Guidance File Patterns

Different CLIs use different file names for project-specific guidance:

| Pattern | CLIs |
|---------|------|
| `CLAUDE.md` | Claude Code |
| `AGENTS.md` | Codex, OpenCode |
| `GEMINI.md` | Gemini CLI |
| `.grok/GROK.md` | Grok CLI |

## Architecture Patterns

### npm-delivered CLIs
- Claude, Codex, OpenCode, Gemini — Share Node-based runtime
- Integration via per-CLI config formats

### Rust CLI
- Codex — External MCP client via `tools` STDIO wrapper

### Python CLIs
- Cursor, Aider — Python/Poetry virtualenv frameworks

## MCP Support

| CLI | MCP Support | Notes |
|-----|-------------|-------|
| Claude Code | ✅ Native | Uses `tools` CLI bridge for STDIO ↔ HTTP |
| Codex | ✅ STDIO-only | Configure in `config.toml` |
| Gemini CLI | ✅ Native | Configure in `settings.json` |
| OpenCode | ✅ Native | Configure in `opencode.json` |
| Cursor | ✅ Native | Configure via CLI or settings |
| Aider | ⚠️ Limited | Via integrations |
| Factory | ⚠️ Community | Via [factory-mcp](https://github.com/iannuttall/factory-mcp) |

## Related Documentation

- [CTO CLI Config Reference](../cto-cli-config-reference.md) — Platform configuration snippets
- [Cursor CLI Available Models](../cursor-cli-available-models.md) — Detailed Cursor model list
- [Multi-CLI Integration Design](../multi-cli-integration-design.md) — Architecture for CLI abstraction

