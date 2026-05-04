# Morgan Hermes Agent Sidecar Design

## Purpose

This document specifies the Wave 2B implementation target for running Morgan as a Hermes-native agent surface: a Kubernetes sidecar that exposes MCP tools to ACPX/Lobster, coordinates through `/workspace` streams, and fits the existing CTO controller/Hermes harness without adding an OpenClaw gateway dependency.

It builds on:

- `docs/2026-04/morgan-meet-hermes-design.md`
- `docs/2026-04/avatar-agent-platform-inventory.md`
- `docs/2026-04/research/hermes-control-plane-behavior-inventory.md`
- `templates/harness-agents/hermes.sh.hbs`
- `crates/controller/src/tasks/code/resources.rs`
- `/opt/data/workspace/morgan-meet/.plan/spec/hermes-adapter.md`

## Current ground truth

Hermes in CTO is a `CodeRun` harness mode, not a standalone gateway runtime.

Current behavior:

- `HarnessAgent::Hermes` launches Lobster + ACPX directly.
- `templates/harness-agents/hermes.sh.hbs` writes workspace identity files, runs `.tasks/index.lobster.yaml`, and uses `/workspace/.agent_done` as the lifecycle sentinel.
- `crates/controller/src/tasks/code/resources.rs` already knows how to add a Hermes-only `hermes-presence-adapter` sidecar when presence is enabled.
- That presence sidecar exposes port `3305`, mounts `/workspace`, registers a route with the centralized Discord bridge, and writes inbound events to `/workspace/<run-subdir>/presence-inbox.jsonl` if the Hermes input endpoint is unavailable.
- ACPX supports configured MCP servers through generated config (`mcpServers`), so Morgan MCP discovery can be done through generated MCP config files and environment variables. A new ACPX flag is useful later but not required to unblock implementation.

## Target topology

```text
CodeRun pod, harness = hermes

  main container
    ├─ /workspace                      shared PVC/volume
    ├─ hermes.sh.hbs                   bootstraps identity + CLI config
    ├─ lobster run .tasks/index...     workflow runner
    └─ ACPX / selected CLI             calls MCP tools

  hermes-presence-adapter sidecar       existing Discord bridge adapter
    ├─ :3305 /presence/inbound
    ├─ registers route with bridge
    └─ writes presence-inbox.jsonl fallback

  morgan-agent sidecar                  new Wave 2B target
    ├─ :4000 MCP server
    ├─ :4001 health/debug HTTP server, optional same process
    ├─ Morgan session/presence bridge
    ├─ LiveKit/avatar/provider adapters
    └─ /workspace JSONL streams
```

The Morgan sidecar is pod-local. It should not expose external ingress by default. External meeting/Discord/web surfaces go through existing bridges, provider APIs, or signed LiveKit URLs.

## Sidecar identity and lifecycle

### Naming

Default sidecar name: `morgan-agent`.

Use `morgan-meet-bot` only for a narrower meeting-specific image. The controller should support a generic Morgan sidecar with feature flags so later non-meeting Morgan surfaces do not require another sidecar type.

### Container contract

Required container behavior:

- Bind MCP HTTP/SSE server on `127.0.0.1:${MORGAN_MCP_PORT:-4000}` or `0.0.0.0` inside the pod only.
- Expose a health endpoint at `GET /healthz` returning JSON.
- Mount `/workspace` read/write.
- Never write secrets to workspace streams.
- Append JSONL atomically. Use one JSON object per line, no multi-line records.
- Stop active meeting/avatar sessions on SIGTERM before process exit.

Recommended image:

```text
ghcr.io/5dlabs/morgan-agent-sidecar:<version>
```

Do not pin production CodeRuns to `latest`; use an explicit semver or git SHA tag once validation starts.

### Environment variables

Required:

