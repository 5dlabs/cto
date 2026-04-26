# Morgan Meet: Hermes Adapter Design

> Companion to [`5dlabs/morgan-meet`](https://github.com/5dlabs/morgan-meet)  
> Covers Phase 4 of the Morgan Meet architecture: **Hermes sidecar/file transport adapter**  
> and identifies upstream OpenClaw PR opportunities that benefit both runtimes.

---

## 1. The Problem: OpenClaw Plugin Assumptions vs. Hermes Reality

The Morgan Meet architecture (`.plan/spec/architecture.md`) calls for runtime adapters in Phase 4. The OpenClaw adapter is straightforward — plugins have a native SDK. The Hermes adapter requires a different mental model because Hermes has no gateway host.

### What OpenClaw's plugin model assumes a Meet bot can do

```
OpenClaw gateway (host process)
  ├── registerGatewayMethod("meet.join", handler)     ← RPC over gateway
  ├── registerService({ id: "meet-bot", start, stop }) ← long-lived worker
  ├── registerHttpRoute("/meet/webhook")               ← HTTP under /plugins/
  └── registerCli("meet-setup")                       ← CLI subcommand
      └── spawns: meet-bot sidecar (child_process)
            gets: OPENCLAW_MEET_GATEWAY_URL + _TOKEN
            writes events → gateway (all participants see them)
```

The gateway is the plugin host. It injects credentials, owns the sidecar lifecycle, routes RPC, and exposes HTTP routes. The plugin manifest (`openclaw.plugin.json`) declares config schema and UI hints.

### What Hermes actually provides

```
Kubernetes pod (CodeRun CRD)
  ├── lobster (workflow orchestration)
  │     └── ACPX (LLM reasoning step)
  ├── /workspace/.agent_done        ← file-sentinel lifecycle
  ├── /workspace/ (shared volume)   ← all coordination
  ├── MCP tool servers              ← ACPX tool surface (localhost ports)
  └── [sidecar containers]          ← Kubernetes, not gateway-spawned
```

**No gateway process. No `registerGatewayMethod`. No plugin registry.**  
Hermes coordinates through files, env vars, lobster steps, and MCP tools.

---

## 2. Concrete Hermes Adapter Design

The principle: **map every OpenClaw plugin primitive to its nearest Hermes equivalent** without faking a gateway or adding unnecessary abstraction.

### 2.1 MCP Control Surface (replaces `registerGatewayMethod`)

The Meet bot sidecar exposes an MCP server on `localhost:4000`. The ACPX model calls it like any other MCP tool:

```
meet_join(room_url, session_id, provider?)   → session_id
meet_leave(session_id)                        → void
meet_get_status(session_id)                   → PresenceStatus
meet_stream_audio(session_id, audio_url)      → void
```

The MCP server URL is injected by the harness:

```bash
# In hermes.sh.hbs (conditional block when meet sidecar is enabled)
export MORGAN_MEET_MCP_URL="http://localhost:${MORGAN_MEET_MCP_PORT:-4000}"
export MORGAN_MEET_SESSION_ID="mmeet_${CODERUN_NAME:-$(uname -n)}_$(date +%s)"
```

`MORGAN_MEET_MCP_URL` is appended to `MCP_SERVERS` so ACPX discovers it on startup. This is the exact pattern for any sidecar-based MCP tool: the harness injects the URL, ACPX discovers it, no gateway needed.

**Why MCP and not raw HTTP?** MCP is already ACPX's native tool surface. Using raw HTTP would require the model to construct requests manually. MCP gives schema, discovery, and error typing for free.

### 2.2 Kubernetes Sidecar (replaces `registerService`)

In the CodeRun CRD spec, Morgan Meet bot runs as a second container in the same pod:

```yaml
spec:
  sidecars:
    - name: morgan-meet-bot
      image: ghcr.io/5dlabs/morgan-meet-bot:latest
      env:
        - name: LIVEKIT_URL
          valueFrom:
            secretKeyRef: { name: cto-secrets, key: livekit-url }
        - name: RECALL_API_KEY
          valueFrom:
            secretKeyRef: { name: cto-secrets, key: recall-api-key }
        - name: MORGAN_MEET_MCP_PORT
          value: "4000"
        - name: WORKSPACE
          value: /workspace
      volumeMounts:
        - name: workspace
          mountPath: /workspace
      ports:
        - name: mcp
          containerPort: 4000
          protocol: TCP
```

The Kubernetes pod scheduler owns the sidecar lifecycle — it starts before the main container, restarts on failure, and terminates with the pod. This is the Hermes equivalent of `registerService({ id, start, stop })` without writing a single line of lifecycle code.

### 2.3 File Stream Coordination (replaces gateway event routing)

Two JSONL files in `/workspace/` carry all event traffic:

```
/workspace/meet-events.jsonl   ← sidecar appends; ACPX tails via lobster step or MCP poll
/workspace/meet-commands.jsonl ← ACPX appends (via MCP tool write or direct); sidecar watches
/workspace/meet-status.json    ← sidecar writes current state; lobster steps poll
```

Event schema matches the architecture doc contract exactly:

```json
{"type": "session_started", "session_id": "mmeet_...", "agent_id": "morgan", "surface": {"kind": "livekit_room", "url": "https://..."}}
{"type": "provider_status", "session_id": "mmeet_...", "mode": "auto", "active_provider": "lemonslice", "fallback_used": false}
{"type": "video_surface", "session_id": "mmeet_...", "surface": {"kind": "webpage", "url": "https://avatar.5dlabs.ai/meeting-bot?session=..."}}
```

This extends the existing Hermes sentinel pattern (`/workspace/.agent_done`) to a streaming event log. No new coordination primitive is required.

### 2.4 Lobster Initialization Step (replaces gateway startup)

A `meet-init` step in the CodeRun lobster workflow gates ACPX startup until the sidecar is ready:

```yaml
- id: meet-init
  command: |
    deadline=$(($(date +%s) + 30))
    while ! curl -sf http://localhost:${MORGAN_MEET_MCP_PORT:-4000}/health > /dev/null 2>&1; do
      if [ $(date +%s) -gt $deadline ]; then
        echo "[meet-init] ERROR: meet sidecar did not become ready within 30s" >&2
        exit 1
      fi
      sleep 1
    done
    # Write initial session identity
    echo '{"session_id":"'"$MORGAN_MEET_SESSION_ID"'","agent_id":"morgan","status":"ready"}' \
      > "$WORKSPACE/meet-status.json"
    echo "[meet-init] Meet sidecar ready: $MORGAN_MEET_MCP_URL"
```

This replaces the implicit ordering guarantee that OpenClaw's gateway provides when it spawns the sidecar process and waits for registration — Hermes makes it explicit in the workflow graph.

### 2.5 CLI Wrapper for Local Dev (replaces `registerCli`)

For local development without a Kubernetes pod, a shell wrapper (`scripts/morgan-meet-local.sh` in the morgan-meet repo) mocks the sidecar:

```bash
#!/usr/bin/env bash
# Starts the Morgan Meet bot sidecar locally (no cluster required)
set -euo pipefail
WORKSPACE="${WORKSPACE:-$(pwd)/workspace}"
mkdir -p "$WORKSPACE"
docker run --rm \
  -p "${MORGAN_MEET_MCP_PORT:-4000}:4000" \
  -v "$WORKSPACE:/workspace" \
  -e LIVEKIT_URL="${LIVEKIT_URL:?required}" \
  -e RECALL_API_KEY="${RECALL_API_KEY:-}" \
  -e MORGAN_MEET_MCP_PORT=4000 \
  -e WORKSPACE=/workspace \
  ghcr.io/5dlabs/morgan-meet-bot:latest
```

Usage mirrors the OpenClaw pattern (`openclaw openclaw-avatar-setup`) without depending on the gateway: just `source .env && ./scripts/morgan-meet-local.sh`.

---

## 3. Comparison Table: OpenClaw Plugin vs. Hermes Adapter

| OpenClaw plugin mechanism | Hermes adapter equivalent | Notes |
|---|---|---|
| `registerGatewayMethod("meet.join")` | MCP tool `meet_join` on `localhost:4000` | Same interface to ACPX; no gateway needed |
| `registerService({ id, start, stop })` | Kubernetes sidecar container | Pod scheduler owns lifecycle |
| `registerHttpRoute("/meet/webhook")` | Sidecar HTTP on localhost (not external) | Only pod-local; ingress optional |
| `registerCli("meet-setup")` | Shell wrapper `morgan-meet-local.sh` | No gateway; Docker or process-local |
| Gateway event routing to all agents | `/workspace/meet-events.jsonl` | Same event contract JSON, file transport |
| Plugin config `openclaw.plugin.json` | CodeRun CRD spec fields + env vars | Schema lives in CRD, not plugin manifest |
| `OPENCLAW_AVATAR_GATEWAY_URL` injection | `MORGAN_MEET_MCP_URL` injection in harness | Same pattern, different transport |
| `OPENCLAW_AVATAR_GATEWAY_TOKEN` | K8s projected secret / pod service account | Auth via pod identity, not gateway token |
| Plugin startup ordering (gateway wait) | `meet-init` lobster step with health poll | Explicit in workflow graph |
| Gateway restart/crash recovery | Kubernetes sidecar restart policy | Pod-level restart, not gateway-managed |

**Key insight:** the ACPX model's experience is nearly identical in both runtimes. It sees MCP tools. It does not see whether those tools are backed by an OpenClaw gateway plugin or a Hermes pod sidecar. The adapter layer is invisible to the agent logic.

---

## 4. Presence Bridge as the Runtime-Neutral Core

The architecture doc's "presence bridge" component owns session IDs, provider routing, fallback logic, and the event contract. In Hermes, the presence bridge lives **inside the sidecar container** (not in the gateway). Its API surface is the MCP server.

```
Hermes pod
  ├── main container (lobster + ACPX)
  │     └── calls MCP tools → meet_join, meet_get_status
  └── morgan-meet-bot sidecar
        ├── MCP server (localhost:4000)         ← presence bridge API
        ├── Presence bridge logic               ← session, provider, fallback
        ├── /workspace/meet-events.jsonl write  ← event log
        ├── /workspace/meet-status.json write   ← current state
        └── Meeting platform connector          ← Recall.ai / browser automation
```

The presence bridge is the same codebase in both runtimes. Only the **transport adapter** changes:
- OpenClaw: bridge registers gateway methods; events go through gateway
- Hermes: bridge exposes MCP server; events go to JSONL file
- Pipecat/RTVI: bridge speaks RTVI protocol on WebSocket

---

## 5. Upstream OpenClaw PR Opportunities

These are concrete contributions that would make OpenClaw easier to target from Hermes-adjacent runtimes without breaking existing plugin consumers.

### PR-1: `standalone: true` plugin mode in `openclaw.plugin.json`

```json
{
  "name": "@5dlabs/morgan-meet",
  "runtime": "standalone",
  "mcpPort": 4000,
  "services": [{ "id": "meet-bot", "command": "node dist/bot.js" }]
}
```

In `standalone` mode:
- `registerGatewayMethod(name, handler)` → exposes as MCP tool on `mcpPort`
- `registerService(opts)` → spawned as a child process (not child of gateway)
- `registerHttpRoute(route)` → HTTP on a configurable localhost port
- `registerCli(definition)` → registered as a CLI subcommand without gateway

This lets the same plugin codebase run in both gateway-hosted (OpenClaw) and standalone (Hermes, local dev, CI) contexts. The manifest's `runtime` field selects the mode.

**Target file:** `openclaw-plugin-sdk` package — the `register*` methods check `process.env.OPENCLAW_STANDALONE` or the manifest `runtime` field and switch backends accordingly.

### PR-2: File stream event adapter built into the gateway

Add a `fileStreamEvents` option to the OpenClaw gateway config:

```json
{
  "fileStreamEvents": {
    "path": "/workspace/meet-events.jsonl",
    "filter": ["meet.*"]
  }
}
```

Gateway events matching the filter are appended to the JSONL file. This makes Hermes file-stream consumers compatible with gateway-hosted plugin events without any code change in the bridge logic.

**Target file:** `openclaw-gateway` — the event bus emitter, after dispatch, checks for registered file stream adapters.

### PR-3: MCP bridge for gateway methods

An OpenClaw plugin (`@clawdbot/mcp-gateway-bridge`) that auto-exposes all registered gateway methods as MCP tools:

```typescript
// inside the plugin
for (const [method, handler] of gatewayMethods) {
  mcpServer.addTool({ name: method, handler })
}
```

This allows ACPX models in OpenClaw itself to call gateway methods via MCP (consistent with the ACPX tool surface), and allows external processes (Hermes pods in the same cluster) to call gateway methods without gateway authentication.

### PR-4: Lobster step definitions for `meet-init` and `avatar-init`

Contribute named step types to the lobster standard library:

```yaml
- id: meet-init
  uses: cto/meet-init@v1
  with:
    mcp_port: 4000
    timeout_seconds: 30
    workspace: $WORKSPACE
```

Step type definitions encode the health poll + sentinel write logic without duplication across every harness that uses a Meet sidecar. This is the lobster equivalent of a reusable GitHub Actions step.

---

## 6. Phase 4 Implementation Checklist

When morgan-meet advances to Phase 4 (runtime adapters):

### Hermes adapter (this design)

- [ ] Create `morgan-meet-bot` container image with MCP server on port 4000
- [ ] Implement presence bridge inside the sidecar (session, provider routing, fallback)
- [ ] Add JSONL file writer for events to `/workspace/meet-events.jsonl`
- [ ] Add CodeRun CRD sidecar spec fields to `infra/charts/cto/crds/coderun-crd.yaml`
- [ ] Add `meet-init` lobster step to hermes harness template (`templates/harness-agents/hermes.sh.hbs`)
- [ ] Add `MORGAN_MEET_MCP_URL` + `MORGAN_MEET_SESSION_ID` injection to harness
- [ ] Write `scripts/morgan-meet-local.sh` wrapper for dev
- [ ] Update `docs/morgan-avatar-openclaw-handoff.md` AvatarProvider interface to include presence bridge contract

### OpenClaw adapter

- [ ] Create OpenClaw plugin wrapper in morgan-meet repo (`openclaw-plugin/`)
- [ ] Register gateway methods as thin wrappers over presence bridge
- [ ] Register `morgan-meet` as a service in the gateway plugin manifest
- [ ] Add plugin config schema to `openclaw.plugin.json`

### Shared

- [ ] Finalize event contract schema (JSON Schema or TypeScript types) in morgan-meet repo
- [ ] Write consent/entry-message language per FR-5
- [ ] Document `morgan@5dlabs.ai` Google account flow for Phase 3 → Phase 4 bridge identity

---

## 7. Known Constraints (from existing research)

- **EchoMimic is async-only** — not suitable as the primary live meeting avatar. LemonSlice or equivalent realtime provider required for low-latency meeting mode.
- **Queue latency is the first bottleneck** — not TTS. Morgan backend queue depth was the observed failure mode in the avatar PoC; the Meet sidecar's event loop must not add to that queue.
- **Desktop owns the session contract** — when CTO desktop (Pixel/Tauri) is in the loop, it owns Morgan sessioning and gateway transport. The Hermes sidecar is a server-side, headless-only surface.
- **Pika is proof, not foundation** — the managed-bot output-media pattern is correct; Pika's specific API is too opaque/beta for the product foundation. Use Recall.ai or Meeting BaaS.
- **AvatarProvider interface** — the existing adapter boundary from `docs/morgan-avatar-openclaw-handoff.md` remains the contract:

```typescript
interface AvatarProvider {
  readonly mode: "live-realtime" | "async-turn-video";
  capabilities(): { video: boolean; audio: boolean; streaming: boolean };
  startSession(input: AvatarSessionInput): Promise<AvatarSession>;
  renderTurn(input: AvatarTurnInput): Promise<AvatarTurnResult>;
  stopSession(sessionId: string): Promise<void>;
}
```

The Hermes sidecar implements this interface behind the MCP tool surface. The OpenClaw plugin implements it behind gateway methods. The ACPX model never sees the difference.
