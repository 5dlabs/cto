# Codex Play Session Issues

## Issue 1 – Reasoning Effort Not Propagated
- **Observed:** Session banner reports `reasoning effort: none` even though Helm/config defaults set the value to `high` for Codex agents.
- **Context:** First Codex run against `5dlabs/rust-basic-api` on 2025-09-26T15:41:45Z.
- **Notes:** Appears identical to prior gap; need to verify template/config rendering and adapter wiring before next run.

## Issue 2 – Agent Stopped for Approval Prompt / No Code Changes
- **Observed:** Rex ended with “Let me know if you want adjustments before I dive in,” implying it awaited user confirmation instead of executing changes. No implementation work was performed before attempting PR creation.
- **Hypothesis:** Approval handling still allows conversational pause despite templates enforcing `approvalPolicy = "never"`; need to verify prompt/instructions discourage stopping for feedback.

## Issue 3 – Auto-PR Shell Errors & Missing Labels
- **Observed:** Auto-PR fallback created branch `task-1-rust-basic-api-20250926-154223` but `container.sh` emitted:
  - `/task-files/container.sh: line 256: task-1-rust-basic-api-20250926-154223: command not found`
  - `/task-files/container.sh: line 256: main: command not found`
- **Impact:** Labels `task-1`, `service-rust-basic-api`, `run-play-workflow-template-8nlbk` were skipped (branch/labels only relevant when PR exists).
- **Notes:** Likely due to unquoted variables in branch checkout/push command; needs inspection of latest template changes.

## Issue 4 – QA/Test Agents Triggered Without PR
- **Observed:** Cleo (and potentially Tess) kicked off despite the PR being auto-created after the container errors; they should wait for an existing PR (workflow relies on GitHub webhook sensor).
- **Hypothesis:** Play workflow now proceeds regardless of PR creation status (recent refactor removed gating) or the webhook sensor mis-detected the auto-generated PR.
- **Action:** Audit workflow logic for PR existence check post-refactor.

> _Note:_ Workflow gating for PR readiness previously worked before the recent refactor; investigate diffs against pre-refactor sensor/controller logic to isolate regression quickly.

## Issue 5 – Implementation Agent Committed Directly to Main
- **Observed:** Rex committed and pushed changes to `main` instead of creating a feature branch before PR creation. Auto-PR fallback then created a branch and pushed the same changes, but the initial push to `main` should never happen.
- **Note:** Claude implementation explicitly instructed agents to work on feature branches only; Cursor/Codex prompts need the same guardrail (task-name+timestamp branch naming, no direct pushes to default branch).
