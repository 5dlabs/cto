# CTO CLI Configuration Reference

This reference summarizes the `cli` and `model` values that we currently ship for each CLI supported by the CTO platform. Use these snippets when populating `cto-config.json` (either under `defaults.code` / `defaults.play` or within individual `agents.<name>` entries).

## Summary

| CLI | `cli` value | Recommended `model` | Notes |
| --- | --- | --- | --- |
| Claude Code | `claude` | `claude-sonnet-4-20250514` | Baseline Claude configuration; no additional `cliConfig` required. |
| OpenAI Codex | `codex` | `gpt-5-codex` | Supports optional `cliConfig` overrides for tokens, temperature, approval, sandbox. |
| Cursor Agent | `cursor` | `gpt-5-cursor` | Headless Cursor runner; accepts optional settings payload for editor and sandbox flags. |
| Factory Droid | `factory` | `gpt-5-factory-high` | Requires API key via `FACTORY_API_KEY`; `cliConfig` can tune sandbox and auto-run levels. |
| OpenCode | `opencode` | `opencode-sonnet` | Provider defaults to Anthropic; `cliConfig` controls provider metadata and instructions. |

## CLI Snippets

### Claude Code

```json
{
  "cli": "claude",
  "model": "claude-sonnet-4-20250514"
}
```

### OpenAI Codex

```json
{
  "cli": "codex",
  "model": "gpt-5-codex",
  "cliConfig": {
    "model": "gpt-5-codex",
    "maxTokens": 64000,
    "temperature": 0.7,
    "approvalPolicy": "on-request",
    "sandboxPreset": "workspace-write",
    "settings": {
      "reasoningEffort": "medium"
    }
  }
}
```

### Cursor Agent

```json
{
  "cli": "cursor",
  "model": "gpt-5-cursor",
  "cliConfig": {
    "model": "gpt-5-cursor",
    "maxTokens": 64000,
    "temperature": 0.7,
    "settings": {
      "sandboxMode": "danger-full-access",
      "approvalPolicy": "never",
      "editor": {
        "vimMode": true
      }
    }
  }
}
```

### Factory Droid

```json
{
  "cli": "factory",
  "model": "gpt-5-factory-high",
  "cliConfig": {
    "model": "gpt-5-factory-high",
    "maxTokens": 64000,
    "settings": {
      "sandboxMode": "danger-full-access",
      "approvalPolicy": "never",
      "reasoningEffort": "high"
    }
  }
}
```

> Note: ensure `FACTORY_API_KEY` is mounted for Factory runs (see `controller/src/cli/bridge.rs`).

### OpenCode

```json
{
  "cli": "opencode",
  "model": "opencode-sonnet",
  "cliConfig": {
    "model": "opencode-sonnet",
    "maxTokens": 16384,
    "temperature": 0.65,
    "instructions": "Follow OpenCode best practices",
    "provider": {
      "name": "anthropic",
      "envKey": "ANTHROPIC_API_KEY"
    }
  }
}
```

These snippets can be embedded directly into the relevant sections of `cto-config.json` to align each agent with the desired CLI runner and model preset. Adjust `maxTokens`, `temperature`, and other optional keys as needed for specific workloads.
