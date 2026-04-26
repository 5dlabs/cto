---
title: CTO Ralph Methodology
description: CTO-specific Ralph loop design for autonomous Play lifecycle testing
---

# CTO Ralph Methodology

This document adapts the Ralph methodology from the transcript in
`docs/youtube-4Nna09dG_c0-transcript.md` to the CTO Play workflow and the
lifecycle gates in `docs/workflow-lifecycle-checklist.md`. It is the operating
model for autonomous lifecycle testing in `lifecycle-test/`.

## Core Principles (from the transcript, applied to CTO)

1. **Two-session model (spec vs execution).**
   - **Spec session** builds a stable "pin" (lookup/spec index) that can be
     rehydrated later.
   - **Execution session** handles one objective per loop to minimize context
     sliding and compaction.
2. **Deterministic loop, single objective.**
   - Each loop answers one question or satisfies one acceptance gate.
   - No multi-step wandering. Backpressure is applied by restating the single
     objective on every iteration.
3. **Low control, high oversight.**
   - The loop can choose the best next action, but it must always meet
     explicit gates (preflight checks, phase checks, verification evidence).
4. **Context as a searchable array.**
   - The "pin" is a lookup index, not a full conversation log.
   - The execution prompt stays compact and points to the pin file as needed.
5. **Always rehydrate from artifacts.**
   - Progress, state, and evidence must be written to disk so the loop can
     resume deterministically after a restart.

## CTO Mapping: Ralph Loop to Play Workflow

The CTO lifecycle already defines phase gates and expected artifacts in
`docs/workflow-lifecycle-checklist.md` and observability expectations in
`docs/heal-play.md`. The Ralph loop should treat these as non-negotiable gates:

```
Intake -> Play -> Quality -> Security -> Testing -> Integration -> Deploy -> PostFlight
```

Each phase is executed as a **single objective loop** with explicit verification
steps and a cleanup fallback.

## Artifact Strategy (Pin + Execution)

### Pin File (Stable Context)
Location: `lifecycle-test/pin.md`

Contents should remain stable and only evolve when the system changes:
- Project context (AlertHub, agent stack, key paths)
- Gate checklist references and commands
- Cleanup procedures and log locations
- Branch/commit rules and verification requirements

### Pin Lookup Index (Search Hints)
Location: `lifecycle-test/pin.lookup.md`

Purpose:
- Map core concepts to aliases and pointer paths
- Boost search hit rates and reduce invention
- Keep agent lookups deterministic across runs

Format:
- One section per concept, with `aliases` and `pointers`
- Update only when system structure changes

### Implementation Plan (Linked Objectives)
Location: `lifecycle-test/implementation-plan.md`

Purpose:
- Single source of objective ordering
- Strong linkage to specs and code targets
- Enables objective selection from a stable list

Format:
- `- [ ] <phase_id> | <objective>`
- Sub-bullets for spec/code/evidence references

### Execution Prompt (Loop Objective)
Location: `lifecycle-test/prompt.md`

This prompt stays compact and only directs the agent to:
- Read the pin file for reference
- Execute **one objective** from `current-objective.md`
- Record evidence to `report.json` and `progress.txt`
- Stop and clean up on failure before retrying

The objective is selected from `implementation-plan.md` and written to
`current-objective.md` by the runner.

## Preflight Gates (Hard Stop)

Before any phase, enforce preflight checks based on:
- `docs/workflow-lifecycle-checklist.md` (services, tunnels, credentials)
- `docs/heal-play.md` (tool inventory and MCP readiness signals)

If any preflight check fails, the loop should stop immediately and record the
failure in `report.json` with log evidence.

## Phase Gates (Hard Stop)

Use phase-specific gates from `docs/workflow-lifecycle-checklist.md`. The loop
must verify every gate with evidence (log output or command output) before
moving on.

| Phase | Examples of required gates (non-exhaustive) |
| --- | --- |
| Intake | Intake CodeRun completes; `tasks.json` exists; Linear issues created |
| Play | CodeRun created for selected task; prompt rendered; tools available |
| Quality | Cleo review posted; language-appropriate lint checks run |
| Security | Cipher scan run; no critical vulnerabilities |
| Testing | Tess runs tests; results recorded; no failing tests |
| Integration | Atlas rebase/merge succeeds; CI checks pass |
| Deploy | Bolt deploy task applied; service health checks pass |
| PostFlight | Telemetry captured; Linear activity timeline complete |

## Backpressure, Cleanup, and Retry

Each loop is responsible for **one gate only**. If a gate fails:
1. Capture evidence (logs + command output)
2. Run cleanup for that phase
3. Record the failure to `report.json` and `progress.txt`
4. Retry only after the system is clean

This prevents duplicate CodeRuns and keeps the system deterministic.

## Attended → Unattended Guardrail

Early loops should be observed. The runner enforces a required number of
attended runs before unattended execution. Use `--attended` to explicitly
permit early runs while the guardrail is active.

## PII-Safe Evidence

All command outputs and report entries should avoid PII and secrets. The runner
applies redaction patterns before storing log files or report entries.

## Observability Contract

The loop must surface evidence in a structured way, based on the observability
expectations in `docs/heal-play.md`:

- **Local services:** `/tmp/cto-launchd/*.log`
- **Kubernetes:** `kubectl get coderuns,pods -n cto` and pod logs
- **Linear:** agent dialog and activity signals
- **GitHub:** PR status and checks

## Reporting Contract

The loop must write a machine-readable report plus a human-readable progress log.

**Report file:** `lifecycle-test/report.json`

Suggested entry format:
```json
{
  "timestamp": "2026-01-17T20:12:55Z",
  "phase": "intake",
  "status": "failed",
  "objective": "Verify intake CodeRun completes",
  "evidence": {
    "command": "kubectl logs -n cto -l type=intake --tail=100",
    "excerpt": "AI response parse error: Failed to parse AI response as JSON"
  }
}
```

**Progress file:** `lifecycle-test/progress.txt`

Each phase should add a short, timestamped summary and link to evidence (issue,
PR, log snippet) so a human can audit the run quickly.

## Deterministic Resume

The wrapper must persist state between iterations. A minimal state structure:
```json
{
  "phase": "intake",
  "attempts": { "intake": 2, "play": 0 },
  "completedObjectives": ["intake"],
  "attendedCompleted": 1,
  "last_success": "2026-01-17T19:42:12Z"
}
```

On restart, the wrapper must rehydrate state and continue from the last
incomplete phase.

## Where This Is Implemented

- Wrapper config: `lifecycle-test/ralph-cto.json`
- Wrapper runner: `scripts/2026-01/ralph-cto.sh`
- Pin file: `lifecycle-test/pin.md`
- Pin lookup index: `lifecycle-test/pin.lookup.md`
- Implementation plan: `lifecycle-test/implementation-plan.md`
- Execution prompt: `lifecycle-test/prompt.md`
- Objective file: `lifecycle-test/current-objective.md`
- Report: `lifecycle-test/report.json`

## Standardization Path

Once the lifecycle test confirms stability, apply the same artifacts and
guardrails to all agent templates so the entire platform uses one Ralph loop
standard.

