# Task 0.1 – Cursor CLI Requirements Baseline

## Source Review Summary

### Command Invocation Patterns
- `cursor-agent` is the entry point for both interactive and non-interactive runs (`docs/cursor-cli/overview.md`, *Interactive mode* / *Non-interactive mode*).
- Headless automation relies on `--print` (or `-p`) to force print mode and `--force` to allow direct file mutations, mirroring Codex’ `--dangerously-bypass-approvals-and-sandbox` for unattended edits (`docs/cursor-cli/headless.md`).
- Output formats (`text`, `json`, `stream-json`) are selected with `--output-format` when print mode is engaged, with `stream-json` as default (`docs/cursor-cli/output-format.md`).
- Global parameter surface includes `--api-key`, `--resume`, `--model`, and background/fullscreen options (`docs/cursor-cli/parameters.md`).

### Authentication
- Browser flow: `cursor-agent login` opens a web flow and stores tokens locally (`docs/cursor-cli/authentication.md`).
- API key flow: export `CURSOR_API_KEY` or pass `--api-key` per invocation for CI/headless use; aligns with our ExternalSecret pattern used for `OPENAI_API_KEY` (`docs/cursor-cli/authentication.md`).
- `cursor-agent status/logout` diagnostics map cleanly to the auth probes we added for Codex.

### Permissions & Configuration
- Permissions managed via JSON tokens (`Shell()`, `Read()`, `Write()`) inside `~/.cursor/cli-config.json` or project-level `.cursor/cli.json` (`docs/cursor-cli/permissions.md`, `docs/cursor-cli/configuration.md`).
- Only permissions are project-scoped; other config remains global, so our templates must mount project overrides under `.cursor/` but keep global settings in the container home directory.
- `--force` is required for write access even when permissions allow it (`docs/cursor-cli/headless.md`).

### MCP Expectations
- CLI reuses the IDE’s `.cursor/mcp.json` and surfaces management commands (`cursor-agent mcp list`, `list-tools`, `login`) (`docs/cursor-cli/mcp.md`).
- Precedence order (project → global) matches our existing ConfigMap layering for Codex memory files, so we can reuse that pattern.

### Approval Semantics
- Cursor prompts for command approval by default (`docs/cursor-cli/using-agent.md`, *Command approval*). We must enforce `approvalPolicy = "never"` equivalent by combining `--print`, `--force`, and template-level instructions that forbid pausing, similar to the Codex prompt updates we just implemented.

### Known Limitations / Operational Notes
- Shell commands timeout after ~30s and large outputs are truncated (`docs/cursor-cli/shell-mode.md`). Container scripts must segment long tasks and check exit messaging, as we did with Codex retries.
- Non-interactive mode still requires explicit `--force` for file edits and uses full write access by default (`docs/cursor-cli/using-agent.md`).
- `stream-json` emits NDJSON with tool call detail; useful for telemetry/logging similar to Codex’ `tee` approach (`docs/cursor-cli/output-format.md`).

### CI / Tooling Reference
- GitHub Actions example demonstrates `--force`, `--print`, `--output-format text`, and GH CLI integration for code review workflows (`docs/cursor-cli/code-review.md`).

### Lessons from Codex to Carry Forward
- Add “execute without pause” directives in both memory templates and banner to avoid agents waiting for approval.
- Implement automation-managed file cleanup (Codex fix removing stale `AGENTS.md`, `task/` assets) to keep retries PVC-safe.
- Maintain retries with completion probes and diagnostic logging (`--output-last-message` equivalent) to detect early exits.
- Preserve robust label creation / PR fallback logic for parity with Codex auto-PR safeguards.
- Ensure reasoning-effort/approval policy plumbed via controller + Helm, mirroring Codex’ TOML templating.

## Delta Table – Cursor vs Codex

| Area | Codex (current) | Cursor (target) | Action |
| --- | --- | --- | --- |
| CLI binary & command | `codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check` | `cursor-agent --print --force --output-format stream-json` (plus model/api settings) | Update container templates & adapters to invoke `cursor-agent` with required flags and environment.
| Auth env var | `OPENAI_API_KEY` | `CURSOR_API_KEY` | Add ExternalSecret + pod env wiring; update config TOML/JSON templates.
| Approval model | Configured via `model_reasoning_effort` + `approval_policy="never"` in config TOML | Default prompts for approval; must rely on `--force`, print mode, and explicit prompts to prevent pauses | Replicas of Codex prompts plus CLI flags; document fallback if Cursor adds direct approval flag in future.
| Permissions storage | Config TOML + AGENTS.md via ConfigMap | `.cursor/cli-config.json`, `.cursor/cli.json`, existing AGENTS/CLAUDE rules (`docs/cursor-cli/permissions.md`) | Expand ConfigMap generator to render `.cursor` assets with allow/deny tokens.
| MCP config | `codex` config TOML references Toolman | `.cursor/mcp.json` autodiscovered; CLI commands to list/login servers | Ensure Task assets mount `.cursor/mcp.json` and pass Toolman endpoint similarly.
| Shell limits | No explicit 30s limit, but sandbox policies apply | Hard 30s shell timeout, truncated output | Adjust container script instrumentation, break long commands, and rely on retries.
| Output logging | `tee` raw stdout, optional JSON logs | Provide `stream-json` NDJSON out-of-box | Capture NDJSON logs under `/tmp` for observability / completion probes.
| PR fallback | Git auto-commit + GH label creation with error handling | Same logic required | Reuse Codex autostage + label creation blocks with binary changes.

## Risk Log
- **Cursor CLI Version Stability:** Auto-update behaviour may pull newer binaries at runtime. Need to pin or cache version in our Docker image to avoid breaking changes mid-run.
- **Force Flag Behaviour:** `--force` bypasses approvals globally; confirm it doesn’t enable destructive shells outside allowed permissions.
- **Shell Timeout Impact:** 30s limit could break lengthy builds/tests. May require splitting commands or handing off to dedicated tools.
- **MCP Compatibility:** Need to verify Toolman’s MCP server works with Cursor’s expectations (transport, auth). Might require adapter tweaks.
- **API Key Provisioning:** Ensure ExternalSecret rotation handles Cursor quotas/limits; confirm key scope includes CLI usage.
- **Output Parsing:** `stream-json` structure differs from Codex event schemas. Downstream log processors/tests must handle Cursor-specific payloads.

## Stakeholder Sign-off
- ✅ Baseline reviewed with project owner (per CLI session, 2025-09-26) confirming readiness to proceed to engineering tasks.
