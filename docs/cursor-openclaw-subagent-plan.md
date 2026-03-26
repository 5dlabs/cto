# Cursor ↔ OpenClaw sub-agent plan

This doc describes an **operating model**: **dedicated Cursor subagents** **monitor and remediate** the matching **OpenClaw agents** in-cluster or on the gateway, so runtime failures surface and get fixed without one overloaded chat doing everything.

## Cursor-native subagents (best practices)

This repo follows **[Cursor Subagents](https://cursor.com/docs/subagents)**:

| Practice | How we apply it |
|----------|------------------|
| **Focused agents** | Two custom definitions in **`.cursor/agents/`** — not a dozen vague “helpers.” |
| **Strong `description`** | Frontmatter descriptions say *when* to delegate (“Morgan/intake”, “named OpenClaw agent + context”). |
| **Context isolation** | Shadows run in their own session; parent gets a **compact handoff**. |
| **Parallel work** | Parent delegates multiple **background** shadows when several agents are hot on a Play. |
| **Skills vs subagents** | One-off scripts / formatting → [skills](https://cursor.com/docs/skills); long monitoring / parallel tracks → subagents. |
| **Anti-pattern** | Avoid 50+ overlapping subagents or “use for everything” descriptions — see Cursor docs. |

**Project files:** [`morgan-intake-shadow`](../.cursor/agents/morgan-intake-shadow.md), [`openclaw-line-shadow`](../.cursor/agents/openclaw-line-shadow.md). Invoke with `/morgan-intake-shadow` or `/openclaw-line-shadow` per [docs](https://cursor.com/docs/subagents#explicit-invocation).

---

## Why

- **OpenClaw agents** do implementation work **asynchronously** (pods, gateway sessions, Lobster steps).
- **One** Cursor chat cannot continuously tail **every** workload, Discord room, and Linear thread at once.
- **Pairing** each OpenClaw identity with a **Cursor counterpart** keeps **ownership**, **context**, and **remediation** local to that slice of the system.

---

## Roles

| Role | Lives in | Responsibility |
|------|----------|----------------|
| **Conductor (intake coordinator)** | Primary Cursor session | Intake pipeline, checkpoints, bridges, cross-cutting Git/Helm/workflow; delegates or spawns line monitors when needed. See [`intake-coordinator.md`](intake-coordinator.md). |
| **Line shadow** | **Cursor subagent** (`.cursor/agents/*.md`) or Task delegation | One OpenClaw agent identity: **observe** its logs + outputs + Discord/Linear surfaces it touches; **remediate** within that domain; **escalate** only after reasonable defaults + fallback (per coordinator agency rules). |
| **Human** | — | Approval gates, MFA, irreversible blast radius, product scope. |

**Healer** (OpenClaw) remains the **in-cluster** automated remediator; Cursor shadows **audit**, **correlate externally** (UI, local tools), and **patch repo/config** when the fix is code or GitOps.

---

## Mapping: OpenClaw agent → Cursor shadow focus

Use this when spawning a **Task** or **separate Cursor chat** for sustained monitoring. **Specialty** hints which repo areas and **`.codex/agents/*.md`** / **`.cursor/skills/*`** to load.

| OpenClaw agent | Cursor shadow monitors… | Typical remediation surface |
|----------------|-------------------------|------------------------------|
| **Morgan** | Intake / Lobster / PRD steps, Linear project/session, `#intake` | `intake/workflows`, bridges, `intake-util`, `cto-config` Linear defaults |
| **Atlas** | Merge gate workflows, branch/PR health | Merge workflows, CI rules, GitHub integration |
| **Stitch** | Review jobs, PR comments, review policy | Review automation, prompts, `intake`/PR templates |
| **Rex** | Rust CodeRuns, build failures | `**/Cargo.toml`, Rust services, Rex skill |
| **Blaze** | Frontend CodeRuns, Next/React | `apps/*` web, Blaze skill |
| **Grizz** | Go CodeRuns | Go services, Grizz skill |
| **Tess** | Test CodeRuns, coverage | Test harnesses, Tess skill |
| **Cleo** | Lint/format gates | Linters, Cleo skill |
| **Cipher** | Security scans | Security tooling, Cipher skill |
| **Healer** | Remediation loops, alarms | Cross-reference with cluster events; don’t duplicate Healer’s in-band fixes |
| **Bolt** | Infra applies, Helm, Argo | `infra/`, Helm, Bolt skill |
| **Angie** | Agent/orchestration changes | OpenClaw configs, agent architecture docs |
| **Keeper** | Cluster hygiene, cron operators | Operational runbooks, Keeper skill |
| **Nova** | Research tasks | Docs research outputs |
| **Spark** | Prototype CodeRuns | Experimental paths |
| **Tap** | Integration/webhook CodeRuns | API clients, webhooks |
| **Vex** | Debug/incident CodeRuns | Diagnostics, Vex skill |
| **Pixel** | Desktop Tauri | `apps`/Tauri, Pixel skill |

Shadows are **not** required to be **always-on** for every agent on every day—spin them up when that agent is **active on a Play** or when **incidents** concentrate there.

---

## What a line shadow does (loop)

1. **Bind signals:** `kubectl logs` (or Loki query) for the OpenClaw workload **labeled or named** for that agent; **Discord** thread/room the workflow uses; **Linear** issues/sessions that agent owns.
2. **Baseline + correlate:** Note timestamps; when OpenClaw posts “step X failed,” the shadow finds the **same minute** in logs and UI.
3. **Remediate:** Apply **reasonable default first** (config, missing secret via `op`, small code fix), **one fallback**, then escalate with **evidence list**—same rules as [`intake-coordinator.md`](intake-coordinator.md) § Agency.
4. **Hand back:** When the Play advances, the shadow can **summarize** residual risks to the **conductor** or **human**.

---

## Conductor vs many shadows

- **Conductor** owns **intake end-to-end** and **shared plumbing** (bridges, preflight, `pipeline.lobster.yaml`).
- **Shadows** own **deep continuity** for **one** OpenClaw identity during a heavy run.
- Avoid **N shadows for N agents** when **N=16** and only **two** agents are hot—scale fan-out to **activity**, not headcount.

Use **Cursor subagents** ([docs](https://cursor.com/docs/subagents)) in **background** mode for long monitors, or the **Task tool** with the same prompts. Each spawn should include **this row** of the mapping table + **run id / namespace** for the bound OpenClaw agent.

---

## Future automation (optional)

- Label Kubernetes pods or log streams with **`agent: morgan`** consistently for cheap filtering.
- Optional **Meta** shadow that only watches **Healer** + **conductor state** (still human-triggered).
- Do **not** require this plan for **every** local test; it targets **multi-agent Plays** and **production-like** intake.

---

## See also

- [`AGENTS.md`](../AGENTS.md) — canonical roster  
- [`docs/intake-coordinator.md`](intake-coordinator.md) — conductor protocol  
- [`docs/intake-observer.md`](intake-observer.md) — observation checklist  
- [`.cursor/skills/cto-platform/SKILL.md`](../.cursor/skills/cto-platform/SKILL.md) — platform lifecycle  