| Env var | Example | Purpose |
|---|---|---|
| `WORKSPACE` | `/workspace` | Shared stream root. |
| `MORGAN_MCP_PORT` | `4000` | MCP listener port. |
| `MORGAN_HEALTH_PORT` | `4001` | Optional debug/health HTTP port if separate. |
| `MORGAN_AGENT_ID` | `morgan` | Stable agent ID in events. |
| `MORGAN_SESSION_ID` | `morgan_<coderun>_<epoch>` | Default session ID for this run. |
| `CODERUN_ID` | `morgan-abc123` | Controller run identity. |
| `PROJECT_ID` | `sigma-one` | Memory/session namespace. |
| `TASK_ID` | `1234` | Optional task identity. |
| `MORGAN_STREAM_DIR` | `/workspace/runs/<uid>` | Per-run stream directory. |
| `MORGAN_EVENT_LOG` | `/workspace/runs/<uid>/morgan-events.jsonl` | Sidecar event output. |
| `MORGAN_COMMAND_LOG` | `/workspace/runs/<uid>/morgan-commands.jsonl` | Agent command input. |
| `MORGAN_STATUS_FILE` | `/workspace/runs/<uid>/morgan-status.json` | Current status snapshot. |

Provider-specific secrets should be injected with `secretKeyRef`, never by writing to ConfigMaps:

| Env var | Secret source |
|---|---|
| `LIVEKIT_URL` | `cto-secrets/livekit-url` or dedicated Morgan secret. |
| `LIVEKIT_API_KEY` | `livekit-api-key`. |
| `LIVEKIT_API_SECRET` | `livekit-api-secret`. |
| `LEMONSLICE_API_KEY` | `lemonslice-api-key`. |
| `RECALL_API_KEY` | `recall-api-key`, if managed bot support is enabled. |
| `MORGAN_PROVIDER_ROUTER_TOKEN` | Optional internal router token. |

### Health endpoint

`GET /healthz` response:

```json
{
  "ok": true,
  "service": "morgan-agent-sidecar",
  "version": "0.1.0",
  "mcp": { "port": 4000, "ready": true },
  "workspace": {
    "root": "/workspace",
    "event_log_writable": true,
    "command_log_readable": true,
    "status_file_writable": true
  },
  "providers": {
    "livekit": "configured",
    "lemonslice": "configured",
    "recall": "missing_optional"
  }
}
```

Readiness means the MCP tool registry is ready and required workspace paths are writable. Provider reachability can be degraded without failing readiness unless the enabled mode requires that provider.

## MCP server contract

MCP server name: `morgan`.

Transport: HTTP/SSE MCP endpoint in pod, configured as an ACPX MCP server:

```json
{
  "mcpServers": {
    "morgan": {
      "type": "http",
      "url": "http://127.0.0.1:4000/mcp",
      "headers": {
        "X-Coderun-Id": "${CODERUN_ID}",
        "X-Morgan-Session-Id": "${MORGAN_SESSION_ID}"
      }
    }
  }
}
```

If the selected CLI only supports stdio MCP at first, wrap the local HTTP server with the existing tools bridge rather than changing Morgan's sidecar API.

### Tool naming

Use `morgan_*` for generic agent/presence tools and `meet_*` for meeting-specific compatibility tools.

The sidecar should expose both names initially where the behavior overlaps:

- `meet_join` delegates to `morgan_session_start` with `surface.kind = "meeting"`.
- `meet_leave` delegates to `morgan_session_stop`.
- `meet_get_status` delegates to `morgan_session_status`.

This preserves the existing Morgan Meet design while allowing a broader Morgan sidecar.

### Required MCP tools, v1

#### `morgan_session_start`

Start or attach Morgan to a live presence surface.

Input schema:

```json
{
  "type": "object",
  "required": ["surface"],
  "properties": {
    "session_id": { "type": "string" },
    "surface": {
      "type": "object",
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["livekit_room", "meeting_url", "discord_thread", "webpage", "audio_only"] },
        "url": { "type": "string" },
        "room_name": { "type": "string" },
        "provider": { "type": "string" }
      }
    },
    "mode": { "enum": ["auto", "live-realtime", "async-turn-video", "audio-only", "symbolic"] },
    "consent": {
      "type": "object",
      "properties": {
        "required": { "type": "boolean" },
        "message": { "type": "string" },
        "accepted_by": { "type": "string" }
      }
    },
    "metadata": { "type": "object" }
  }
}
```

Output:

