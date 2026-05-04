# Hermes GitHub Source Supply Notes

## Summary

GitHub access is available through `gh` for the `5dlabs` organization and public `openclaw` repositories. The important clarification is that **Hermes is not a separate upstream GitHub framework repo**. In the current CTO architecture, Hermes is a **5dlabs-internal CodeRun harness mode** implemented in `5dlabs/cto` plus upstream building blocks from `openclaw/acpx` and `openclaw/lobster`.

This removes the earlier ambiguity: there is no missing `5dlabs/hermes` repo in the visible GitHub supply. The source supply to inspect/modify is:

| Source | Role | Status |
|---|---|---|
| `5dlabs/cto` | Owns `HarnessAgent::Hermes`, Hermes launcher template, CodeRun pod resources, centralized Discord bridge, Hermes presence adapter | cloned at `/opt/data/workspace/cto` |
| `5dlabs/morgan-meet` | Morgan-specific design/spec for Hermes sidecar/MCP/event-stream integration | cloned at `/opt/data/workspace/morgan-meet` |
| `openclaw/acpx` | ACP CLI powering Hermes agent invocations | cloned at `/opt/data/workspace/acpx` |
| `openclaw/lobster` | Workflow DSL/shell powering Hermes workflows | cloned at `/opt/data/workspace/lobster` |

## Evidence

`docs/2026-04/agent-presence-hub.md` states:

> `Hermes` as defined in `crates/controller/src/crds/coderun.rs` is a **5dlabs-internal CRD variant**. There is no external "Hermes" framework to contribute to.

It identifies relevant upstream targets:

- `openclaw/acpx`
- ACP spec (`agentclientprotocol/agent-client-protocol`)
- `@clawdbot/lobster` / `openclaw/lobster`
- `livekit/agents-js`

## Current implementation loci

In `5dlabs/cto`:

- `crates/controller/src/crds/coderun.rs`
  - defines `HarnessAgent::Hermes`.
- `templates/harness-agents/hermes.sh.hbs`
  - Hermes runtime entrypoint / file-sentinel pattern.
- `crates/controller/src/tasks/code/templates.rs`
  - renders Hermes launcher.
- `crates/controller/src/tasks/code/resources.rs`
  - builds pod resources, skips OpenClaw gateway/Discord secrets for Hermes, injects Hermes presence adapter sidecar when enabled.
- `apps/discord-bridge/src/*`
  - centralized Discord credential boundary, normalization, route registry, fanout, outbound intents.
- `apps/hermes-presence-adapter/src/*`
  - runtime worker adapter for Hermes; receives normalized events and queues to Hermes API/input or JSONL inbox.

In `openclaw/acpx`:

- `src/cli/config.ts`
  - loads `mcpServers` config from global/project `.acpx` config.
- `src/mcp-servers.ts`
  - validates MCP server definitions.
- `src/runtime/engine/*`
  - passes configured MCP servers into ACP sessions.

This means Hermes already has an MCP injection path through config. The previously proposed upstream `acpx --presence-mcp-url` is a convenience/portability improvement, not a blocker for CTO implementation.

In `openclaw/lobster`:

- workflow support exists as a separate upstream project; CTO Hermes launcher currently uses Lobster workflow orchestration through templates.

## Implication for blockers

The blocker is not “missing GitHub access.” The blocker is narrower:

1. inspect/extend the CTO-owned Hermes harness mode and templates;
2. inspect/extend acpx MCP config/injection behavior only if CTO needs a cleaner upstream flag/env var;
3. implement Morgan/Hermes sidecar/MCP/workspace-stream behavior in `morgan-meet` and wire it through `cto`.

## Recommended unblocking path

1. Treat `5dlabs/cto` as the authoritative Hermes implementation source.
2. Rename/reword the Hermes parity plan to centralized Discord control plane terminology.
3. Start implementation with CTO-owned changes first:
   - Hermes session/home route semantics in `apps/discord-bridge` and `apps/hermes-presence-adapter`.
   - real Hermes CodeRun E2E validation script/docs.
   - Morgan sidecar env/MCP injection in `templates/harness-agents/hermes.sh.hbs` and `crates/controller/src/tasks/code/resources.rs` once the sidecar exists.
4. Defer upstream `openclaw/acpx --presence-mcp-url` PR unless/ until config-only MCP injection is insufficient.
