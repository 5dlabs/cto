# OpenClaw CLI / Provider Routing Design

**Status:** Design — April 2026  
**Scope:** Avatar / OpenClaw pod  
**Authors:** Angie (agent architecture)

---

## 1. CLI Inventory

Each CLI binary runs inside the OpenClaw pod (`infra/charts/openclaw-agent`). The init container
materialises per-CLI config files from the `cli-backend-configs` ConfigMap before the main
container starts.

| CLI | Binary | Config path (pod) | Default model | Provider | Secret key (`openclaw-api-keys`) | Env var |
|-----|--------|-------------------|---------------|----------|----------------------------------|---------|
| `claude` | `/usr/local/bin/claude` | `/workspace/.claude/` | `claude-opus-4-7-20260610` | Anthropic | `anthropic-api-key` | `ANTHROPIC_API_KEY` |
| `codex` | `/usr/local/bin/codex` | `/workspace/.codex/config.toml` | `gpt-5.3-codex` | OpenAI | `openai-api-key` ¹ | — (intentionally unset) ² |
| `copilot` | gh extension / standalone | `/workspace/.copilot/config.json` | Claude Opus 4.6 | GitHub Copilot | `github-pat` | `COPILOT_GITHUB_TOKEN` + `GH_TOKEN` |
| `cursor` | cursor agent | `/workspace/.cursor/mcp.json` | `opus-4.7` | Cursor | — (OAuth flow) ³ | — |
| `gemini` | gemini CLI | `/workspace/.gemini/settings.json` | `gemini-2.5-pro` | Google | `gemini-api-key` | `GEMINI_API_KEY` + `GOOGLE_API_KEY` |
| `opencode` | opencode CLI | `/workspace/opencode.json` | `fireworks/kimi-k2-turbo` | Fireworks | `fireworks-api-key` | `FIREWORKS_API_KEY` |
| `factory` / `droid` | `/usr/local/bin/droid` | `/workspace/.factory/mcp.json` | `glm-4-plus` | ZhipuAI | `factory-api-key` | `FACTORY_API_KEY` |
| `dexter` | dexter CLI | — | `claude-opus-4-7-20260610` | Anthropic | `anthropic-api-key` | `ANTHROPIC_API_KEY` |

Additional keys present in the pod:

| Env var | Key | Purpose |
|---------|-----|---------|
| `DO_INFERENCE_KEY` | `DO_INFERENCE_KEY` | DigitalOcean Gradient AI — tier-3 OpenCode fallback (Claude Opus 4.7 via `do-inference` provider) |
| `PROVIDER_OPENAI_API_KEY` | `openai-api-key` | Non-Codex OpenAI provider fallback; conditionally injected when `openai` or `openai-api` provider is enabled |
| `BRAVE_API_KEY` | `brave-api-key` | Web-search tool (Brave) |
| `FIRECRAWL_API_KEY` | `firecrawl-api-key` | Scraping tool (Firecrawl) |

**Notes**

¹ OpenAI is disabled in `model-providers.json` (`"enabled": false`). The secret key is maintained but the provider is not active.  
² `OPENAI_API_KEY` is deliberately **not** set as an env var. Setting it causes Codex CLI to rewrite its OAuth token (`/workspace/.codex/auth.json`) to API-key mode on startup, clobbering the OAuth session. Codex uses OAuth-based auth (`codex-auth-sub1` / `codex-auth-sub2`).  
³ Cursor is re-enabled (`model-providers.json`: `"Cursor": { "enabled": true }`) but uses an OAuth flow through `/workspace/.cursor/mcp.json`. No `CURSOR_API_KEY` env var is injected in the current deployment. If Cursor moves to a token-based API, add `cursor-api-key` → `CURSOR_API_KEY` following the existing pattern.

---

## 2. Routing Policy

### 2.1 Task-type → CLI tier mapping

Agents select a CLI at spawn time via the `agent:` parameter of `sessions_spawn`. The following
tiers guide selection; they are conventions rather than enforced constraints today (see §7 for
enforcement via CRD fields).

