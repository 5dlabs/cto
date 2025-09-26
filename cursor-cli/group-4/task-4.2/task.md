# Task 4.2 – Operations & CI Notes

## Dependencies
- Task 4.1 (core docs) and completion of Groups 1–3 so procedures reflect reality.

## Parallelization Guidance
- Can run just before Group 5 validation to ensure documentation matches final behaviour.

## Task Prompt
Update top-level readmes/runbooks/CI docs to acknowledge Cursor CLI support and provide concrete commands for operators.

Key updates:
1. `README.md` (repository root)
   - Add Cursor CLI to the multi-CLI support section (commands to run `cursor-agent` locally, highlight `--print --force` usage).
2. `infra/README.md`
   - Document Helm values/flags required to enable Cursor (image, CLI type, secret setup).
   - Mention new ExternalSecret entries and how to sync them.
3. CI tooling docs (`docs/ci/*.md` or pipeline readmes) if new workflows (code review, automation) rely on Cursor CLI steps.
4. Reference auto-PR fallback improvements (branch verification, label skipping) so reviewers know what to expect in logs.

## Acceptance Criteria
- Documentation builds (if any) succeed; `README` changes linted if pipeline exists.
- Ops team acknowledges updates (e.g., Slack thread or PR approval noted in task log).
- No stale references to Codex-only features in updated sections (ensure wording now covers both Codex and Cursor).

## Implementation Notes / References
- Pull examples from `docs/cursor-cli/headless.md` (non-interactive commands) and `code-review.md` (CI integration) to give operators copy-paste snippets.
- Tie in risk mitigations (approval policy, auto-PR fallback) to reassure ops about behaviour differences vs Codex.
