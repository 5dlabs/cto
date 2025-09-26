# Cursor CLI Integration – Lessons Learned from Codex Rollout

## Prompting & Agent Behaviour
- **Explicit autonomy is mandatory**: Codex repeatedly paused after planning until we reinforced “execute without pause” in both the shared memory template and the prompt banner. Cursor templates must include the same directive so agents do not wait for human confirmation mid-run.
- **Restate feature-branch rules in every channel**: Even after Codex was bootstrapped onto the correct branch, we needed redundant reminders in the prompt banner, shared memory, and Rex-specific instructions to stop accidental pushes to `main`. Cursor should mirror that triple coverage.

## Runtime Resilience
- **Clean automation-managed artefacts before checkout**: When we re-ran Codex against an existing PVC, cached `AGENTS.md`, `task/*`, and config files blocked `git checkout`. We now remove those paths when they are untracked. Cursor containers should perform the same cleanup so retries do not require manual PVC resets.
- **Capture completion signals**: Adding `--output-last-message` logging and a follow-up "is the task complete?" probe gave us a deterministic way to detect premature exits. Cursor’s wrapper should keep that pattern (or an equivalent sentinel) so we can retry safely and surface actionable logs.
- **Controlled retries with guardrails**: Automating up to three Codex retries (and skipping PR enforcement when completion fails) prevented half-finished PRs. Cursor’s execution loop should include the same retry + short-circuit logic, ideally with a configurable max attempt count.

## Tooling & Diagnostics
- **Persist diagnostic logs under `/tmp`**: Recording run and completion JSON/last-message files provided context for every attempt. Keep that habit for Cursor so on-cluster debugging is straightforward.
- **Graceful label creation**: GitHub label creation must tolerate existing labels—Codex now checks before creating and logs the response. Cursor scripts should reuse that logic to avoid noisy warnings.

## Template Consistency
- **AGENTS.md injection**: Pulling the rendered AGENTS.md into the prompt ensured Codex respected project memory. Cursor templates should render and inline the appropriate memory file just like Codex/Claude.
- **Retry-safe asset sync**: Task docs and guidelines are copied on every attempt. Ensure Cursor follows the same copy semantics so reruns stay idempotent.

Applying these patterns during Cursor CLI implementation will save rework and keep the workflow resilient under retries, long-running tasks, or partial failures.
