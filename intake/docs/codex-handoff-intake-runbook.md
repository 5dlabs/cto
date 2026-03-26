# Codex Handoff: Intake E2E Hardening Runbook

This runbook is for resuming intake stabilization with low-token, no-voice operation and strict artifact quality checks.

## Goal

Get deterministic end-to-end intake runs that produce:

- non-empty `.tasks` artifacts,
- parent/task/subtask hierarchy in Linear,
- branch-valid links in Linear issue bodies,
- reliable assignment diagnostics,
- strict quality gates (deterministic + LLM-assisted).

## Operating mode

- Use **quick path** first (`deliberate=false`) to reduce cycle time and token burn.
- Keep **voice disabled** (already configured in scripts/workflows).
- Prefer visual verification via logs + generated artifacts.

## Main scripts

- Quick run: `intake/scripts/run-quick-intake.sh`
- Full run: `intake/scripts/run-full-e2e.sh`
- Checkpoints: `intake/scripts/iteration-checkpoints.sh`
- LLM quality gate helper: `intake/scripts/quality-gate.sh`

## Critical env flags

- `INTAKE_STRICT_CONTENT_GATES=true`  
  Fail when docs/prompts/workflows are malformed/empty.

- `INTAKE_REQUIRE_SUBTASKS=true`  
  Fail when refinement yields zero subtasks.

- `INTAKE_ENABLE_LLM_GATES=true`  
  Enable quality-gate LLM checks.

- `INTAKE_STRICT_ASSIGNMENTS=false` (default recommended initially)  
  If true, fail when unresolved Linear agent assignments exist.

- `INTAKE_FAN_OUT_CONCURRENCY=2` (quick profile default)  
  Conservative parallelism.

## Stage flow to follow

1. **Stage A: preflight sanity**
   - Run checkpoints twice:
     - `op run --env-file=intake/local.env.op -- env WORKSPACE="$PWD" INTAKE_PREFLIGHT_BRIDGES_SKIP=true bash intake/scripts/iteration-checkpoints.sh`
   - Need two consecutive green runs.

2. **Stage B: quick intake (`deliberate=false`)**
   - Run:
     - `bash intake/scripts/run-quick-intake.sh`
   - Verify:
     - `.intake/intake-summary.json`
     - `.tasks/tasks/tasks.json`
     - `.tasks/docs/task-*/{task.md,acceptance.md,prompt.md}`
     - no `.tasks/docs/task-undefined`

3. **Stage C: sync proof (`sigma-1`)**
   - Confirm branch + PR in `/Users/jonathon/5dlabs/sigma-1`.
   - Confirm Linear hierarchy:
     - parent issue,
     - task child issues,
     - subtask child issues when present.

4. **Stage D: full regression (`deliberate=true`)**
   - Run full profile only after quick path is green.
   - Re-verify same artifact and delivery gates.

## What was changed already

### Quality/strictness

- Added strict validation behavior and subtask enforcement in intake workflow:
  - `intake/workflows/intake.lobster.yaml`
  - `intake/workflows/task-refinement.lobster.yaml`
- Added LLM quality gate assets:
  - `intake/scripts/quality-gate.sh`
  - `intake/prompts/quality-gate-system.md`
  - `intake/schemas/quality-gate.schema.json`
- Added pipeline-level quality gate steps + stricter intake output checks:
  - `intake/workflows/pipeline.lobster.yaml`

### Linear sync/hierarchy/assignment

- Hardened agent extraction + normalization.
- Added parent issue creation + stats in output (task/subtask/assignment counts).
- Added embedded doc excerpts and PR link support in issue body.
- Added unresolved assignment diagnostics.
  - File: `apps/intake-util/src/sync-linear.ts`
  - CLI passthrough for `--pr-url`: `apps/intake-util/src/index.ts`

### Branch-valid link sequencing

- Deferred Linear issue sync to post-push step:
  - `sync-linear-issues-post-push` in `intake/workflows/intake.lobster.yaml`

## Current known failure pattern

Recent failures showed:

- `fan-out-docs: FATAL — intake-util fan-out failed (Invalid JSON input)`

Observed root cause:

- `refine-tasks` output can be concatenated multi-JSON (e.g., `null` + array) in some degradation paths, causing downstream JSON parsing errors.

Mitigations already added:

- Robust extraction of `expanded_tasks` via `jq -cs ...` in fan-out stages.
- Debug capture files:
  - `.intake/tmp/fanout-docs-expanded-tasks.json`
  - `.intake/tmp/fanout-prompts-expanded-tasks.json`
- Normalization + fallback doc/prompt generation in `validate-docs` and `validate-prompts`.

Next action after handoff:

- Re-run quick intake and confirm fan-out no longer fails.
- If it still fails, inspect:
  - `.intake/tmp/fanout-docs-expanded-tasks.json`
  - `.intake/tmp/refine-tasks-input.json`
  - `.intake/intake-sub-workflow.log`

## Linear agent assignment checklist

Reference:

- `intake/docs/linear-agent-assignment-checklist.md`

Use this before enabling `INTAKE_STRICT_ASSIGNMENTS=true`.

## Verification checklist (must pass)

- `task_count > 0`
- `subtask_count > 0` (unless intentionally disabled)
- no empty required files under `.tasks/docs/task-*`
- no `task-undefined` paths
- non-zero Linear `issueCount`
- unresolved assignment diagnostics are visible and actionable
- PR URL/branch links in Linear resolve to existing branch content

## If stuck

1. Keep strict gates on, but temporarily set:
   - `INTAKE_STRICT_ASSIGNMENTS=false` (only assignment strictness)
2. Do not disable content/subtask gates unless debugging parser-level failures.
3. Fix the earliest failing gate, re-run quick path, then continue.