```json
{
  "session_id": "morgan_coderun-abc_1770000000",
  "status": "starting",
  "surface": { "kind": "livekit_room", "url": "https://..." },
  "active_provider": "lemonslice",
  "workspace": {
    "events": "/workspace/runs/abc/morgan-events.jsonl",
    "commands": "/workspace/runs/abc/morgan-commands.jsonl",
    "status": "/workspace/runs/abc/morgan-status.json"
  }
}
```

#### `morgan_session_stop`

Input:

```json
{ "session_id": "morgan_coderun-abc_1770000000", "reason": "task_complete" }
```

Output:

```json
{ "session_id": "morgan_coderun-abc_1770000000", "status": "stopped" }
```

#### `morgan_session_status`

Input:

```json
{ "session_id": "morgan_coderun-abc_1770000000" }
```

Output:

```json
{
  "session_id": "morgan_coderun-abc_1770000000",
  "status": "speaking",
  "active_provider": "lemonslice",
  "fallback_used": false,
  "surface": { "kind": "livekit_room", "url": "https://..." },
  "last_event_seq": 42,
  "updated_at": "2026-05-03T00:00:00Z"
}
```

#### `morgan_say`

Ask Morgan to speak or publish text/audio into the current surface.

Input:

```json
{
  "session_id": "morgan_coderun-abc_1770000000",
  "text": "I can take that follow-up.",
  "interrupt": true,
  "voice": { "provider": "auto", "style": "concise" },
  "visibility": "public"
}
```

Output:

```json
{
  "turn_id": "turn_000042",
  "status": "queued",
  "estimated_start_ms": 250
}
```

#### `morgan_set_state`

Publish non-speech state for avatar/UI fallback.

Input:

```json
{
  "session_id": "morgan_coderun-abc_1770000000",
  "state": "thinking",
  "message": "Checking the repository before answering.",
  "ttl_ms": 30000
}
```

Allowed `state` values:

```text
idle, listening, thinking, speaking, interrupted, tool_running,
avatar_unavailable, degraded, error, recovered, stopped
```

#### `morgan_events_tail`

Return recent events from the workspace JSONL log for CLIs that cannot tail files directly.

Input:

```json
{ "session_id": "morgan_coderun-abc_1770000000", "after_seq": 41, "limit": 50 }
```

Output:

```json
{
  "events": [
    { "seq": 42, "type": "state", "session_id": "...", "state": "speaking" }
  ],
  "next_after_seq": 42
}
```

### Meeting compatibility tools

Expose these as thin aliases for Morgan Meet compatibility:

```text
meet_join(room_url, session_id?, provider?) -> session_id/status/surface
meet_leave(session_id) -> stopped status
meet_get_status(session_id) -> current session status
meet_stream_audio(session_id, audio_url) -> queued turn/provider status
```

`meet_stream_audio` is only required if Morgan Meet still needs external audio artifact ingestion. For the normal Hermes path, `morgan_say` plus provider TTS should be preferred.

## Workspace stream contract

Use per-run stream files under the existing run subdirectory instead of writing all runs to `/workspace` root.

Default:

```text
/workspace/runs/<coderun-uid>/morgan-events.jsonl
/workspace/runs/<coderun-uid>/morgan-commands.jsonl
/workspace/runs/<coderun-uid>/morgan-status.json
/workspace/runs/<coderun-uid>/presence-inbox.jsonl       existing presence fallback path
/workspace/.agent_done                                  existing lifecycle sentinel
```

If a run subdirectory is unavailable, fall back to `/workspace/morgan-*.jsonl` for local development only.

### Event envelope

Every JSONL event must use this envelope:

```json
{
  "seq": 1,
  "ts": "2026-05-03T00:00:00.000Z",
  "source": "morgan-agent-sidecar",
  "session_id": "morgan_coderun-abc_1770000000",
  "coderun_id": "coderun-abc",
  "agent_id": "morgan",
  "type": "session_started",
  "payload": {}
}
```

Rules:

- `seq` is monotonically increasing within a session.
- `ts` is UTC RFC3339 with milliseconds.
- `source` is one of `morgan-agent-sidecar`, `acpx`, `lobster`, `hermes-presence-adapter`, `provider:<name>`.
- Unknown `type` values must be ignored by readers.
- Corrupt JSONL lines should be skipped with a warning, not fatal to the whole run.

### Event types

