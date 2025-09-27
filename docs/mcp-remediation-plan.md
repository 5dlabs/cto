# MCP Remediation Plan

## Background
Codex runs on the CLI-agnostic controller started returning **zero tools** after the latest refactor. The controller now renders `client-config.json` per CLI, and each container copies that file into `/workspace`. Toolman logs show the config is read correctly, but when Codex calls `tools/list`, Toolman responds with the full 56-tool catalog or an empty list; Codex never sees the filtered set and frequently stalls waiting for destructive cleanup approval (`rm -rf …`). The same workflow worked reliably with Claude before the CLI migration, so we suspect the new runtime wiring introduced regressions.

## Symptoms Observed
- `tools/list` responses from Toolman are empty in Codex (`{"tools": []}`), even though the runtime logs confirm 10 remote tools.
- Direct `curl` to the Toolman MCP endpoint returns all 56 tools, indicating server-side filtering is disabled.
- Codex repeatedly attempts `rm -rf` on the repo and `~/.rustup/downloads`, triggering approval prompts and hanging the workflow.
- `MCP_CLIENT_CONFIG` is exported, but earlier scripts overwritten it or copied it inconsistently across CLIs.

## Current State
- Codex container script now copies the config to `/workspace` and exports `MCP_CLIENT_CONFIG`. Guards prevent recursive deletes of `/workspace`, repo root, and `~/.rustup`; the Rustup cache is cleared at startup to discourage cleanup commands.
- Cursor and Factory still use older patterns (no guard, direct copy). `values.yaml` and `cto-config.json` still reference outdated tool names (underscores instead of hyphens), contributing to the mismatch between Toolman responses and expected names.
- Toolman client (Rust STDIO bridge) applies client-side filtering. Logs prove it loads the curated 10-tool list but the HTTP server returns every tool because the base URL passed to `toolman` resolves to “relative URL without a base” in the tmp pod, meaning the CLI isn’t sending the correct base when invoked from Codex.

### 2025-09-27 Findings
- The Codex pod’s rendered `/workspace/client-config.json` still contains underscore tool identifiers (e.g., `brave_web_search`, `context7_get_library_docs`). Toolman advertises hyphenated names (`brave_search_brave_web_search`, `context7_get-library-docs`), so the Rust MCP bridge filters everything out and Codex sees `tools: []`. Source of truth is the Helm values under `infra/charts/controller/values.yaml`; those entries need to be normalized to Toolman’s canonical names so the controller emits the correct list.
- Executing `toolman --url http://toolman.agent-platform.svc.cluster.local:3000/mcp tools list` inside the Codex pod fails with `builder error -> relative URL without a base`. The bridge logs (“Making HTTP request to: tools”) show that the CLI joins relative paths onto a base lacking a trailing slash. All defaults in controller code and templates (`TOOLMAN_SERVER_URL`) end with `/mcp` and must be migrated to `/mcp/` to keep the final segment when joining.
- The guard injected into the Codex container (`guard_rm`) currently emits `grep: unrecognized option '--|^-r.*-f|-f.*-r'` because the pattern starts with `--`. That forces the check to short-circuit and Codex can still run `rm -rf` against protected directories. Adjust the `grep` invocation to pass the pattern after `--` or replace the detection with a pure shell expression so the guard actually blocks destructive deletes.

### Recommended Next Steps (in addition to prior plan)
1. Update Helm values (and any mirrored sources such as `cto-config.json`) to use the exact Toolman names, then regenerate the controller ConfigMap so Codex receives hyphenated identifiers.
2. Add a normalization helper in the controller (or templates) that appends a trailing slash to `TOOLMAN_SERVER_URL` before writing env vars and CLI configs; confirm the regenerated pods stop logging the `relative URL without a base` error.
3. Patch `guard_rm` in `container-base.sh.hbs` to use a safe pattern match (e.g., `grep -Eq -- '(^| )-(r.*-f|-f.*-r)( |$)'`) or a string case statement so destructive commands finally short-circuit.
4. Re-run the Codex workload or the `codex-mcp-test` pod after regenerating configs to validate that `tools/list` returns the expected 10 tools and the guard blocks `rm -rf /workspace`.

## Proposed Fixes
1. **Codex Runtime**
   - Keep the guard + Rustup cache clean; confirm the updated ConfigMap is redeployed so new pods use it.
   - Add the same guard/copy logic to Cursor and Factory for consistency.

2. **Tool Whitelists**
   - Update `values.yaml`, `client-config.json`, and `cto-config.json` to use the correct tool identifiers (e.g. `brave_search_brave_web_search`, `context7_get-library-docs`). Remove unused documentation tools if they should not be exposed.

3. **Toolman Client**
   - In the controller, ensure the `TOOLMAN_SERVER_URL` passed to `toolman` includes the full base (e.g. `http://…/mcp`) and verify `--url` is honoured in the deployed CLI. If not, update the CLI invocation (or Toolman itself) so the base URL is respected.

4. **Redeploy and Validate**
   - Helm upgrade controller to refresh ConfigMaps/scripts.
   - Run Codex job; confirm `tools/list` returns the curated list and the workflow finishes without approval loops.

## Risks / Unknowns
- Toolman server may still broadcast all tools regardless of the client list; need to confirm filtering actually happens once the base URL bug is resolved.
- Changes to `values.yaml` must be mirrored in controller templates to ensure generated configs stay consistent.

## Next Steps
1. Mirror the Codex guard/config copy into Cursor and Factory scripts.
2. Normalize tool names across `values.yaml`, `cto-config.json`, `client-config.json` and regenerate runtime configs.
3. Ensure `toolman` invocation in the controller no longer produces “relative URL without a base” errors; adjust CLI arguments or Toolman binaries if needed.
4. Redeploy controller and re-run Codex; validate tool visibility and guard behaviour in the live pod.
