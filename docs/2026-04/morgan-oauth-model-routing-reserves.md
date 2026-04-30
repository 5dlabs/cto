# Morgan OAuth Model Routing and Reserves Handoff

**Owner / next reviewer:** Metal

**Date:** 2026-04-30

## Goal

Dogfood Morgan's promised model-routing feature: choose an OAuth-authenticated runtime/model for each play task based on task complexity, precision requirements, and remaining reserve capacity instead of always blasting the largest model or waiting for failures.

This document captures the current field findings and a proposed routing policy. It intentionally does **not** include secret values.

## Hard constraint

For play execution, count **OAuth-token runtimes only**.

Do **not** count direct API-key availability as sufficient for play capacity, even if those API keys are currently working. API-key probes are still useful for diagnostics, but play routing and reserve accounting should be based on OAuth-backed runtimes.

## Current OAuth-backed options verified

Verified from `openclaw-swarm-0` using non-secret probes.

### Claude Code OAuth

Available OAuth secret keys in `cto-coder-api-keys`:

- `anthropic-sub1-oauth`
- `anthropic-sub2-oauth`

Working models / aliases:

| Runtime | Model / alias | Notes |
|---|---|---|
| Claude Code | `sonnet` | Resolves to `claude-sonnet-4-6`; works. |
| Claude Code | `opus` | Resolves to `claude-opus-4-7`; works. |
| Claude Code | `claude-sonnet-4-6` | Works. |
| Claude Code | `claude-opus-4-6` | Works. |
| Claude Code | `claude-opus-4-7` | Works. |

Important: date-stamped future-style model IDs were rejected by Claude Code OAuth in this environment. Prefer short IDs / aliases for Claude Code:

- Use `claude-opus-4-7`, not `claude-opus-4-7-20260610`.
- Use `claude-sonnet-4-6`, not `claude-sonnet-4-6-20260514`.

### Codex OAuth

Available OAuth auth-json secret keys in `cto-coder-api-keys`:

- `codex-auth-sub1`
- `codex-auth-sub2`

Working ChatGPT/Codex OAuth models:

| Runtime | Model | Notes |
|---|---|---|
| Codex | `gpt-5.2` | Works. |
| Codex | `gpt-5.4` | Works. |

Unsupported with the current ChatGPT/Codex OAuth account:

- `gpt-5.2-codex`
- `gpt-5.1-codex`
- `gpt-5`
- `gpt-5.1`
- `o3`
- `o4-mini`

Operational note: `openclaw-swarm-0` had a stale `/workspace/.codex/auth.json`; installing the refreshed JSON from `cto-coder-api-keys` made Codex OAuth work. The chart/controller should ensure the pod auth file is refreshed from the active OAuth secret, not left stale on the workspace volume.

## Current reserve visibility

There is no confirmed clean remaining-balance API from either Claude Code OAuth or Codex OAuth yet.

What is visible now:

### Claude Code

`claude -p ... --output-format json` returns per-run usage and cost:

- `total_cost_usd`
- `usage`
- `modelUsage`
- resolved model
- cache read/create token counts

Tiny field probes showed roughly:

| Alias | Resolved model | Tiny probe cost |
|---|---|---:|
| `sonnet` | `claude-sonnet-4-6` | about `$0.0055` |
| `opus` | `claude-opus-4-7` | about `$0.0121` |

These are not meaningful task estimates by themselves because prompt caching and workspace context dominate, but they prove cost telemetry is accessible per run.

### Codex

`codex exec --json` returns per-run token usage:

- `input_tokens`
- `cached_input_tokens`
- `output_tokens`

A tiny `gpt-5.2` probe emitted usage like:

- `input_tokens`: `28790`
- `cached_input_tokens`: `28672`
- `output_tokens`: `5`

No dollar cost or remaining balance was observed from Codex CLI. Morgan can still track relative burn from token usage and maintain an internal budget ledger.

## Proposed Morgan feature behavior

Morgan should maintain a routing decision object for each play stage:

```json
{
  "taskClass": "simple|standard|large|high-risk|review|security|test-fix",
  "risk": "low|medium|high",
  "complexity": "small|medium|large",
  "primary": { "runtime": "codex", "model": "gpt-5.2", "oauthPool": "codex" },
  "fallbacks": [
    { "runtime": "claude", "model": "claude-sonnet-4-6", "oauthPool": "anthropic" },
    { "runtime": "codex", "model": "gpt-5.4", "oauthPool": "codex" }
  ],
  "reserveProtected": [
    { "runtime": "claude", "model": "claude-opus-4-7" }
  ],
  "escalationTriggers": [
    "same_test_failure_after_2_attempts",
    "no_meaningful_diff",
    "auth_or_provider_ambiguity",
    "large_unreviewed_diff"
  ]
}
```

## Proposed routing policy

### Tier A: daily drivers

Use for most work and as the first pass for ordinary implementation:

- Codex `gpt-5.2`
- Claude Code `claude-sonnet-4-6` / `sonnet`

Expected share: about 70% of tasks.