| Tier | CLIs | When to use |
|------|------|-------------|
| **Frontier** | `claude`, `dexter`, `copilot`, `cursor` | Complex multi-step reasoning, architecture work, long-context code review |
| **Fast** | `codex`, `gemini` | Tight loops, inline edits, sub-second latency tasks |
| **Specialised** | `opencode` (Kimi K2), `factory`/`droid` (ZhipuAI) | Tasks where Kimi K2 excels (math/tool-use), or cheap auxiliary work on ZhipuAI |
| **Orchestration** | `copilot` (via gh ext) | PR shepherding, GitHub API-integrated tasks where Copilot context is advantageous |

### 2.2 Cost hierarchy (daily quota sensitivity)

From most expensive / quota-sensitive to least:

```
Anthropic (claude / dexter)
  ↓
GitHub Copilot (copilot)
  ↓
Cursor (cursor) ← re-enabled but OAuth-constrained
  ↓
Google Gemini (gemini) ← API-key, effectively unlimited for embeddings
  ↓
Fireworks / Kimi K2 (opencode) ← token-based, lowest cost per token
  ↓
ZhipuAI (factory/droid) ← cheapest, narrower capability set
```

### 2.3 `cliModels` (from `cto-config.json`)

`cto-config.json` maps CLI names to model strings consumed by `sessions_spawn`. These are the
**local dev defaults** (pointing at `github-copilot` / `claude-opus-4.7`). In-cluster the model
selection is overridden by the per-CLI config files rendered into the pod (see §1).

### 2.4 `modelDefaults` fallback chain (`values.yaml`)

The OpenClaw gateway level has its own model fallback chain independent of CLI selection:

```yaml
modelDefaults:
  primary: "anthropic/claude-3-5-haiku-latest"
  fallbacks:
    - "anthropic/claude-sonnet-4-5"
    - "anthropic/claude-opus-4-5"
```

This is the gateway's fallback for **direct API calls**; it does not change which CLI binary is
spawned for ACP sessions.

---

## 3. Quota-Aware Rotation

### 3.1 Current state

There is **no automatic quota-aware CLI rotation** today. Agents manually select a CLI by setting
`agent:` in `sessions_spawn`. The gateway-level model fallback chain (§2.4) handles API 4xx errors
for direct gateway calls, but ACP session backends are not yet subject to the same fallback logic.

### 3.2 Proposed rotation mechanism

When a CLI session receives a quota / rate-limit signal, the following escalation chain should be
followed:

```
429 / quota exhausted on current CLI
  → save mem0 checkpoint (§3.3)
  → rotate to next CLI in chain (§3.4)
  → recover checkpoint in new session
  → continue task
```

**Rotation chain (priority order):**

```
claude / dexter  →  copilot  →  cursor  →  opencode (Kimi K2)  →  gemini  →  factory/droid
```

For `codex` (OpenAI disabled): skip to `copilot`.

### 3.3 Session persistence protocol (pre-switch checkpoint)

Before switching CLIs, the agent **must** execute the session-persistence checkpoint protocol
documented in `infra/charts/openclaw-agent/skills/openclaw/session-persistence.md`:

1. Write `mem0.store` with current task state, last outputs, and remaining sub-tasks.
2. Record the reason for the switch (e.g., `"quota_exhausted: anthropic"`) in metadata.
3. Spawn new session with replacement CLI.
4. Load `mem0.recall` in the new session to recover state.

### 3.4 Detecting quota exhaustion

Signal sources that should trigger rotation:

| Signal | Detection | Action |
|--------|-----------|--------|
| HTTP 429 from provider API | Gateway logs / CLI stderr | Rotate immediately |
| `RateLimitError` in CLI output | Parse stderr / exit code | Rotate immediately |
| `insufficient_quota` / `billing` error | CLI output matching | Rotate immediately + alert |
| Auth error (401 / 403) | CLI exit code 1 with auth message | Mark CLI unavailable for session; skip in rotation |
| Model not found (404) | CLI output | Try same CLI with fallback model; then rotate CLI |
| CLI binary missing / crash | exit code ≠ 0, binary absent | Skip CLI in rotation chain |

### 3.5 Future: `quotaRotation` block in `values.yaml`

A `quotaRotation` section should be added to `values.yaml` to make the rotation chain
declarative and operator-configurable:

