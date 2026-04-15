---
name: provider_failover
description: Switch between Claude Code, Codex, and Fireworks providers when credits run out
metadata: {"openclaw": {"always": true}}
---

# Provider Failover

## Rules

1. **Always use Claude Code CLI first.** It is your primary coding tool. Never claim it's unavailable.
2. When credits run out on one subscription, **switch immediately** — don't retry, don't ask.
3. **Flush context to mem0 before switching** so the next provider can resume.
4. Gateway model failover is automatic — don't touch it unless all models fail.

## Credit exhaustion signals

- `429 Too Many Requests` / `rate_limit_error`
- `402 Payment Required` / `usage_limit_exceeded`
- Claude CLI: "Usage limit reached"
- Codex CLI: "Rate limit exceeded"

## Claude Code: two OAuth subscriptions

| Env var | Description |
|---------|-------------|
| `CLAUDE_CODE_OAUTH_TOKEN` | Active (loaded at startup) |
| `CLAUDE_CODE_OAUTH_TOKEN_SUB2` | Backup |

### Switch to sub 2

```bash
export CLAUDE_CODE_OAUTH_TOKEN="$CLAUDE_CODE_OAUTH_TOKEN_SUB2"
claude -p "Say hello" # verify
```

### Switch back (requires pod restart)

The original value is set at pod startup from the K8s secret.

## Codex CLI: two OAuth subscriptions

| Env var | Description |
|---------|-------------|
| `CODEX_AUTH_SUB1` | Active (written to ~/.codex/auth.json at init) |
| `CODEX_AUTH_SUB2` | Backup |

### Switch to sub 2

```bash
printf '%s' "$CODEX_AUTH_SUB2" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json
```

### Switch back to sub 1

```bash
printf '%s' "$CODEX_AUTH_SUB1" > ~/.codex/auth.json
chmod 600 ~/.codex/auth.json
```

## Gateway automatic failover

The model chain handles 429/402/401 automatically:

```
fireworks/kimi-k2p5-turbo → qwen3p6-plus → minimax-m2p7 → glm-5p1
```

### Manual gateway model override

```bash
openclaw config set agents.defaults.model.primary "fireworks/accounts/fireworks/models/qwen3p6-plus"
```

## Session recovery after switch

1. Finish the current atomic operation (don't switch mid-edit)
2. Call `memory_add` with current task state
3. Switch the credential
4. Start with: "Resuming from memory after provider switch"
5. Use `memory_search({ query: "current task" })` if auto-recall misses context

## Docker builds (via kaniko shim)

`docker build` is shimmed to use the kaniko sidecar automatically. Use it like normal:

```bash
docker build -t ghcr.io/5dlabs/cto:latest -f Dockerfile .
docker build -t ghcr.io/5dlabs/myimage:v1 --build-arg FOO=bar .
```

- `docker push` is a **no-op** — kaniko pushes during build
- `docker run`, `docker ps` etc. are **not supported** (no daemon)
- GHCR auth is pre-configured
- Use `docker version` to verify the shim is active
