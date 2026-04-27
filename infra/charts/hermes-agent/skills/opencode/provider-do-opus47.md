---
name: opencode_do_opus47
description: Use DigitalOcean Gradient AI Claude Opus 4.7 via the OpenCode CLI as a third Claude provider when both Anthropic OAuth subs are out of credits
metadata: {"openclaw": {"always": false}}
---

# OpenCode + DigitalOcean Gradient AI — Claude Opus 4.7 Fallback

## When to use this

You exhausted **both** Claude Code OAuth subscriptions (Sub 1 and Sub 2) and still need Claude-class reasoning — e.g. a long refactor, hard debug, deep review. Before falling back to Fireworks gateway models (which are non-Claude), try **Claude Opus 4.7 hosted on DigitalOcean Gradient AI** via the **OpenCode CLI**.

This is the **third tier** of Claude availability:

```
Claude Code OAuth Sub 1  (claude CLI, primary)
   ↓ exhausted
Claude Code OAuth Sub 2  (claude CLI, backup)
   ↓ exhausted
Claude Opus 4.7 on DO Gradient  (opencode CLI, third tier)   ← this skill
   ↓ exhausted
Fireworks gateway chain   (non-Claude: kimi-k2p6 → qwen3p6-plus → …)
```

Do **not** reach for this skill before both OAuth subs are actually exhausted. Cost per token on DO is higher than the OAuth subs we already pay for.

## Why OpenCode (not Claude Code, not Codex)

DO Gradient exposes an **OpenAI-compatible** endpoint, not the native Anthropic Messages API. That means the `claude` CLI can't talk to it. We picked the **OpenCode** CLI because:

1. It supports arbitrary OpenAI-compatible providers via `opencode.json` + `@ai-sdk/openai-compatible`.
2. It's a real terminal coding agent (edit/apply/run loop), not just a chat REPL, so it swaps in cleanly where `claude` was running.
3. Using it here also gets a third CLI into our rotation alongside Claude Code and Codex.

## Credentials

- **Env var:** `DO_INFERENCE_KEY`
- **K8s secret:** `openclaw-api-keys` (key: `DO_INFERENCE_KEY`)
- **This is a model access key (not `DIGITALOCEAN_TOKEN`)** — minted in the DO Gradient AI → Model access keys panel. Never use the generic DO API token here.

Verify it's present:

```bash
test -n "$DO_INFERENCE_KEY" && echo "DO_INFERENCE_KEY set (len=${#DO_INFERENCE_KEY})"
```

## Endpoint

| Field | Value |
|---|---|
| Base URL | `https://inference.do-ai.run/v1` |
| Model ID | `anthropic-claude-opus-4.7` |
| Auth | `Authorization: Bearer $DO_INFERENCE_KEY` |
| Shape | OpenAI Chat Completions (NOT Anthropic native) |
| Context | 200k |
| Max output | 8k |

## OpenCode config

Write `~/.config/opencode/opencode.json` (or `./opencode.json` in repo root) — OpenCode merges both:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "do-gradient": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "DigitalOcean Gradient AI",
      "options": {
        "baseURL": "https://inference.do-ai.run/v1",
        "apiKey": "{env:DO_INFERENCE_KEY}"
      },
      "models": {
        "anthropic-claude-opus-4.7": {
          "name": "Claude Opus 4.7 (DO)",
          "limit": { "context": 200000, "output": 8192 }
        }
      }
    }
  }
}
```

Verification: `opencode models | grep -i 'claude-opus-4.7'` should list `do-gradient/anthropic-claude-opus-4.7`.

## Invocation

```bash
# One-shot task
opencode run \
  --provider do-gradient \
  --model anthropic-claude-opus-4.7 \
  "Refactor crates/controller/src/tasks/code/templates.rs::qualify_model_for_openclaw \
   to return Result<String, ModelError> instead of String"

# Interactive session (default TUI)
opencode --provider do-gradient --model anthropic-claude-opus-4.7
```

## Smoke test (before committing to a real task)

```bash
curl -sS -X POST https://inference.do-ai.run/v1/chat/completions \
  -H "Authorization: Bearer $DO_INFERENCE_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "anthropic-claude-opus-4.7",
    "max_tokens": 32,
    "messages": [{"role":"user","content":"reply with the single word: alive"}]
  }' | jq -r '.choices[0].message.content'
# expect: alive
```

If this returns 401/403, the `DO_INFERENCE_KEY` is invalid or the model access key was revoked — rotate via DO console and re-sync the K8s secret.

## OpenCode not installed?

If `opencode` is not on PATH in this agent image, install ad-hoc into `$HOME/.local/bin`:

```bash
# Preferred: bun global install (repo convention)
bun install -g @opencode-ai/opencode
# or npm
npm install -g @opencode-ai/opencode
```

This is temporary — a follow-up PR will bake OpenCode into the agent base image so it's always available.

## Exhaustion / fallback out of this tier

If DO credits run low, DO billing shuts the key off — you'll see `401 Unauthorized` or `403 quota_exceeded` on the endpoint. When that happens, drop to the Fireworks gateway chain per the main `provider-failover` skill. Do NOT retry the same key.

## Cost guardrails

- Opus 4.7 on DO is the **most expensive option in the failover chain**. Only use for tasks that genuinely require Claude-class reasoning.
- For mechanical edits, linting, and format-only changes, fall through to Fireworks/Kimi K2.6 directly — skip this tier entirely.
- If the task looks like <5 min of work, don't burn Opus tokens on it.

## Memory flush before switching

Same rule as the main failover skill:

```bash
# 1. Save context to mem0 before the switch
memory_add "Switching to DO Opus 4.7 via OpenCode. Current task: <summary>. Files touched: <list>."
# 2. Start new opencode session
opencode --provider do-gradient --model anthropic-claude-opus-4.7
# 3. In the new session, prime with:
#    "Resuming from memory after provider switch. Call memory_search({query: 'current task'})."
```
