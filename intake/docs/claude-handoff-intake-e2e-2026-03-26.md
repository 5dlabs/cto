# Claude Handoff: Intake E2E Status (2026-03-26)

## Current status

- Latest `bash intake/scripts/run-quick-intake.sh` completed far enough to produce a fresh intake summary and the canonical artifact set:
  - `.intake/intake-summary.json`
  - `.tasks/tasks/tasks.json`
  - `.tasks/docs/task-1` through `.tasks/docs/task-6`
- Current artifact manifest is stable again:
  - `task_count = 6`
  - `subtask_count = 6`
  - no `task-undefined`
  - no stray `task-10+` directories
- The local target repo was updated:
  - repo: `/Users/jonathon/5dlabs/sigma-1`
  - branch: `intake/sigma-1-20260326-175934`
  - latest commit: `8dd4a52 feat(intake): generated 6 tasks for sigma-1`

## What was fixed in this session

### 1. Late-stage task drift in intake workflow

File:
- `intake/workflows/intake.lobster.yaml`

Problem:
- Downstream steps were reparsing `CTO_REFINE_TASKS_OUT` and could resurrect non-canonical task sets late in the run.
- This is what previously rewrote `.tasks/tasks/tasks.json` with the wrong task list and recreated bogus `task-10+` docs.

Fix:
- `verify-artifact-gates`
- `commit-outputs`
- `create-pr`
- `sync-linear-issues-post-push`

These now prefer the canonical snapshot chain:
1. `.intake/tmp/fanout-docs-expanded-tasks.json`
2. `.intake/tmp/fanout-prompts-expanded-tasks.json`
3. `.intake/tmp/generate-scale-expanded-tasks.json`
4. `.tasks/tasks/tasks.json`
5. `.intake/tmp/refine-tasks-input.json`
6. `.intake/initial-tasks.json`

Result:
- `.tasks/tasks/tasks.json` now lands on the expected 6-task manifest instead of the bogus late-stage task expansion.

### 2. Pipeline intake quality-gate truncation

File:
- `intake/workflows/pipeline.lobster.yaml`

Problem:
- `quality-gate-intake-output.txt` was too large.
- `quality-gate.sh` truncates content above 14k chars.
- The model failed the stage because the bundle looked incomplete/truncated.

Fix:
- Rebuilt the bundle into a compact all-task snapshot:
  - compact summary
  - compact `task_summaries` for all tasks
  - compact per-task prompt/doc excerpts for every task

Verification:
- Manual rerun of:
  - `intake/scripts/quality-gate.sh --stage pipeline-intake-output ...`
- Result:
  - `{"pass": true, "score": 9, ...}`

### 3. Prompt fan-out robustness

File:
- `intake/workflows/intake.lobster.yaml`

Problem:
- `fan-out-prompts` hard-failed when one item exhausted providers.
- This blocked the full intake even though `validate-prompts` already had fallback logic later.

Fix:
- Added deterministic fallback prompt generation directly inside `fan-out-prompts`.
- If `intake-util fan-out` fails or returns empty output, the step now emits fallback prompt objects instead of aborting.

Result:
- Prompt generation is now resilient to partial provider failure.

## Verified outcomes

- Current `.tasks/tasks/tasks.json` contains 6 tasks with IDs `[1,2,3,4,5,6]`.
- Current `.tasks/docs` contains only:
  - `task-1`
  - `task-2`
  - `task-3`
  - `task-4`
  - `task-5`
  - `task-6`
- Latest summary:
  - `artifact_counts.task_count = 6`
  - `artifact_counts.doc_count = 6`
  - `artifact_counts.prompt_count = 6`
  - `artifact_counts.workflow_count = 6`
  - `delivery.issue_count = 7`
  - `delivery.pr_url = "none"`

## Known remaining risks / follow-ups

### A. Upstream task-refinement still drifts task count

This is not fully solved.

Observed earlier in the run loop:
- `.intake/tmp/refine-tasks-input.json` had 6 tasks
- `vote-content-r0/r1/r2.json` expanded to 9-10 top-level tasks

Current mitigation:
- downstream artifact and delivery steps now pin to the canonical snapshot chain

Remaining concern:
- upstream task-refinement behavior is still semantically inconsistent with the requested 6-task quick profile

Suggested follow-up:
- inspect `intake/workflows/task-refinement.lobster.yaml`
- decide whether refinement should be allowed to increase top-level task count in quick mode

### B. PR creation still appears non-fatal / unresolved

Latest summary still shows:
- `pr_url = "none"`

Meaning:
- local commit/branch creation worked in `sigma-1`
- PR creation did not produce a usable URL during this run

Suggested follow-up:
- inspect `create-pr` behavior in `intake/workflows/intake.lobster.yaml`
- verify provider/auth/repo conditions for PR creation

### C. Linear hierarchy still deserves a final visual check

The run summary reports:
- `issue_count = 7`

That sounds plausible for:
- 1 parent issue
- 6 task child issues

But I did not complete a final visual validation of:
- parent/task/subtask hierarchy in Linear
- issue body links / branch-valid links

## Files changed in this repo for this handoff

- `intake/workflows/intake.lobster.yaml`
- `intake/workflows/pipeline.lobster.yaml`

## Recommended next steps for Claude

1. Review the two workflow diffs above first.
2. Re-run `bash intake/scripts/run-quick-intake.sh` once to confirm green on Claude’s side.
3. Verify whether `pr_url = none` is acceptable for the intended flow or needs a fix.
4. Do one explicit Linear verification pass for parent/task hierarchy and links.
5. If time allows, harden `task-refinement.lobster.yaml` so quick mode preserves the requested top-level task count instead of relying on downstream canonicalization.