Required v1 event types:

| Type | Producer | Purpose |
|---|---|---|
| `sidecar_ready` | sidecar | MCP and workspace stream ready. |
| `session_started` | sidecar | Morgan session was created/attached. |
| `provider_status` | sidecar | Active/fallback provider state changed. |
| `state` | sidecar/ACPX | UI/avatar state changed. |
| `turn_queued` | sidecar | Speech/text turn accepted. |
| `turn_started` | provider/sidecar | Speech/video output began. |
| `turn_completed` | provider/sidecar | Speech/video output completed. |
| `interrupted` | sidecar | User/agent interrupted current output. |
| `video_surface` | sidecar | Browser/LiveKit surface URL available. |
| `transcript_partial` | sidecar | Optional partial STT transcript. |
| `transcript_final` | sidecar | Optional finalized transcript. |
| `error` | sidecar | Recoverable or fatal error. |
| `session_stopped` | sidecar | Session closed. |

### Command log

The command log is optional for direct MCP callers but required for file-transport fallback and replay.

Command envelope:

```json
{
  "seq": 1,
  "ts": "2026-05-03T00:00:00.000Z",
  "source": "acpx",
  "session_id": "morgan_coderun-abc_1770000000",
  "command_id": "cmd_000001",
  "type": "say",
  "payload": { "text": "Hello", "interrupt": true }
}
```

Supported command types mirror MCP tools:

```text
start_session, stop_session, say, set_state, interrupt, provider_switch, tail_events
```

### Status snapshot

`morgan-status.json` is a single-object snapshot overwritten atomically with write-temp + rename.

Schema:

```json
{
  "session_id": "morgan_coderun-abc_1770000000",
  "coderun_id": "coderun-abc",
  "agent_id": "morgan",
  "status": "ready",
  "state": "idle",
  "active_provider": "lemonslice",
  "fallback_used": false,
  "surface": { "kind": "livekit_room", "url": "https://..." },
  "last_event_seq": 42,
  "last_error": null,
  "updated_at": "2026-05-03T00:00:00.000Z"
}
```

## Controller wiring

### Minimal implementation path: env-driven sidecar

Do not add a broad CRD schema first. The lowest-risk implementation mirrors the current `hermes-presence-adapter` pattern:

1. Add controller config for Morgan sidecar defaults:

   ```yaml
   controller:
     morgan:
       enabled: false
       image: ghcr.io/5dlabs/morgan-agent-sidecar:0.1.0
       mcpPort: 4000
       healthPort: 4001
   ```

2. Enable per CodeRun through existing `spec.env` keys:

   ```yaml
   spec:
     harnessAgent: hermes
     env:
       MORGAN_AGENT_ENABLED: "true"
       MORGAN_MODE: "meet"
       MORGAN_PROVIDER_MODE: "auto"
   ```

3. In `resources.rs`, add a helper next to `build_hermes_presence_adapter_spec`:

   ```rust
   fn build_morgan_agent_sidecar_spec(
       code_run: &CodeRun,
       config: &ControllerConfig,
       workspace_subdir: &str,
   ) -> Option<serde_json::Value>
   ```

   Conditions:

   - `code_run.spec.effective_harness() == HarnessAgent::Hermes`
   - `MORGAN_AGENT_ENABLED == "true"` or controller `morgan.enabled == true` with an allowlist for Morgan agent CodeRuns

4. Add the returned container to the pod container list exactly like `hermes-presence-adapter` is added.

5. Inject corresponding env vars into the main container so the harness and generated MCP config can discover Morgan:

   ```text
   MORGAN_MCP_URL=http://127.0.0.1:4000/mcp
   MORGAN_HEALTH_URL=http://127.0.0.1:4001/healthz
   MORGAN_SESSION_ID=morgan_${CODERUN_ID}_${epoch}
   MORGAN_STREAM_DIR=/workspace/<workspace_subdir>
   MORGAN_EVENT_LOG=/workspace/<workspace_subdir>/morgan-events.jsonl
   MORGAN_COMMAND_LOG=/workspace/<workspace_subdir>/morgan-commands.jsonl
   MORGAN_STATUS_FILE=/workspace/<workspace_subdir>/morgan-status.json
   ```

