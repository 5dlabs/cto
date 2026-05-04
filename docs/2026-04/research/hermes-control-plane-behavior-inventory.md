# Hermes Control-Plane Behavior Inventory

## Scope

This inventory is based on the GitHub/source supply currently available in the workspace:

- `5dlabs/cto` at `/opt/data/workspace/cto`
- `5dlabs/morgan-meet` at `/opt/data/workspace/morgan-meet`
- `openclaw/acpx` at `/opt/data/workspace/acpx`
- `openclaw/lobster` at `/opt/data/workspace/lobster`

The key discovery is that Hermes is not a standalone external GitHub framework. It is a 5dlabs-internal CodeRun harness mode implemented in CTO plus upstream `acpx` and `lobster` building blocks.

## Current Hermes runtime implementation

### CodeRun harness mode

File: `crates/controller/src/crds/coderun.rs`

- Defines `HarnessAgent::Hermes`.
- Hermes means standalone ACPX + Lobster, without the OpenClaw gateway.

### Hermes launcher

File: `templates/harness-agents/hermes.sh.hbs`

Observed behavior:

- Sets up writable `HOME` under `/tmp/agent-home`.
- Symlinks CLI state/log directories from `/workspace` into `HOME`.
- Writes workspace identity files including `AGENTS.md`.
- Runs Lobster workflow directly:
  - `lobster run .tasks/index.lobster.yaml`
  - fallback: `npx -y @clawdbot/lobster run .tasks/index.lobster.yaml`
- Uses `/workspace/.agent_done` as lifecycle sentinel.
- Tails CLI logs to stdout for Datadog/container log collection.
- Does not use OpenClaw gateway.
- Does not mount Discord secrets for Hermes.

### Hermes presence adapter sidecar

File: `crates/controller/src/tasks/code/resources.rs`

Observed behavior:

- Adds `hermes-presence-adapter` sidecar only when:
  - effective harness is Hermes; and
  - presence config is enabled.
- Sidecar image defaults to `ghcr.io/5dlabs/hermes-presence-adapter:latest`.
- Sidecar exposes port `3305` as `presence`.
- Mounts `/workspace`.
- Injects:
  - `PRESENCE_ROUTER_URL`
  - `PRESENCE_ROUTE_ID`
  - `AGENT_ID`
  - `CODERUN_ID`
  - `PROJECT_ID`
  - `TASK_ID`
  - `HERMES_INPUT_URL=http://127.0.0.1:8080/input`
  - `HERMES_INBOX_PATH=/workspace/<subdir>/presence-inbox.jsonl`
  - `POD_IP`
  - `PRESENCE_SHARED_TOKEN` by secret reference only
  - optional Discord account/guild/channel/thread filters from CodeRun env

### Hermes adapter app

Files: `apps/hermes-presence-adapter/src/*`

Observed behavior:

- Exposes `/presence/inbound`.
- Accepts authenticated `cto.presence.v1` events for runtime `hermes`.
- Converts inbound Discord event to Hermes input payload with:
  - text content
  - attachment list text
  - metadata including runtime, agent, project, task, coderun, Discord account/channel/thread/message IDs, and session key
  - session object with platform `discord`, chat ID/type, user ID/name, thread ID
- Posts to `HERMES_INPUT_URL` when configured.
- Falls back to appending JSONL inbox file when Hermes input endpoint is unavailable.
- Posts non-fatal status intents to centralized bridge `/presence/outbound`.
- Registers/deletes its presence route with centralized bridge.

## Current centralized Discord bridge implementation

Files: `apps/discord-bridge/src/*`

Observed behavior from current code/previous validation:

