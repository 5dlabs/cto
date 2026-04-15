# Provider Failover & Credit Recovery

## RULES

1. **Always use Claude Code first.** It is your primary CLI. Never say "Claude Code isn't available" or "I can't use Claude Code" — it IS available via OAuth.
2. **Never use raw API calls to Anthropic.** Use the `claude` CLI with `CLAUDE_CODE_OAUTH_TOKEN`.
3. **When credits run out on one sub, switch to the other immediately.** Don't wait, don't ask.
4. **After switching, flush context to mem0** so the next provider can resume.

## How credits run out

You'll see errors like:
- `429 Too Many Requests` / `rate_limit_error`
- `402 Payment Required`
- `usage_limit_exceeded`
- Claude CLI exits with "Usage limit reached"

When this happens, **switch immediately** — don't retry the same token.

## Claude Code: Two OAuth Subscriptions

Two tokens are available as env vars:

| Env Var | Secret Key | Description |
|---------|------------|-------------|
| `CLAUDE_CODE_OAUTH_TOKEN` | `anthropic-sub1-oauth` | Active (loaded at startup) |
| `CLAUDE_CODE_OAUTH_TOKEN_SUB2` | `anthropic-sub2-oauth` | Backup |

### Switch Claude to Sub 2

```bash
export CLAUDE_CODE_OAUTH_TOKEN="$CLAUDE_CODE_OAUTH_TOKEN_SUB2"
# Verify
claude -p "Say hello" --max-turns 1
```

### Switch Claude back to Sub 1

```bash
# Re-read from original env (requires pod restart, or source from secret)
# The original value was set at pod startup
```

**Note:** Switching is one-way per pod lifecycle. If both subs are exhausted, fall back to Fireworks gateway models.

## Codex CLI: Two OAuth Subscriptions

Two ChatGPT OAuth auth.json files stored in K8s secrets:

| Env Var | Secret Key | Description |
|---------|------------|-------------|
| `CODEX_AUTH_SUB1` | `codex-auth-sub1` | Active (written to ~/.codex/auth.json at init) |
| `CODEX_AUTH_SUB2` | `codex-auth-sub2` | Backup |

### Switch Codex to Sub 2

```bash
printf '%s' "$CODEX_AUTH_SUB2" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json
```

### Switch back to Sub 1

```bash
printf '%s' "$CODEX_AUTH_SUB1" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json
```

## Gateway Automatic Failover

The gateway model chain handles Fireworks failover automatically:

```
primary: fireworks/kimi-k2p5-turbo
  → fireworks/qwen3p6-plus
  → fireworks/minimax-m2p7
  → fireworks/glm-5p1
```

No action needed — 429/402/401 triggers the next model.

### Manual gateway model switch

```bash
openclaw config set agents.defaults.model.primary "fireworks/accounts/fireworks/models/qwen3p6-plus"
```

## Session Recovery After Switch

### Before switching
1. Finish the current atomic operation (don't switch mid-edit)
2. Tell yourself: "Saving progress to memory before provider switch"
3. Use `memory_add` to store current task state

### After switching
1. Start with: "Resuming from memory after provider switch"
2. mem0 auto-recall injects recent context
3. Use `memory_search({ query: "current task" })` if needed

## Docker Builds & GHCR Push

Docker daemon is NOT available. Use **kaniko** sidecar for image builds:

```bash
# Build and push via kaniko sidecar
POD=$(hostname)
kubectl exec $POD -c kaniko -- /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/Dockerfile \
  --destination=ghcr.io/5dlabs/cto:latest \
  --cache=true

# GHCR auth is pre-configured in kaniko via /kaniko/.docker/config.json
# DOCKER_CONFIG is also set in agent container at /workspace/.docker
```

For `docker push` from the agent container, GHCR credentials are mounted at `$DOCKER_CONFIG/config.json`.

## Credit Dashboards

- Anthropic: https://console.anthropic.com/settings/billing
- OpenAI/ChatGPT: https://chatgpt.com/admin
- Fireworks: https://fireworks.ai/account/billing