### Later typed CRD fields

Once the env-driven path is validated, promote the stable surface to typed fields:

```yaml
spec:
  morgan:
    enabled: true
    mode: meet
    image: ghcr.io/5dlabs/morgan-agent-sidecar:0.1.0
    mcpPort: 4000
    healthPort: 4001
    providerMode: auto
    livekit:
      enabled: true
      secretName: morgan-livekit
    avatar:
      primaryProvider: lemonslice
      fallback: symbolic
```

Avoid typed CRD fields until the sidecar API survives one end-to-end validation cycle.

## Harness and MCP config wiring

### `hermes.sh.hbs`

Add a conditional Morgan block after environment setup and before Lobster starts:

```bash
if [ "${MORGAN_AGENT_ENABLED:-false}" = "true" ]; then
  export MORGAN_MCP_URL="${MORGAN_MCP_URL:-http://127.0.0.1:${MORGAN_MCP_PORT:-4000}/mcp}"
  export MORGAN_HEALTH_URL="${MORGAN_HEALTH_URL:-http://127.0.0.1:${MORGAN_HEALTH_PORT:-4001}/healthz}"
  export MORGAN_SESSION_ID="${MORGAN_SESSION_ID:-morgan_${CODERUN_NAME:-$(hostname)}_$(date +%s)}"
  export MORGAN_STREAM_DIR="${MORGAN_STREAM_DIR:-$WORKSPACE_DIR}"
  export MORGAN_EVENT_LOG="${MORGAN_EVENT_LOG:-$MORGAN_STREAM_DIR/morgan-events.jsonl}"
  export MORGAN_COMMAND_LOG="${MORGAN_COMMAND_LOG:-$MORGAN_STREAM_DIR/morgan-commands.jsonl}"
  export MORGAN_STATUS_FILE="${MORGAN_STATUS_FILE:-$MORGAN_STREAM_DIR/morgan-status.json}"
fi
```

Then run a `morgan-init` health gate before `lobster run`:

```bash
if [ "${MORGAN_AGENT_ENABLED:-false}" = "true" ]; then
  deadline=$(($(date +%s) + ${MORGAN_INIT_TIMEOUT_SECS:-30}))
  until curl -fsS "$MORGAN_HEALTH_URL" >/tmp/morgan-health.json 2>/tmp/morgan-health.err; do
    if [ "$(date +%s)" -gt "$deadline" ]; then
      echo "[morgan-init] ERROR: Morgan sidecar did not become ready" >&2
      cat /tmp/morgan-health.err >&2 || true
      exit 1
    fi
    sleep 1
  done

  mkdir -p "$MORGAN_STREAM_DIR"
  printf '{"session_id":"%s","agent_id":"morgan","status":"ready","updated_at":"%s"}\n' \
    "$MORGAN_SESSION_ID" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$MORGAN_STATUS_FILE"
  echo "[morgan-init] Morgan sidecar ready: $MORGAN_MCP_URL"
fi
```

### Generated MCP config

The controller already generates MCP config for supported CLIs in `templates.rs`. Add the Morgan server to generated `mcpServers` when the sidecar is enabled:

```json
{
  "mcpServers": {
    "tools": { "command": "tools", "args": ["--url", "..."] },
    "morgan": {
      "type": "http",
      "url": "http://127.0.0.1:4000/mcp"
    }
  }
}
```

Do not rely only on an `MCP_SERVERS` env var unless the selected CLI actually consumes it. The design target is generated config because ACPX already supports configured `mcpServers`.

## Interaction with existing Hermes presence adapter

Keep responsibilities separate:

| Component | Owns | Does not own |
|---|---|---|
| `hermes-presence-adapter` | Discord bridge route registration; inbound Discord normalization; outbound status/send intents. | Avatar provider routing; LiveKit sessions; Morgan speech/video lifecycle. |
| `morgan-agent` sidecar | Morgan sessions; avatar/provider routing; MCP tools; workspace Morgan event streams. | Discord credentials; centralized route fanout; OpenClaw gateway behavior. |
| Main Hermes container | Lobster/ACPX reasoning; task execution; MCP calls. | Long-running meeting bot lifecycle. |

Integration rule:

