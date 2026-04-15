# Provider Failover & Credit Recovery

## Overview

When a model provider runs out of credits or returns rate-limit / auth errors,
you can switch to an alternative provider without losing session context.
Your session is periodically flushed to mem0 (OpenMemory) so you can recover
context after a provider switch.

## Available Providers

| Provider | Model ID | Env Var | Notes |
|----------|----------|---------|-------|
| Anthropic (OAuth) | claude-sonnet-4-20250514 | `CLAUDE_CODE_OAUTH_TOKEN` | Primary for Claude Code CLI |
| Fireworks / Kimi K2.5 Turbo | accounts/fireworks/routers/kimi-k2p5-turbo | `FIREWORKS_API_KEY` | Primary gateway model |
| Fireworks / Qwen 3.6 Plus | accounts/fireworks/models/qwen3p6-plus | `FIREWORKS_API_KEY` | Fallback 1 |
| Fireworks / MiniMax M2.7 | accounts/fireworks/models/minimax-m2p7 | `FIREWORKS_API_KEY` | Fallback 2 |
| Fireworks / GLM 5.1 | accounts/fireworks/models/glm-5p1 | `FIREWORKS_API_KEY` | Fallback 3 |

## Automatic Failover

The gateway's `modelDefaults.fallbacks` chain handles automatic failover:

```
primary: fireworks/accounts/fireworks/routers/kimi-k2p5-turbo
  → fallback 1: fireworks/accounts/fireworks/models/qwen3p6-plus
  → fallback 2: fireworks/accounts/fireworks/models/minimax-m2p7
  → fallback 3: fireworks/accounts/fireworks/models/glm-5p1
```

When the primary model returns a 429 (rate limit), 402 (payment required),
or 401 (auth error), the gateway automatically tries the next fallback.

## Manual Provider Switch

If all automatic fallbacks are exhausted or you need to switch the **primary**
model:

### 1. Switch gateway model via config

```bash
# Check current model
openclaw config get agents.defaults.model

# Switch to a different primary
openclaw config set agents.defaults.model.primary "fireworks/accounts/fireworks/models/qwen3p6-plus"

# Or switch to Claude (if OAuth token is valid)
openclaw config set agents.defaults.model.primary "anthropic/claude-sonnet-4-20250514"
```

### 2. Switch Claude Code auth token

If the primary OAuth token runs out of credits, swap to the backup:

```bash
# The two tokens are stored in the secret as:
#   claude-oauth-token-sub1  (primary)
#   claude-oauth-token-sub2  (backup)
#
# To switch, update the env var:
export CLAUDE_CODE_OAUTH_TOKEN="$(cat /run/secrets/claude-oauth-token-sub2)"
```

### 3. Switch Fireworks API key

If the Fireworks account runs out of credits:

```bash
# Update the API key env var with a backup key
export FIREWORKS_API_KEY="<new-key>"
```

## Session Context Recovery

When switching providers mid-session, context may be lost. To recover:

### Automatic (mem0 memory flush)

The gateway automatically flushes session context to mem0 every time
compaction triggers (approximately every 8000 tokens of conversation).
This means:

- Task status, decisions, and progress are saved to OpenMemory
- On the next turn after switching providers, mem0 auto-recall injects
  relevant memories into the prompt context
- The agent picks up where it left off

### Manual Recovery

If automatic recovery isn't sufficient:

```
Use the memory_search tool to find recent task context:
  memory_search({ query: "current task status" })

Or list recent memories:
  memory_list({ limit: 10 })
```

### Best Practices

1. **Before switching:** Finish the current atomic operation if possible.
   Don't switch mid-file-edit.
2. **After switching:** Start your next message with context like
   "Continuing from where we left off on [task]" — mem0 auto-recall
   will inject relevant memories.
3. **Verify:** After switching, run a simple test to confirm the new
   provider works (e.g., ask a basic question).
4. **Monitor credits:** Check provider dashboards periodically:
   - Anthropic: https://console.anthropic.com/settings/billing
   - Fireworks: https://fireworks.ai/account/billing

## CLI Backend Session Persistence

Each CLI backend (Claude Code, OpenCode) persists its session state:

- **Claude Code:** Uses `--session-id` for named sessions. The gateway
  passes thread-bound session IDs automatically.
- **OpenCode:** Session state stored in `/workspace/.opencode/sessions/`.
- **Compaction:** When context gets large, the gateway compacts the
  conversation and flushes key facts to mem0 before truncating.

The combination of compaction + mem0 flush + session memory indexing means
that even if a provider switch forces a session restart, the agent retains
its operational context through memory recall.
