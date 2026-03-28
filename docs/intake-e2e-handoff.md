# Intake E2E Test — Handoff Doc

**Date:** 2026-03-27
**Branch:** `intake/sigma-1-20260327-185814`
**Status:** Pipeline complete, Linear sync failed (401), uncommitted improvements pending

---

## What Was Accomplished

The last two sessions hardened the quick-run e2e path and verified the sigma-1 intake pipeline runs end-to-end:

1. **PR #4548** (`fix(intake): harden e2e quick-run workflow`) — merged. Fixes:
   - Late-stage task drift (downstream steps reparsing raw refinement output)
   - Pipeline-level LLM quality gate truncating oversized bundle → false negatives
   - `fan-out-prompts` crashing on provider failure instead of falling back to deterministic prompts

2. **PR #4549** (`feat(intake): sigma-1 task breakdown`) — open, created by the pipeline.
   - 6 tasks generated with 612 subtasks
   - All docs, prompts, workflows, acceptance criteria written
   - Handoff artifact gates passed: 6 tasks / 6 docs / 6 prompts / 6 workflows / 25 decision points

3. Pipeline ran to completion — failed only at the final `sync-linear` step (401 auth, see below).

---

## Current Failure: Linear 401

The last log line:
```
Fatal error: Linear API returned 401: Authentication required, not authenticated
```

The Linear session ID in `.intake/linear-session-id.txt`:
```
c30eac95-8679-4f16-bd30-a4fb846432b0
```

This session token is expired/invalid. Fix: refresh via `intake-util sync-linear` with a valid token, or re-run the pipeline with a fresh `op run` session that can pull a live Linear API key.

---

## Uncommitted Changes (working tree)

These are improvements discovered during the current e2e iteration. All unrelated to the 401 failure — they address correctness/robustness issues found during the run.

### `intake/workflows/task-refinement.lobster.yaml`
Added a `normalize_subtask_objects` jq function that handles the multiple subtask serialization shapes the LLM can return:
- Flat key-value array (alternating keys and values)
- JSON-encoded strings inside an array
- Proper object arrays (happy path)

Without this, subtasks with non-standard shapes silently become empty arrays.

### `intake/workflows/intake.lobster.yaml`
- **`fan-out-docs` fallback:** Instead of fatally exiting when `intake-util fan-out` fails or returns empty, now builds deterministic fallback docs from task metadata (title, description, subtasks, test_strategy). Keeps the pipeline moving.
- **`validate-docs` task ID extraction:** Changed from reparsing raw refinement output to reading the canonical expanded-task snapshot from file. Fixes task ID drift.
- **Comment added** to `verify-refine-tasks` jq chain explaining the two output formats handled.

### `apps/intake-util/src/validate.ts`
Enhanced `expanded-tasks` validation to check each subtask object for required fields (`id`, `title`, `description`, `dependencies`). Previously only checked that the subtasks array was non-empty.

### `intake/prompts/quality-gate-system.md`
Rewrote the quality gate prompt to be aware that bundles are **sampled** (do not penalize for truncated/missing tasks outside the sample). Added explicit scoring rubric for subtask counts, doc length, and placeholder detection. Prevents false negatives on valid large outputs.

### `intake/prompts/expand-task-system.md`
Changes here — check `git diff HEAD -- intake/prompts/expand-task-system.md` for details.

### `intake/schemas/generated-subtask.schema.json`
- Renamed `subagent_type` → `subagentType` (camelCase, consistent with LLM output)
- Added `subagentType` and `parallelizable` to `required` array
- `additionalProperties: false` enforced

---

## Next Steps

1. **Fix Linear auth** — refresh session token and run `intake-util sync-linear` to complete the last step. Or re-run `./intake/scripts/run-quick-intake.sh sigma-1` end-to-end once auth is confirmed working.

2. **Commit + PR the uncommitted changes** — these are substantive improvements. Suggested commit message: `fix(intake): subtask normalization, fan-out-docs fallback, quality gate prompt`

3. **Re-run clean e2e** — after committing, run `./intake/scripts/run-quick-intake.sh sigma-1` again to verify the full pipeline passes including Linear sync. Expected outcome: all steps green, tasks synced to Linear, PR created.

4. **Verify PR #4549** — once Linear sync works, check that sigma-1 tasks appear in Linear under the correct project/team.

---

## Key Files

| File | Role |
|------|------|
| `intake/scripts/run-quick-intake.sh` | E2E test entry point |
| `intake/workflows/intake.lobster.yaml` | Main intake workflow |
| `intake/workflows/task-refinement.lobster.yaml` | Task refinement sub-workflow |
| `intake/workflows/pipeline.lobster.yaml` | Top-level orchestrator |
| `.intake/quick-intake-run.log` | Last run log (1 line = failed at Linear) |
| `.intake/tmp/handoff-artifact-gates.json` | Artifact gate results from last run |
| `.intake/tmp/handoff-delivery-gates.json` | Delivery gate: PR URL + issue counts |
| `.tasks/docs/` | Generated task docs (6 tasks) |
| `.tasks/tasks/tasks.json` | Canonical task list |

---

## Run Command

```bash
cd /Users/jonathon/5dlabs/cto
./intake/scripts/run-quick-intake.sh sigma-1
```