```yaml
quotaRotation:
  enabled: true
  chain:
    - cli: claude-cli
      provider: anthropic
    - cli: copilot-cli
      provider: github-copilot
    - cli: cursor-cli
      provider: cursor
    - cli: opencode-cli
      provider: fireworks
    - cli: gemini-cli
      provider: google
    - cli: droid-cli
      provider: zhipuai
  softLimitAction: "rotate"     # rotate | alert | block
  hardLimitAction: "block"      # block | alert
```

The gateway can render this into an env var (`OPENCLAW_ROTATION_CHAIN`) or a mounted ConfigMap
for the session manager to consume.

---

## 4. Pod Secrets Spec

All provider API keys live in the `openclaw-api-keys` Kubernetes Secret (configured via
`secrets.apiKeysSecret` in `values.yaml`). All `secretKeyRef` entries use `optional: true` so
the pod starts even if a key is absent.

### 4.1 Complete key inventory

| Secret key | Env var(s) | CLI(s) that use it | Required for |
|------------|------------|---------------------|--------------|
| `anthropic-api-key` | `ANTHROPIC_API_KEY` | `claude`, `dexter` | Frontier tasks |
| `gemini-api-key` | `GEMINI_API_KEY`, `GOOGLE_API_KEY` | `gemini` | Fast/embedding tasks |
| `openai-api-key` | `PROVIDER_OPENAI_API_KEY` | (disabled) | Conditional; do not set `OPENAI_API_KEY` |
| `factory-api-key` | `FACTORY_API_KEY` | `droid` / `factory` | Specialised cheap tasks |
| `fireworks-api-key` | `FIREWORKS_API_KEY` | `opencode` | Kimi K2 tasks |
| `DO_INFERENCE_KEY` | `DO_INFERENCE_KEY` | `opencode` (tier-3 fallback) | DigitalOcean Gradient AI |
| `github-pat` | `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN` | `copilot`, `gh` CLI | GitHub / Copilot |
| `brave-api-key` | `BRAVE_API_KEY` | web-search tool | Search |
| `firecrawl-api-key` | `FIRECRAWL_API_KEY` | scraping tool | Scraping |

**Cursor:** No API key secret key today. Cursor authenticates via OAuth config in
`/workspace/.cursor/mcp.json` written by the init container. When a token-based API becomes
available, add `cursor-api-key` → `CURSOR_API_KEY` following the standard pattern.

### 4.2 Discord tokens

Agent-specific Discord bot tokens live in a separate Secret (`openclaw-discord-tokens`), keyed by
agent ID (e.g., `morgan`, `atlas`). This is distinct from provider API keys and is referenced via
`secrets.discordTokensSecret` in `values.yaml`.

### 4.3 Additional injected credentials

| Env var | Source | Purpose |
|---------|--------|---------|
| `ANTHROPIC_VERTEX_PROJECT_ID` | values literal | Anthropic via GCP Vertex AI |
| `ANTHROPIC_FOUNDRY_RESOURCE` | values literal | Anthropic via Azure AI Foundry |
| `ANTHROPIC_FOUNDRY_API_KEY` | Secret | Anthropic Azure Foundry key |
| `OP_SERVICE_ACCOUNT_TOKEN` | Secret | 1Password service account (runtime secret fetch) |

---

## 5. Readiness Checks

### 5.1 Current pod-level probes

From `values.yaml`:

| Probe | `initialDelaySeconds` | `periodSeconds` | `failureThreshold` | Effective window |
|-------|-----------------------|-----------------|--------------------|-----------------|
| Startup | 10 | 10 | 18 | 180 s |
| Liveness | — | 60 | 5 | 5 min to detect death |
| Readiness | 30 | 30 | 3 | 90 s to lose readiness |

These probes check the OpenClaw **gateway process**, not individual CLI backends.

### 5.2 Per-CLI health check commands

The following commands can be used to validate CLI availability before task assignment:

| CLI | Health check command | Expected output |
|-----|----------------------|-----------------|
| `claude` | `claude --version` | Version string |
| `codex` | `codex --version` | Version string |
| `copilot` | `gh copilot --version` | Version string |
| `cursor` | `cursor --version` | Version string |
| `gemini` | `gemini --version` | Version string |
| `opencode` | `opencode --version` | Version string |
| `droid` | `droid --version` | Version string |
| `dexter` | `dexter --version` | Version string |

