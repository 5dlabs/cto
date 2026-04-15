# Provider Failover & Credit Recovery

When a provider runs out of credits or returns errors, switch to an alternative
without losing context. Session state is flushed to mem0 on every compaction.

## Available Providers

| Provider | Model ID | Auth | Notes |
|----------|----------|------|-------|
| Anthropic Sub 1 (OAuth) | claude-sonnet-4-20250514 | `CLAUDE_CODE_OAUTH_TOKEN` | Primary Claude Code |
| Anthropic Sub 2 (OAuth) | claude-sonnet-4-20250514 | swap `~/.claude/.credentials.json` | Backup Claude |
| Codex Sub 1 (OAuth) | codex/gpt-5.4 | `~/.codex/auth.json` (sub1) | Primary Codex |
| Codex Sub 2 (OAuth) | codex/gpt-5.4 | swap `~/.codex/auth.json` (sub2) | Backup Codex |
| Fireworks / Kimi K2.5 Turbo | accounts/fireworks/routers/kimi-k2p5-turbo | `FIREWORKS_API_KEY` | Primary gateway |
| Fireworks / Qwen 3.6 Plus | accounts/fireworks/models/qwen3p6-plus | `FIREWORKS_API_KEY` | Fallback 1 |
| Fireworks / MiniMax M2.7 | accounts/fireworks/models/minimax-m2p7 | `FIREWORKS_API_KEY` | Fallback 2 |
| Fireworks / GLM 5.1 | accounts/fireworks/models/glm-5p1 | `FIREWORKS_API_KEY` | Fallback 3 |

## Automatic Failover (gateway)

```
primary: fireworks/kimi-k2p5-turbo
  → fireworks/qwen3p6-plus → fireworks/minimax-m2p7 → fireworks/glm-5p1
```

429 (rate limit), 402 (payment), or 401 (auth) triggers the next fallback.

## Manual Switch: Gateway Model

```bash
openclaw config get agents.defaults.model
openclaw config set agents.defaults.model.primary "fireworks/accounts/fireworks/models/qwen3p6-plus"
```

## Manual Switch: Claude Code Subscription

Two OAuth subscriptions are stored as K8s secrets. Switch when credits run out:

```bash
# Check which sub is active
echo $CLAUDE_CODE_OAUTH_TOKEN | head -c 30

# Switch to sub 2
export CLAUDE_CODE_OAUTH_TOKEN="$(cat /run/secrets/claude-oauth-token-sub2)"

# Switch back to sub 1
export CLAUDE_CODE_OAUTH_TOKEN="$(cat /run/secrets/claude-oauth-token-sub1)"
```

## Manual Switch: Codex CLI Subscription

Two ChatGPT OAuth subscriptions stored in K8s secrets as base64-encoded auth.json files.

```bash
# Check current sub
cat ~/.codex/auth.json | python3 -c "
import json,sys,base64
d=json.load(sys.stdin)
t=d['tokens']['id_token'].split('.')[1]
t+='='*(4-len(t)%4)
c=json.loads(base64.b64decode(t))
a=c.get('https://api.openai.com/auth',{})
print(f'Account: {a.get(\"chatgpt_account_id\")}, Plan: {a.get(\"chatgpt_plan_type\")}, Email: {c.get(\"email\")}')
"

# Switch to sub 2
printf '%s' "$CODEX_AUTH_SUB2" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json

# Switch back to sub 1
printf '%s' "$CODEX_AUTH_SUB1" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json
```

**Important:** Codex uses OAuth with refresh tokens. The refresh token auto-renews
the access token, so auth.json stays valid long-term. If it expires, re-run
`codex login` locally, copy the new auth.json, and update the K8s secret.

## Session Recovery After Switch

### Automatic (mem0)

Compaction flushes context to mem0 (~every 8000 tokens). After switching:
- mem0 auto-recall injects recent memories into the next prompt
- The agent resumes from stored context

### Manual

```
memory_search({ query: "current task status" })
memory_list({ limit: 10 })
```

### Procedure

1. **Before switching:** Finish the current atomic operation. Don't switch mid-edit.
2. **Flush to memory:** If possible, tell the agent "save your current progress to memory"
   before the switch to ensure the latest state is captured.
3. **Switch the credential** (see commands above).
4. **After switching:** Start with "Continuing from where we left off" — mem0 recall fills context.
5. **Verify:** Run a quick test to confirm the new provider responds.

### Credit Dashboards

- Anthropic: https://console.anthropic.com/settings/billing
- OpenAI/ChatGPT: https://platform.openai.com/usage (API) or https://chatgpt.com/admin (subscription)
- Fireworks: https://fireworks.ai/account/billing

## CLI Session Persistence

- **Claude Code:** Thread-bound `--session-id`, auto-managed by gateway
- **Codex:** Session in `~/.codex/sessions/`, persists across restarts
- **OpenCode:** Session in `/workspace/.opencode/sessions/`
- **Compaction + mem0:** Key facts flushed to memory before context truncation
