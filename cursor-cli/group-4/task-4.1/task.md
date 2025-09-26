# Task 4.1 – Project Documentation Set

## Dependencies
- Task 0.1 baseline (so topics are understood). Engineering tasks (Groups 1–3) should be far enough along to reference actual behaviour.

## Parallelization Guidance
- Can start once interfaces stabilise; update alongside engineering tasks to avoid drift.

## Task Prompt
Document the Cursor CLI integration for internal users and operators, mirroring the depth of existing Codex docs.

Deliverables:
- Create a documentation bundle under `docs/projects/cursor-cli/` (or similar) containing:
  - `overview.md` describing architecture (controller -> job -> cursor-agent) and parallels to Codex.
  - `installation.md` summarising Helm values, secret requirements (`CURSOR_API_KEY`), and image tags.
  - `runbook.md` with operational procedures (smoke test command, interpreting `stream-json` output, log locations, handling auto-PR fallback messages).
  - `troubleshooting.md` capturing known issues (approval policy misconfig, missing `--force`, GitHub label failures, MCP connectivity) and remediation steps referencing code paths (e.g., templates or adapter).
- Cross-link official Cursor documentation sections (`docs/cursor-cli/*.md`) where appropriate; make it obvious which behaviour comes from upstream vs our wrapper.
- Include lessons learned from Codex rollout (auto PR fallback adjustments, reasoning effort plumbing) so future CLI additions reuse the pattern.

## Acceptance Criteria
- Docs reviewed by engineering + operations (record sign-off in PR description or checklist).
- Each doc references relevant code/config (paths, commits) so readers can dive deeper.
- No TODOs or placeholders; all sections filled with actionable guidance.
- Markdown passes existing linting (if any) and integrates into `README`/navigation.

## Implementation Notes / References
- Use `Cursor CLI/group-*` task outputs as inputs (e.g., requirements notes, test plans).
- Highlight security posture (approval policy enforced to `never`) and compare to Codex for context.
- Coordinate with Group 4.2 to avoid duplication in README updates.