After per-CLI binary check, `openclaw doctor` validates the full configuration including model
endpoint reachability. It should be run as a **pre-task hook** when routing to a new CLI, since it
tests the rendered config rather than just binary presence.

> **Note:** After `openclaw doctor` runs, the init template is restored to keep routing
> declarative. Do not rely on `openclaw doctor` output to mutate `/workspace/openclaw.json`
> permanently.

### 5.3 Recommendations

1. **Extend startup probe** when new CLIs (cursor, gemini, opencode) are added to `cliBackends` —
   each CLI registration adds startup latency.
2. **Add CLI readiness gate**: before the pod is marked Ready, run a lightweight CLI availability
   sweep (`which <binary> && <binary> --version`) in an init container or a pre-start lifecycle
   hook. Failed CLIs should be flagged in a status annotation, not cause pod failure.
3. **Expose per-CLI status** via a `/healthz/cli` sub-endpoint on the gateway so external probes
   (e.g., Kubernetes readiness probes, Argo health checks) can distinguish "gateway up" from
   "all CLIs available".

---

## 6. Failure Fallback Behaviour

### 6.1 Error taxonomy and actions

| Error class | Example signals | Recommended action |
|-------------|-----------------|-------------------|
| **Rate limit / quota** | HTTP 429, `RateLimitError`, `quota_exceeded` | Save mem0 checkpoint → rotate CLI (§3.2) |
| **Auth failure** | HTTP 401, HTTP 403, `invalid_api_key`, `authentication_required` | Mark CLI unavailable for session → rotate CLI; alert operator |
| **Quota billing hard limit** | `insufficient_quota`, `billing_hard_limit_reached` | Rotate CLI + page operator; do not retry same provider today |
| **Binary missing / not executable** | `command not found`, `errno: ENOENT` | Skip CLI in rotation; log as configuration error |
| **Startup / config error** | Non-zero exit on `--version` or `--help` | Skip CLI; flag in pod status annotation |
| **Model not found** | HTTP 404, `model_not_found`, `unknown_model` | Try model fallback within same CLI first; if no fallback, rotate CLI |
| **Context window exceeded** | `context_length_exceeded`, `max_tokens` | Summarise + truncate context; retry same CLI |
| **Timeout / hung session** | No output after `sessionTimeout`; exit code 124 | Kill session; restore from checkpoint; rotate CLI |
| **Network partition** | TCP timeout, DNS failure, `ConnectionRefused` | Retry with backoff (3×); then rotate CLI |
| **SIGKILL / pod eviction** | OOMKilled, pod restart | On next spawn, mem0 checkpoint recovery is the primary mitigation |

### 6.2 Fallback escalation flowchart

```
Task assigned to CLI X
  │
  ├─ Binary present & healthy? ──No──→ Skip X → try next in rotation chain
  │
  └─ Yes: run session
        │
        ├─ Success → done
        │
        ├─ Rate limit / quota → save checkpoint → rotate to next CLI → recover checkpoint
        │
        ├─ Auth failure → mark X unavailable → rotate → page ops
        │
        ├─ Context exceeded → truncate/summarise → retry same CLI
        │
        ├─ Model not found → try model fallback within X → if none, rotate CLI
        │
        └─ Timeout / crash → restore from checkpoint → rotate CLI
```

### 6.3 "All CLIs exhausted" handling

If every CLI in the rotation chain fails or is unavailable:

1. Set task status to `blocked`.
2. Post alert to Discord (`#incidents` or the active work channel).
3. Store partial results in mem0 with `status: "exhausted"` metadata.
4. Do **not** retry autonomously beyond the rotation chain — escalate to the human operator.

---

## 7. CRD Configurability

### 7.1 Current `cliBackends` in `values.yaml`

Only three CLIs are registered in `cliBackends` today:

```yaml
cliBackends:
  claude-cli:
    command: "/usr/local/bin/claude"
  codex-cli:
    command: "/usr/local/bin/codex"
  droid-cli:
    command: "/usr/local/bin/droid"
    args: ["--json"]
    output: "json"
    modelArg: "--model"
    sessionArg: "--session"
```