- If an inbound Discord event arrives through `presence-inbox.jsonl`, Lobster/ACPX can decide to call `morgan_say`, `morgan_set_state`, or meeting tools.
- The Morgan sidecar may publish status events, but user-visible Discord messages should still go through the centralized bridge path, not direct Discord API calls from Morgan.

## Security and privacy

- The Morgan sidecar must never receive Discord bot tokens.
- Provider API keys are K8s secrets projected into the sidecar only.
- Workspace streams may include meeting URLs only when they are already intended for the agent. If URLs are signed, redact before long-term memory storage.
- Consent/disclosure is required for meeting entry and recording/transcription.
- The sidecar should default to audio/text/symbolic fallback if avatar provider configuration is absent.
- Health endpoints are pod-local only. No Service/Ingress by default.

## Implementation sequence

### Phase 0: sidecar stub

- Build a stub `morgan-agent-sidecar` image that exposes `/healthz` and MCP tools returning deterministic responses.
- It should write `sidecar_ready` and `session_started` events to JSONL.
- No provider integration yet.

### Phase 1: controller + harness wiring

- Add env-driven `build_morgan_agent_sidecar_spec` in `resources.rs`.
- Add main-container env vars.
- Add generated MCP config entry.
- Add `morgan-init` health gate in `hermes.sh.hbs`.
- Unit-test rendered pod spec and MCP config.

### Phase 2: local/provider loop

- Implement `morgan_session_start`, `morgan_session_stop`, `morgan_session_status`, `morgan_say`, `morgan_set_state`, `morgan_events_tail`.
- Add command/event/status replay tests using local filesystem.
- Wire LiveKit/LemonSlice only after the stub path is green.

### Phase 3: Morgan Meet compatibility

- Add `meet_join`, `meet_leave`, `meet_get_status`, `meet_stream_audio` aliases.
- Validate against `/opt/data/workspace/morgan-meet/.plan/spec/hermes-adapter.md` event expectations.

## Validation plan

### Unit tests

Controller tests:

- Hermes + `MORGAN_AGENT_ENABLED=true` adds `morgan-agent` container.
- Non-Hermes harness never adds the sidecar.
- Disabled Morgan env never adds the sidecar.
- Main container receives `MORGAN_MCP_URL`, `MORGAN_SESSION_ID`, and stream paths.
- Generated MCP config includes `morgan` server only when sidecar is enabled.
- Secret env refs are optional/missing-safe for providers not used in stub mode.

Sidecar tests:

- `/healthz` reports workspace writability.
- MCP `tools/list` includes required v1 tools.
- `morgan_session_start` writes `session_started` and status snapshot.
- `morgan_say` writes `turn_queued`.
- `morgan_events_tail` returns events after a seq cursor.
- SIGTERM writes `session_stopped` for active sessions.

### Pod smoke test

Create a Hermes CodeRun with:

```yaml
spec:
  harnessAgent: hermes
  env:
    MORGAN_AGENT_ENABLED: "true"
    MORGAN_PROVIDER_MODE: "stub"
```

Expected observations:

1. Pod has main container, `hermes-presence-adapter` when presence enabled, and `morgan-agent` sidecar.
2. Main container logs `[morgan-init] Morgan sidecar ready` before Lobster starts.
3. Generated MCP config includes `morgan`.
4. `/workspace/<run>/morgan-status.json` exists.
5. `/workspace/<run>/morgan-events.jsonl` has `sidecar_ready`.
6. A minimal Lobster/ACPX step can call `morgan_session_status`.
7. `/workspace/.agent_done` behavior remains unchanged.

### End-to-end acceptance

A Wave 2B implementation is unblocked when:

- A stub Morgan sidecar can be enabled per Hermes CodeRun.
- ACPX can discover and call Morgan MCP tools.
- Workspace stream files are created and contain valid JSONL.
- Existing Hermes presence adapter behavior remains unchanged.
- No OpenClaw gateway is required in the pod.

## Non-goals for this wave

- Full LiveKit/LemonSlice production integration.
- External ingress for the sidecar.
- A new OpenClaw plugin model.
- Broad CRD schema migration before env-driven validation.
- Persistent long-term memory policy; see `morgan-memory-skills-policy.md`.
