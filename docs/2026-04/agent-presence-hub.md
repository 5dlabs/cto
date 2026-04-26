# Agent Presence Hub — Generic Design

> **Status**: design-spec (2026-07).  
> **Replaces**: `docs/morgan-meet-hermes-design.md` — this doc supersedes the Morgan-only framing.  
> **Companion**: `morgan-meet/.plan/spec/hermes-adapter.md` (low-level Hermes adapter; still accurate for its scope).  
> **Related**: [`avatar-architecture.md`](./avatar-architecture.md), [`morgan-avatar-openclaw-handoff.md`](../2026-03/morgan-avatar-openclaw-handoff.md), [`acp-nats-research.md`](../2026-03/acp-nats-research.md).

---

## 1. Why "generic"

The original "Morgan Meet" framing locked the presence layer to a single agent. The actual problem is infrastructure-level: every CTO agent (Morgan, Rex, Blaze, Bolt, Nova, …) may eventually need to appear in a meeting, stream audio/video, or join a voice call. The hub must serve any agent, running under any runtime — OpenClaw gateway or Hermes standalone.

**Scope of this document:**
- Design of the Agent Presence Hub as a runtime-neutral self-hosted service
- Two runtime integration paths: OpenClaw gateway plugin adapter, Hermes/acpx MCP sidecar
- Upstream PR analysis — where does a minimal, engineering-correct PR land? And what is the marketing upside?
- Non-Morgan use cases that validate the generic shape

---

## 2. What the hub is (and is not)

| Is | Is not |
|---|---|
| A self-hostable service exposing REST + WebSocket + MCP | An OpenClaw plugin itself |
| A provider router (LemonSlice ↔ EchoMimic ↔ audio-only) | Tied to any single agent or model |
| A meeting connector (LiveKit native, Google Meet via Recall.ai) | A video conferencing product |
| A thin adapter surface for OpenClaw gateway and Hermes/acpx | A replacement for OpenClaw or acpx |
| Open-source, Docker Compose + Helm deployable | A managed SaaS |

The hub is positioned at the **infrastructure layer** — the same layer as the NATS cluster and the LiveKit SFU. Individual agents call into it via HTTP or MCP tools; they do not embed avatar rendering code.

---

## 3. Core architecture

```
                         Agent Presence Hub
              ┌────────────────────────────────────────┐
              │  Control API                            │
              │  HTTP REST  ·  WebSocket  ·  MCP tools │
              │  (runtime-neutral; any agent calls in)  │
              ├────────────────────────────────────────┤
              │  Session Router                         │
              │  · agent registry (any agent ID)        │
              │  · capability negotiation per-session   │
              │  · session lifecycle FSM                │
              │    (idle → joining → live → leaving)    │
              ├────────────────────────────────────────┤
              │  Provider Registry                      │
              │  · LemonSlice   (realtime, <300 ms)     │
              │  · narrator-sidecar (MuseTalk/Hunyuan)  │
              │  · EchoMimic    (async MP4, not live)   │
              │  · audio-only   (TTS → LiveKit track)   │
              │  · plugin slot  (Pipecat-compatible)    │
              ├────────────────────────────────────────┤
              │  Meeting Connectors                     │
              │  · LiveKit native   (SFU bot in-room)   │
              │  · Google Meet      (Recall.ai browser) │
              │  · Zoom             (Recall.ai, phase 2)│
              ├────────────────────────────────────────┤
              │  Runtime Adapters                       │
              │  · OpenClaw gateway plugin adapter      │
              │  · acpx MCP sidecar (Hermes path)       │
              │  · Direct HTTP (any agent, no adapter)  │
              └────────────────────────────────────────┘
```

**Key invariant**: the hub always outputs a **LiveKit participant** — a bot in a LiveKit room that publishes audio/video tracks. All five LemonSlice repos terminate in a LiveKit room; all LemonSlice providers and the narrator-sidecar design already assume this. Meeting connectors consume LiveKit tracks and inject them into the target platform (Google Meet, etc.) via Recall.ai or native WebRTC.

---

## 4. Event contract

Every agent communicates with the hub using a platform-neutral event envelope. This decouples the agent runtime from the avatar provider.