Missing registrations needed: `cursor-cli`, `gemini-cli`, `opencode-cli`, `copilot-cli`,
`dexter-cli`. Each should follow the same schema and add `healthCheckCommand` and `modelArg` where
applicable.

### 7.2 Proposed `cliBackends` extension schema

```yaml
cliBackends:
  cursor-cli:
    command: "/usr/local/bin/cursor"
    args: ["--headless"]
    modelArg: "--model"
    healthCheckCommand: "cursor --version"
    provider: "cursor"
  gemini-cli:
    command: "/usr/local/bin/gemini"
    modelArg: "--model"
    healthCheckCommand: "gemini --version"
    provider: "google"
  opencode-cli:
    command: "/usr/local/bin/opencode"
    modelArg: "--model"
    healthCheckCommand: "opencode --version"
    provider: "fireworks"
  copilot-cli:
    command: "/usr/local/bin/gh"
    args: ["copilot", "suggest"]
    healthCheckCommand: "gh copilot --version"
    provider: "github-copilot"
  dexter-cli:
    command: "/usr/local/bin/dexter"
    healthCheckCommand: "dexter --version"
    provider: "anthropic"
```

New fields:

- **`provider`**: links the CLI to a provider in `model-providers.json` for quota tracking.
- **`healthCheckCommand`**: command run during pod init sweep; failure sets CLI to `unavailable`.

### 7.3 Proposed `CodeRunSpec` fields (`crates/controller/src/crds/coderun.rs`)

Add to `CodeRunSpec`:

```rust
/// Preferred CLI backend for this run (e.g. "claude-cli", "copilot-cli").
/// If absent, the controller uses the default routing policy.
#[serde(skip_serializing_if = "Option::is_none")]
pub preferred_cli: Option<String>,

/// Ordered list of CLI backends to try if the preferred CLI is unavailable or exhausted.
/// Overrides the default rotation chain for this run.
#[serde(skip_serializing_if = "Option::is_none")]
pub provider_fallback_chain: Option<Vec<String>>,
```

Add to `CodeRunStatus`:

```rust
/// CLI backend actually used for this run (may differ from preferredCli after fallback).
#[serde(skip_serializing_if = "Option::is_none")]
pub active_cli: Option<String>,

/// Reason the active CLI differs from preferredCli, if applicable.
#[serde(skip_serializing_if = "Option::is_none")]
pub cli_fallback_reason: Option<String>,
```

Update the CRD YAML (`infra/charts/cto/crds/coderun-crd.yaml` and
`infra/charts/cto-lite/crds/coderun-crd.yaml`) to add:

```yaml
# under spec.properties:
preferredCli:
  type: string
  description: "Preferred CLI backend (e.g. 'claude-cli')"
providerFallbackChain:
  type: array
  description: "Ordered fallback CLI list for this run"
  items:
    type: string
# under status.properties:
activeCli:
  type: string
  description: "CLI backend actually used"
cliFallbackReason:
  type: string
  description: "Why the active CLI differs from preferredCli"
```

### 7.4 `skills/openclaw/acp-sessions.md` additions (recommended)

Add a **Quota Rotation** section to the ACP sessions skill documenting:

- When to trigger rotation (§3.4 signal table)
- Required mem0 save before switching (§3.3)
- Rotation chain priority order (§3.2)
- The `cliFallbackReason` status field to record in `CodeRunStatus`

---

## Summary of Gaps and Recommended Follow-Up Work

| Gap | Location | Priority |
|-----|----------|----------|
| `cursor-cli`, `gemini-cli`, `opencode-cli`, `copilot-cli`, `dexter-cli` not in `cliBackends` | `values.yaml` | High |
| `CURSOR_API_KEY` not injected (Cursor uses OAuth only) | `deployment.yaml` | Medium (blocked on Cursor API availability) |
| No `quotaRotation` block in `values.yaml` | `values.yaml` | Medium |
| `preferredCli` / `providerFallbackChain` missing from `CodeRunSpec` | `coderun.rs` + CRD YAMLs | Medium |
| Per-CLI health check sweep not in init container | `deployment.yaml` | Medium |
| `session-persistence.md` lacks quota-exhaustion signal section | skill file | Low |
| OpenAI (`codex`) re-enablement path not documented | `model-providers.json` | Low |