- Owns Discord credentials and Discord API side effects.
- Normalizes Discord messages to `cto.presence.v1` once at the bridge boundary.
- Route runtime type includes `hermes`, `openclaw`, and `hosted`.
- Fanout routing supports Discord route filters and project/task/coderun specificity.
- Shared-channel ambient messages fail closed unless explicitly addressed.
- Mention normalization includes Discord snowflake IDs and stable lowercased names/usernames.
- Thread events preserve parent channel and actual thread ID separately.
- Outbound intents include `send`, `edit`, `react`, `typing`, and `status`.

## /sethome patch behavior

File: `infra/manifests/hermes-control-plane-builder/patch-discord-sethome.py`

Observed behavior:

- Patches the Hermes v2026.4.23 runtime file `/opt/hermes/gateway/platforms/discord.py` in the image/pod.
- Adds explicit Discord command contexts to `/sethome`:
  - guilds enabled
  - DMs enabled
  - private channels enabled
  - guild installs enabled
  - user installs enabled
- Extends safe slash-command sync payload diffing so contexts/integration types are compared and stale global commands are recreated.

This fixes Discord rejecting `/sethome` before Hermes handles it with “Unknown integration.”

## MCP / acpx behavior

Files:

- `openclaw/acpx/src/cli/config.ts`
- `openclaw/acpx/src/mcp-servers.ts`
- `openclaw/acpx/src/runtime/engine/*`

Observed behavior:

- acpx already supports configured `mcpServers` in global/project config.
- `src/mcp-servers.ts` validates HTTP/SSE/stdio MCP server definitions.
- Runtime/session code passes configured MCP servers into ACP sessions.

Implication:

- CTO can inject Morgan/presence MCP servers through generated acpx config or environment/config files today.
- An upstream `acpx --presence-mcp-url` flag remains useful but is not required to unblock CTO implementation.

## Morgan/Hermes target behavior from docs

Files:

- `docs/morgan-meet-hermes-design.md`
- `docs/2026-04/morgan-meet-hermes-design.md`
- `morgan-meet/.plan/spec/hermes-adapter.md`

Target behavior:

- Morgan Meet bot runs as a Hermes CodeRun sidecar.
- Sidecar exposes MCP tools on localhost, e.g.:
  - `meet_join`
  - `meet_leave`
  - `meet_get_status`
  - `meet_stream_audio`
- Harness injects `MORGAN_MEET_MCP_URL` and `MORGAN_MEET_SESSION_ID`.
- ACPX discovers the MCP tools before/at startup.
- Workspace streams coordinate events/commands/status:
  - `/workspace/meet-events.jsonl`
  - `/workspace/meet-commands.jsonl`
  - `/workspace/meet-status.json`
- A `meet-init` Lobster step waits for sidecar readiness and writes initial status.

## Known gaps to Hermes parity

1. **Real Hermes CodeRun E2E validation**
   - Need live CodeRun receiving bridge-normalized event and producing outbound status/send effects.

2. **Session/home/crown semantics**
   - `/sethome` install-context behavior is fixed.
   - Control-plane home/session mapping still needs explicit implementation/testing.
   - No separate source found yet for a “crown” system beyond current CTO/Hermes harness docs; likely needs live runtime or deeper image inspection if it is inside `/opt/hermes/gateway`.

3. **MCP injection for Morgan**
   - acpx supports MCP config.
   - CTO Hermes launcher/controller does not yet generate Morgan MCP config/env/meet-init wiring.

4. **Morgan sidecar implementation**
   - Design exists.
   - Actual sidecar/MCP implementation is not present in current `morgan-meet` repo snapshot.

5. **OpenCloud/OpenClaw adapter**
   - Runtime type and central bridge contract exist.
   - Full adapter remains to implement/validate.

## Recommended next implementation target

Start with CTO-owned, high-confidence work:

1. Add a real Hermes CodeRun smoke/validation script and documentation.
2. Implement centralized control-plane session/home metadata rules in bridge + Hermes adapter tests.
3. Add controller/template support for optional Morgan MCP sidecar config once sidecar image/API is available.
4. Implement OpenCloud/OpenClaw adapter after Hermes path is end-to-end green.