```typescript
// Presence session lifecycle events (agent → hub)
interface PresenceEvent {
  sessionId: string;          // hub-assigned after join
  agentId:   string;          // "morgan" | "rex" | "blaze" | ...
  meetingId?: string;         // Google Meet URL, LiveKit room name, etc.
  event:
    | { type: "join";        capabilities: AgentCapabilities }
    | { type: "turn_start";  turnId: string; context?: string }
    | { type: "turn_chunk";  turnId: string; text: string; final?: boolean }
    | { type: "turn_end";    turnId: string }
    | { type: "silence";     durationMs: number }
    | { type: "leave";       reason?: string };
  ts: number; // Unix ms
}

interface AgentCapabilities {
  voice:     boolean;    // can emit TTS audio
  video:     boolean;    // can emit avatar video
  provider?: string;     // preferred: "lemonslice" | "narrator" | "audio"
  model?:    string;     // provider-specific model/avatar ID
}

// Hub → agent notification events (webhooks / SSE / JSONL tail)
interface HubNotification {
  sessionId: string;
  event:
    | { type: "session_ready";  livekit_room: string; participant_token: string }
    | { type: "turn_rendered";  turnId: string; durationMs: number }
    | { type: "meeting_joined"; platform: string; roomUrl: string }
    | { type: "error";          code: string; message: string };
  ts: number;
}
```

The JSONL file format (`/workspace/presence-events.jsonl`) is exactly this schema, one record per line — usable by the Hermes file-sentinel pattern without any changes to the agent.

---

## 5. OpenClaw integration path

### 5.1 Plugin adapter (`@cto/openclaw-presence`)

This is a thin OpenClaw gateway plugin that relays calls to the hub service. The hub itself is not a plugin; the plugin is just the adapter shim.

```typescript
// openclaw.plugin.json
{
  "name": "@cto/openclaw-presence",
  "version": "0.1.0",
  "entry": "dist/index.js",
  "config": {
    "$schema": "...",
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "hubUrl":  { "type": "string",  "default": "http://localhost:4100" },
      "agentId": { "type": "string" },
      "provider": {
        "type": "string",
        "enum": ["lemonslice", "narrator", "audio"],
        "default": "lemonslice"
      },
      "lemonsliceApiKey": { "type": "string", "env": "LEMONSLICE_API_KEY",  "sensitive": true },
      "recallApiKey":     { "type": "string", "env": "RECALL_API_KEY",      "sensitive": true }
    }
  }
}
```

```typescript
// plugin entry point
export function register(api: OpenClawPluginApi) {
  const hub = new HubClient(api.config.hubUrl, api.config.agentId);

  api.registerGatewayMethod("presence.session.join",  hub.join.bind(hub));
  api.registerGatewayMethod("presence.session.leave", hub.leave.bind(hub));
  api.registerGatewayMethod("presence.turn.send",     hub.turn.bind(hub));
  api.registerGatewayMethod("presence.status",        hub.status.bind(hub));

  api.registerService({
    id: "cto-presence-hub",
    start: () => hub.startProxy(),  // if hub is co-located; no-op if remote
    stop:  () => hub.stopProxy(),
  });

  api.registerHttpRoute({
    method: "GET",
    path:   "/plugins/cto-presence/health",
    handler: (_req, res) => res.json({ status: "ok", hubUrl: api.config.hubUrl }),
  });
}
```

Note: `runtime.videoAvatar` **is a reserved OpenClaw extension point** (confirmed from `lemonsliceai/openclaw-avatar/types/openclaw-plugin-sdk.d.ts`). The plugin adapter should implement the `runtime.videoAvatar.synthesizeSpeech` hook to allow OpenClaw itself to trigger avatar rendering through the hub.

### 5.2 Agent-side usage (OpenClaw agent)

After the plugin is installed, any OpenClaw agent calls gateway methods — no direct hub URL required:

```typescript
// From within an agent's gateway method handler
await gateway.call("presence.session.join", {
  meetingId: "https://meet.google.com/abc-defg-hij",
  capabilities: { voice: true, video: true, provider: "lemonslice" }
});

// During a turn:
await gateway.call("presence.turn.send", {
  sessionId, turnId: crypto.randomUUID(), text: "Here is my analysis…"
});
```