### Tier B: escalation / precision

Use when complexity or ambiguity is higher, or Tier A shows concrete failure signals:

- Codex `gpt-5.4`
- Claude Code `claude-opus-4-6`

Expected share: about 20–25% of tasks.

### Tier C: protected reserve

Keep available for must-fix, high-risk, or deeply ambiguous work:

- Claude Code `claude-opus-4-7` / `opus`

Expected share: about 5–10% of tasks.

## Task-class mapping

| Task class | Primary | Fallback | Escalation / reserve |
|---|---|---|---|
| Simple docs/config/mechanical edits | Claude `sonnet` | Codex `gpt-5.2` | none unless tests fail repeatedly |
| Standard feature implementation | Codex `gpt-5.2` | Claude `sonnet` | Codex `gpt-5.4` |
| Large multi-file implementation | Codex `gpt-5.2` | Codex `gpt-5.4` | Claude `opus` |
| High-risk architecture/root-cause debugging | Claude `opus` | Codex `gpt-5.4` | Claude `sonnet` for cleanup |
| Review after Codex implementation | Claude `sonnet` | Claude `opus` for risky/security changes | n/a |
| Review after Claude implementation | Codex `gpt-5.2` | Codex `gpt-5.4` | n/a |
| Test-fix loop with exact failure | Claude `sonnet` | Codex `gpt-5.2` | Opus only after repeated non-convergence |
| Security-sensitive change | Claude `opus` | Codex `gpt-5.4` | independent review with Claude `sonnet` |

## Escalation triggers

Escalate before total failure when any of these occurs:

- Same test failure after two fix attempts.
- Agent produces no meaningful diff after a complete run.
- Agent modifies too many files relative to task size.
- Agent invents unsupported model/provider/API names.
- Auth, OAuth, provider, Kubernetes, or GitOps ambiguity is central to the task.
- Patch grows beyond a review threshold, e.g. more than 10 files or high-risk ownership boundaries.

## De-escalation triggers

Use cheaper/daily-driver options when:

- The failure is deterministic and localized.
- The task is docs/config/test snapshot maintenance.
- A human or previous agent already identified the exact fix.
- The next step is cleanup, formatting, or applying known diffs.

## Fallback should not mean blind retry

Fallback should preserve context and diagnosis:

1. Capture failing command, exact error, and changed files.
2. Summarize what the previous model attempted.
3. Ask the fallback model for a bounded next action, not a full restart.
4. Limit each fallback to a fixed attempt budget.

## Reserve accounting recommendation

Until true account balance is available, Morgan should keep an internal ledger:

- For Claude: record `total_cost_usd`, resolved model, input/output/cache token counts per play stage.
- For Codex: record `input_tokens`, `cached_input_tokens`, `output_tokens`, model, and duration per play stage.
- Maintain rolling windows per OAuth pool: last 1h, 24h, 7d.
- Protect at least one premium lane (`claude-opus-4-7` or `gpt-5.4`) for incident/high-risk use.
- Add manual override fields for Metal to inject actual account balances/API-key dashboard readings when available.

Suggested reserve object:

```json
{
  "oauthPools": {
    "anthropic-sub1": {
      "runtime": "claude",
      "knownBalance": null,
      "observedSpendUsd24h": 0,
      "protectedModels": ["claude-opus-4-7"]
    },
    "anthropic-sub2": {
      "runtime": "claude",
      "knownBalance": null,
      "observedSpendUsd24h": 0,
      "protectedModels": ["claude-opus-4-7"]
    },
    "codex-sub1": {
      "runtime": "codex",
      "knownBalance": null,
      "observedTokens24h": 0,
      "protectedModels": ["gpt-5.4"]
    },
    "codex-sub2": {
      "runtime": "codex",
      "knownBalance": null,
      "observedTokens24h": 0,
      "protectedModels": ["gpt-5.4"]
    }
  }
}
```

## Implementation notes for Metal

Likely useful areas to inspect:

- `model-providers.json`
- `cto-config.json`
- `crates/controller/src/crds/coderun.rs`
- `crates/controller/src/tasks/code/templates.rs`
- OpenClaw/Morgan chart values under `infra/gitops/agents/`
- Any play scheduler / agent-harness code that selects CLI/model per task stage

Key implementation questions:

1. Where should Morgan persist the observed usage ledger?
2. Should routing be deterministic config, learned policy, or a hybrid?
3. How should a user override routing per play?
4. How should Morgan expose “reserves” in Discord/UX?
5. How should refreshed OAuth auth-json be mounted into runtime pods to prevent stale workspace auth?

## Acceptance criteria

- Play routing only counts OAuth-token backed runtimes.
- Morgan can list currently usable OAuth-backed runtimes/models without revealing secrets.
- Morgan records usage after every model run.
- Morgan selects a default model based on task class, not a fixed global default.
- Morgan escalates based on explicit triggers, not only final failure.
- Morgan keeps premium models as protected reserve unless task risk justifies use.
- Metal can optionally provide true dashboard/API balance data and have Morgan incorporate it.
