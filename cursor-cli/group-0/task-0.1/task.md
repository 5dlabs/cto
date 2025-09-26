# Task 0.1 – Requirements Baseline for Cursor CLI

## Dependencies
- None (kick-off task for the entire initiative).

## Parallelization Guidance
- Must complete before any engineering work (Groups 1–5) to ensure shared understanding.

## Task Prompt
Thoroughly analyse the Cursor CLI documentation set under `docs/cursor-cli/` (overview, installation, authentication, parameters, permissions, configuration, headless usage, shell mode, MCP, using-agent, output-format). Produce a requirements dossier that contrasts Cursor behaviours with the Codex implementation we just finished. Focus on:
- Command invocation patterns (`cursor-agent` vs `codex exec`), including the `--print`, `--force`, and `--output-format` flags required for headless automation.
- Authentication flows (browser vs `CURSOR_API_KEY`) and how they map onto our ExternalSecret + ENV wiring (`CURSOR_API_KEY` parallel to `OPENAI_API_KEY`).
- Permission model (`Shell()`, `Read()`, `Write()` tokens) and how it aligns with our existing `cli-config.json` templating.
- MCP integration expectations (reuse of `.cursor/mcp.json`, command surface for listing servers/tools).
- Default command approval semantics (Cursor CLI prompts by default; we need to enforce `approvalPolicy = "never"`).
- Known limitations (30s shell timeout, truncation behaviour) that may affect container scripts.
- Tool invocation patterns for CI (see code-review example in `code-review.md`).

Capture lessons-learned from the Codex rollout (auto PR fallback behaviour, run label handling, reasoning effort plumbing) so we can anticipate analogous requirements for Cursor.

## Acceptance Criteria
- Written summary (commit to repo in `Cursor CLI/group-0/task-0.1/notes.md`) covering every bullet above with explicit references to source docs (path + section).
- Delta table highlighting where Cursor deviates from Codex (e.g., binary name, auth env var, permission schema) and how we will address each delta.
- Risk log enumerating open questions or prerequisites (e.g., Cursor credit limits, need for `--force` default, MCP compatibility).
- Stakeholder sign-off recorded (can be a comment referencing Slack/Jira/PR) acknowledging the baseline is complete.

## Implementation Notes / References
- Docs live in `docs/cursor-cli/*.md`; cite specific headings when summarising.
- Reuse Codex experience: approval policy enforcement lives in `controller/src/tasks/code/templates.rs` and `infra/charts/controller/agent-templates/code/codex/config.toml.hbs`; document analogues we’ll need.
- Note existing Helm/ConfigMap wiring for Codex (`infra/charts/controller/templates/task-controller-config.yaml`, `values.yaml`) as templates for future tasks.
