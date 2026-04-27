# Avatar OpenClaw Autonomy Policy

**Status:** Active
**Version:** 1.0
**Scope:** Morgan avatar GLB/VRM runtime asset objective + all future OpenClaw/OpenClient objectives
**Live standing orders:** `infra/gitops/agents/morgan-values.yaml` -> `workspace.files.AGENTS.md`
**Reference skill:** `infra/charts/openclaw-agent/skills/openclaw/autonomy-policy.md`

---

## Summary

This policy defines how the OpenClaw conductor operates without human intervention. The agent drives toward a concrete, verifiable objective, chooses reasonable defaults autonomously, retries through a structured ladder, rotates CLIs/providers when needed, and escalates only for genuine hard blockers (irreversible actions, exhausted retry ladder, missing credentials after documented attempts).

OpenClaw standing orders are loaded from workspace files, typically `AGENTS.md`. The Morgan deployment now carries the concise runtime version there; this document and the OpenClaw skill file are the expanded reference.

## Current Primary Objective

**Browser-controllable Morgan GLB/VRM-style runtime asset**

Done condition:
1. GLB/VRM candidate generated via Scenario-hosted models and uploaded to the Scenario project
2. Scenario `asset_id`, provenance, and generation parameters recorded in the asset ledger
3. Candidate bundle includes inventories for skeleton, morph targets, animation clips, viseme mapping, validation, provenance, and render artifacts
4. Asset renders in a browser WebGL canvas without console errors and passes the runtime validator gates
5. (Auxiliary) MP4 preview generated and linked when requested, but not as a blocker for the runtime asset

## Policy Sections

| Section | Location in skill |
|---|---|
| Objective contract | §1 |
| Allowed vs prohibited decisions | §2 |
| Escalation criteria | §3 |
| Retry/fallback ladder | §4 |
| Stop conditions & loop guard | §5 |
| Context compaction / restart continuity | §6 |
| Human-visible progress cadence | §7 |
| Safety / cost / secret guardrails | §8 |
| Standing orders (CRD-ready YAML) | §9 |

## CRD Integration Path

The `spec.autonomyPolicy` YAML block in §9 is ready for embedding into the `CodeRun` or a new `AgentPolicy` CRD when the controller is extended. Current manual application: inject via Helm values under `extraEnv` or mount as a ConfigMap and reference from the agent's system prompt.

## Key Defaults

| Decision | Default | Override |
|---|---|---|
| Generation provider priority | Scenario → Hyper3D → Hunyuan3D | HANDOFF.md `CONSTRAINT:` |
| Output format | GLB (browser compat) | Explicit task instruction |
| Resolution (iteration) | 1k | Final: 2k |
| Max cost per run | $10 | Human instruction |
| Loop guard (steps) | 50 without milestone | HANDOFF.md `LOOP_GUARD:` |
| Progress channel | `#agent-coordination` | Per-agent config |

## Standing-order deployment path

OpenClaw’s automation docs distinguish design guidance from standing orders:

- **Standing orders** live in workspace files such as `AGENTS.md` and are injected into every session automatically.
- **Heartbeat** reads `HEARTBEAT.md` and should advance active work when the next action is inferable.
- **Task Flow / background tasks** should track durable multi-step work and detached sub-agents.

For Morgan, the live standing orders are embedded in:

```text
infra/gitops/agents/morgan-values.yaml
  workspace.files.AGENTS.md
  workspace.files.HEARTBEAT.md
```

They instruct Morgan to keep working from the detailed avatar plan, use Scenario first, persist state in ledgers and memory, rotate CLIs/providers with checkpoints, enforce cost/loop guards, and escalate only for serious blockers.

## Relationship to Existing Skills

| Skill | Relationship |
|---|---|
| `reboot-continuity.md` | Implements §6 (restart protocol); autonomy-policy adds objective contract and loop guard |
| `orchestrator.md` | Implements sub-agent delegation; autonomy-policy adds retry ladder Levels 4-5 |
| `provider-failover.md` | Implements credential rotation; autonomy-policy references as Level 4 |
| `status-reactions.md` | Implements Discord posting; autonomy-policy specifies when to post (§7) |