---

## 6. Hermes integration path (no gateway)

### 6.1 K8s sidecar pattern

The hub runs as a second container in the CodeRun pod, sharing the `/workspace/` volume. No gateway, no NATS — just a MCP server on `localhost:4100` and a JSONL event file.

```yaml
# In CodeRunSpec, when presence is enabled:
sidecars:
  - name: presence-hub
    image: ghcr.io/5dlabs/agent-presence-hub:latest
    ports:
      - containerPort: 4100   # MCP server
      - containerPort: 4101   # REST API
    env:
      - name: AGENT_ID
        value: "{{ .agentId }}"
      - name: HUB_PROVIDERS
        value: "lemonslice,audio"
      - name: LIVEKIT_URL
        valueFrom:
          secretKeyRef: { name: livekit, key: url }
      - name: LIVEKIT_API_KEY
        valueFrom:
          secretKeyRef: { name: livekit, key: apiKey }
      - name: LIVEKIT_API_SECRET
        valueFrom:
          secretKeyRef: { name: livekit, key: apiSecret }
    volumeMounts:
      - name: workspace
        mountPath: /workspace
```

The CRD co-change requirements (per the table in `CLAUDE.md`) apply if this becomes a standard CodeRun option:
- `crates/controller/src/tasks/code/resources.rs`
- `crates/controller/src/crds/coderun.rs` (`CodeRunSpec`)
- `infra/charts/cto/crds/coderun-crd.yaml` AND `infra/charts/cto-lite/crds/coderun-crd.yaml`

### 6.2 MCP tools exposed (for acpx)

When `MCP_SERVERS` env var contains the hub's MCP server URL (`http://localhost:4100/mcp`), acpx injects these tools into the agent's tool context automatically:

| MCP Tool | Args | Returns |
|---|---|---|
| `presence_join` | `meetingId, capabilities` | `{ sessionId, livekit_room }` |
| `presence_turn` | `sessionId, text, final?` | `{ turnId, accepted: true }` |
| `presence_leave` | `sessionId, reason?` | `{ ok: true }` |
| `presence_status` | `sessionId` | `SessionStatus` |

The agent tail-reads `/workspace/presence-events.jsonl` for turn-rendered notifications (same pattern as the existing `/workspace/.agent_done` sentinel).

### 6.3 Lobster step for meeting init

```yaml
# In agent's lobster workflow, before the main task:
- id: presence-init
  name: Join meeting if requested
  command: |
    if [ -n "$MEETING_URL" ]; then
      curl -sX POST http://localhost:4101/sessions \
        -H 'Content-Type: application/json' \
        -d "{\"agentId\":\"$AGENT_ID\",\"meetingId\":\"$MEETING_URL\",
             \"capabilities\":{\"voice\":true,\"video\":${HUB_VIDEO:-false}}}" \
        > /workspace/presence-session.json
      SESSION_ID=$(jq -r .sessionId /workspace/presence-session.json)
      export SESSION_ID
    fi

- id: main-task
  name: Run agent task
  command: acpx $ACPX_AGENT exec -f prompt.md
  # presence_turn is called by the agent during task execution via MCP tool

- id: presence-teardown
  name: Leave meeting
  command: |
    if [ -n "$SESSION_ID" ]; then
      curl -sX DELETE http://localhost:4101/sessions/$SESSION_ID
    fi
```

---

## 7. Provider registry

### 7.1 LemonSlice (realtime, recommended default)

- `@livekit/agents-plugin-lemonslice` already wraps LiveKit agents
- Hub creates a LiveKit room, spawns the LemonSlice agent runner as a sidecar process
- Agent text → hub → LemonSlice TTS/avatar → LiveKit room → meeting connector
- Latency: <300 ms lip-sync (per LemonSlice's stated SLA)
- License: `lemonsliceai/openclaw-avatar` is MIT; `lemonsliceai/agents-js` is Apache-2.0

### 7.2 Narrator-sidecar (MuseTalk/Hunyuan/EchoMimicV3)

- The narrator-sidecar (see `avatar-architecture.md` Δ2) joins the LiveKit room as a bot participant
- Hub sends text → narrator synthesizes → narrator publishes track to room
- Requires H100 GPU; not suitable for latency-sensitive meetings without hardware
- Async MP4 mode (EchoMimicV3) is explicit fallback for async meeting recording, not live

### 7.3 Audio-only fallback

- Hub calls ElevenLabs / Groq / local TTS → raw PCM → LiveKit audio track
- No avatar video; works with any inference budget
- Triggered when `capabilities.video = false` or provider is unavailable

### 7.4 Provider selection logic

```
join(capabilities):
  if capabilities.video && provider == "lemonslice" && lemonslice.available():
    → LemonSlice realtime
  elif capabilities.video && provider == "narrator" && gpu.available():
    → narrator-sidecar
  elif capabilities.video && provider == "narrator" && !gpu.available():
    → audio-only (degrade gracefully, notify agent via HubNotification.error)
  else:
    → audio-only
```

---

## 8. Meeting connectors

### 8.1 LiveKit native (default)

- Hub itself joins a LiveKit room; the meeting IS a LiveKit room
- Browser `AvatarTile` (in CTO sidebar or Morgan Meet frontend) joins via `livekit-client`
- Existing LiveKit infrastructure already deployed for intake

### 8.2 Google Meet (Recall.ai browser bot)

- Hub calls Recall.ai API to spawn a browser bot in the Google Meet room
- Recall bot receives audio from the meeting, forwards to hub as speech events
- Hub pushes avatar audio/video out via the Recall bot's `output_media` API
- Credentials: `meet@5dlabs.ai` OAuth token (stored in 1Password, injected via K8s Secret)
- Preferred over Pika (too opaque/beta — per prior avatar PoC research)

```
Agent → hub.turn(text)
         ↓
    LemonSlice renders audio/video
         ↓
    Recall.ai bot plays output_media in Google Meet room
         ↓
    Meeting participants see/hear the agent avatar
```

### 8.3 Zoom (Phase 2, listen-only first)

- Recall.ai supports Zoom; same pattern as Google Meet
- Listen-only (no output_media) is a safe Phase 2 start — observe meetings, transcribe, notify agent

---

## 9. Upstream PR analysis — Hermes / acpx / ACP

### 9.1 What "upstream Hermes" actually means

`Hermes` as defined in `crates/controller/src/crds/coderun.rs` is a **5dlabs-internal CRD variant**. There is no external "Hermes" framework to contribute to. The relevant upstream targets are:

| Project | Relationship | GitHub |
|---|---|---|
| `acpx` | The CLI powering Hermes's agent invocations | `github.com/openclaw/acpx` |
| ACP spec | The protocol `acpx` implements | `github.com/agentclientprotocol/agent-client-protocol` |
| `@clawdbot/lobster` | The orchestration DSL powering Hermes workflows | OpenClaw org |
| `livekit/agents-js` | The realtime agent framework all providers use | `github.com/livekit/agents-js` |

"NousResearch/Hermes" (the open-weight model series) is unrelated — there's no plugin surface in a language model.

### 9.2 The minimal acpx PR

**Target**: `github.com/openclaw/acpx`  
**PR title**: `feat: accept --presence-mcp-url to inject a presence MCP server into every new session`

When `--presence-mcp-url <URL>` is passed (or set via `ACPX_PRESENCE_MCP_URL` env var), acpx appends the presence hub's MCP server to the session's `mcpServers` list during `session/new`. The underlying agent then receives `presence_*` tools automatically — without any ACP protocol change, binary streaming, or new event types.

**Why this is engineering-correct:**
- MCP server injection is already the acpx extension model (`MCP_SERVERS` env var)
- Adding one more server URL to the injection list is a zero-protocol-change PR
- No binary transport; media stays in the hub service where it belongs
- Enables the Hermes sidecar pattern for any team using acpx, not just 5dlabs

**Estimated diff size**: ~20–40 lines (session initialization, CLI flag, env var parsing, test)

```diff
// Proposed change in acpx session/new handler (pseudocode)
const presenceMcpUrl = process.env.ACPX_PRESENCE_MCP_URL ?? opts.presenceMcpUrl;
if (presenceMcpUrl) {
  sessionConfig.mcpServers.push({ name: "presence", url: presenceMcpUrl });
}
```

### 9.3 Do NOT submit ACP spec `capabilities.media`

**Temptation**: add a `media` capability type to the ACP `session/initialize` handshake, making it look like CTO contributed to the cross-company standard.

**Why not:**
1. ACP is an **editor↔agent** protocol (Zed, JetBrains, Cursor inject ACP to talk to coding agents in the editor). Media/presence is **agent↔infrastructure** — wrong layer.
2. The spec group (Zed, JetBrains, Block) would be technically correct to reject it. "More publicity" is not a valid engineering rationale.
3. It conflates two orthogonal concerns: the ACP session handshake describes what the agent can do for the user inside the editor; it doesn't describe external infrastructure the agent uses.
4. A rejected spec PR hurts more than no PR (signals low protocol understanding).

### 9.4 Upstream tradeoff summary

| Target | Audience | PR effort | Engineering fit | Marketing upside |
|---|---|---|---|---|
| `openclaw/acpx` `--presence-mcp-url` | Headless ACP users | ~40 LOC | ✅ Correct layer | Low–medium |
| `livekit/agents-js` presence plugin | All LiveKit users | Medium | ✅ Correct layer | **High** (Apache-2.0, large ecosystem) |
| OpenClaw community plugin `@cto/openclaw-presence` | OpenClaw users | Medium | ✅ Correct layer | Low (small community) |
| `@clawdbot/lobster` step library | Lobster workflow users | Small | ✅ Correct layer | Low |
| ACP spec `capabilities.media` | Zed / JetBrains / Block | Large | ❌ Wrong layer | Highest (but likely rejected) |

**Recommended order:**
1. **Ship `@cto/openclaw-presence` plugin** — works today, validates the hub API surface
2. **Submit `openclaw/acpx --presence-mcp-url` PR** — tiny, correct, unlocks Hermes path for any team
3. **Explore `livekit/agents-js` plugin contribution** — highest engineering-correct visibility (LiveKit has millions of ecosystem reach, Apache-2.0, active maintainership)
4. **Skip ACP spec PR** — marketing rationale does not override engineering correctness

### 9.5 On authorship via GitHub Apps

CTO agents can author PRs via their GitHub apps (each agent has an app installation). The PR quality must be engineer-grade: clear problem statement, small diff, tests, CI passing. A PR authored by a CTO agent that fails review because it's architecturally wrong would damage credibility more than not submitting.

---

## 10. Self-hosted deployment

### 10.1 Docker Compose (local dev / small team)

```yaml
# docker-compose.yml
services:
  presence-hub:
    image: ghcr.io/5dlabs/agent-presence-hub:latest
    ports:
      - "4100:4100"   # MCP server (acpx / Hermes)
      - "4101:4101"   # REST API (lobster, direct HTTP)
      - "4102:4102"   # WebSocket (SSE notifications)
    environment:
      LIVEKIT_URL:          ${LIVEKIT_URL}
      LIVEKIT_API_KEY:      ${LIVEKIT_API_KEY}
      LIVEKIT_API_SECRET:   ${LIVEKIT_API_SECRET}
      LEMONSLICE_API_KEY:   ${LEMONSLICE_API_KEY}
      RECALL_API_KEY:       ${RECALL_API_KEY}          # optional
      HUB_PROVIDERS:        lemonslice,audio            # comma-separated priority list
      HUB_AGENT_ID_DEFAULT: morgan                      # overridden per-request
    volumes:
      - workspace:/workspace    # only needed for Hermes co-pod mode
```

### 10.2 Helm (production, co-deployed with CTO)

```yaml
# infra/charts/agent-presence-hub/values.yaml
hub:
  image:
    repository: ghcr.io/5dlabs/agent-presence-hub
    tag: latest
  providers:
    - lemonslice
    - audio
  livekit:
    secretName: livekit
  meetings:
    google_meet:
      enabled: true
      recall:
        secretName: recall-api-key
    zoom:
      enabled: false
  mcp:
    port: 4100
  rest:
    port: 4101
```

---

## 11. Non-Morgan use cases (validating generic scope)

| Agent | Scenario | Provider |
|---|---|---|
| **Morgan** | Live intake meeting with client stakeholders | LemonSlice realtime |
| **Rex** | Post-PR walkthrough recording (async) | EchoMimic MP4 |
| **Bolt** | Incident bridge (audio-only, Zoom listen-only) | Audio fallback |
| **Nova** | Research briefing to a Google Meet | LemonSlice + Recall.ai |
| **Blaze** | UI demo session with designer | LemonSlice + LiveKit |
| **Any agent** | No meeting; voice response in sidebar | Audio-only, LiveKit |

All six scenarios use the same hub API. The agent identifies itself via `agentId`; the hub picks the provider and meeting connector.

---

## 12. Implementation phases

| Phase | Deliverables | Depends on |
|---|---|---|
| **M0** | Hub service skeleton: REST API, session FSM, audio-only provider | LiveKit infra |
| **M1** | LemonSlice provider adapter; OpenClaw plugin adapter `@cto/openclaw-presence` | LemonSlice API key |
| **M2** | Hermes MCP sidecar adapter; CodeRun CRD optional `presence` sidecar field | CRD co-change |
| **M3** | Google Meet connector via Recall.ai | Recall API key + `meet@5dlabs.ai` account |
| **M4** | narrator-sidecar provider (MuseTalk/Hunyuan via LiveKit) | H100 GPU |
| **M5** | `acpx --presence-mcp-url` upstream PR; `livekit/agents-js` plugin exploration | M0–M2 working |

M0–M1 can begin immediately (no GPU dependency). M2 unblocks Hermes for all agents. M3 delivers the Google Meet use case from the original Morgan Meet PRD.

---

## 13. Open questions

1. **Hub co-location vs. shared service**: For OpenClaw agents, the hub runs as a separate deployed service (shared across all agents). For Hermes, it runs as a K8s sidecar (per-pod). Should there be a single shared hub that Hermes pods connect to remotely, or per-pod? Per-pod is simpler for isolation; shared is cheaper.

2. **`meeting@5dlabs.ai` credential management**: Google Meet bots need persistent OAuth refresh tokens. These should live in 1Password and be rotated by a dedicated credentials rotation job — not embedded in the Recall.ai request.

3. **Pipecat compatibility**: LemonSlice's Pipecat fork (`lemonsliceai/pipecat`) adds a LemonSlice transport. If we want Pipecat pipeline compatibility (e.g., custom VAD, custom wake word), the hub's provider slot should accept a Pipecat `FrameProcessor` — but this is an M4+ concern.

4. **Morgan Meet frontend**: The morgan-meet repo (`5dlabs/morgan-meet`) serves as the **reference client** for the presence hub's browser UI — the `AvatarTile` component, the LiveKit room connection, the PiP window. The hub is backend infrastructure; Morgan Meet is one frontend that uses it. The repos remain separate.

---

## Sources

- [`docs/avatar-architecture.md`](./avatar-architecture.md) — LemonSlice deep-dive, `runtime.videoAvatar` extension point, narrator-sidecar Δ2 proposal
- [`docs/morgan-avatar-openclaw-handoff.md`](../2026-03/morgan-avatar-openclaw-handoff.md) — `AvatarProvider` interface, queue-latency lessons, adapter boundary
- [`docs/acp-nats-research.md`](../2026-03/acp-nats-research.md) — ACPX/ACP architecture, NATS approach, Approach A verdict
- [`crates/controller/src/crds/coderun.rs`](../../crates/controller/src/crds/coderun.rs) — `HarnessAgent::Hermes` definition
- [`templates/harness-agents/hermes.sh.hbs`](../../templates/harness-agents/hermes.sh.hbs) — Hermes runtime entrypoint, file sentinel pattern
- [`morgan-meet/.plan/spec/architecture.md`](../../morgan-meet/.plan/spec/architecture.md) — original presence bridge design, event contract
- `lemonsliceai/openclaw-avatar` (MIT) — reference implementation, OpenClaw plugin SDK surface, `runtime.videoAvatar` hook
