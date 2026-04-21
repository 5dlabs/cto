# Avatar Architecture — LemonSlice / OpenClaw integration

> **Status**: design-spec update (2026-04). Supersedes nothing yet, extends `avatar-report.md`.
> **Companion research**: full repo deep-dive lives in session state as `lemonslice-repo-deep-dive.md` (5 repos, ~24KB), summarized here for repo consumers.
> **Related**: [`agent-avatar-prompts.md`](./agent-avatar-prompts.md), [`morgan-avatar-openclaw-handoff.md`](./morgan-avatar-openclaw-handoff.md), [`../avatar-report.md`](../avatar-report.md) (open-source candidate research), [`../.firecrawl/building-plugins.md`](../.firecrawl/building-plugins.md) (OpenClaw plugin authoring scraped docs).

---

## Why this doc exists

While GPU provisioning (OVH AI Deploy H100 quota, DigitalOcean Hatch credits) was blocked, we deep-dived five repos from [LemonSlice](https://github.com/lemonsliceai) — notably their **MIT-licensed, production-shipping OpenClaw gateway plugin** `@lemonsliceai/openclaw-avatar`. That plugin **IS** the reference implementation for OpenClaw-native avatar integrations, and it directly informs both:

1. Our **intake-demo stopgap** (ship on LemonSlice + their plugin while H100s are pending), and
2. Our **long-term multi-provider avatar architecture** (narrator-sidecar for MuseTalk / Hunyuan / EchoMimicV3).

The repos analyzed:

| Repo | Purpose | License |
|---|---|---|
| `lemonsliceai/openclaw-avatar` | Production OpenClaw gateway plugin (PiP avatar chat) | MIT |
| `lemonsliceai/lemonslice-examples` | 5 reference apps (Daily hosted, LiveKit Py/Node, Pipecat) | Apache-2.0 |
| `lemonsliceai/agents` | Fork of LiveKit Python agents; source of `livekit-plugins-lemonslice` | Apache-2.0 |
| `lemonsliceai/agents-js` | Fork of LiveKit Node agents; source of `@livekit/agents-plugin-lemonslice` | Apache-2.0 |
| `lemonsliceai/pipecat` | Fork of Pipecat adding a LemonSlice transport | BSD-2 |

---

## Key architectural findings

### 1. `openclaw-avatar` plugin — 3-process design

```
┌─────────── OpenClaw gateway (host) ───────────┐
│ registers:                                     │
│   • gateway methods (avatar.chat.send,         │
│     avatar.session.{create,stop}, ...)         │
│   • HTTP routes under /plugins/openclaw-avatar │
│   • CLI: `openclaw openclaw-avatar-setup`      │
│   • service lifecycle (start/stop sidecar)     │
│ uses runtime.{tts,stt,videoAvatar,config}      │
└─────────────┬──────────────────────────────────┘
              │ child_process.spawn
              │  (OPENCLAW_AVATAR_GATEWAY_URL +
              │   _GATEWAY_TOKEN back to host)
              ▼
┌─────── avatar-agent-runner sidecar (Node) ────┐
│ @livekit/agents 1.2.0                          │
│ @livekit/agents-plugin-lemonslice 1.2.0        │
│ joins a LiveKit room as a bot participant      │
└─────────────┬──────────────────────────────────┘
              │ LiveKit WebRTC
              ▼
┌─────── Browser (PiP avatar chat UI) ──────────┐
│ livekit-client joins the same room             │
│ Document PiP API for floating FaceTime window  │
└────────────────────────────────────────────────┘
```

**All five LemonSlice repos terminate in a LiveKit room.** LiveKit is the universal transport; LemonSlice participates as "just another bot in the room."

### 2. OpenClaw Plugin SDK — the full surface area

Extracted verbatim from `openclaw-avatar/types/openclaw-plugin-sdk.d.ts`:

```typescript
export type OpenClawPluginApi = {
  runtime: {
    config:         { loadConfig, writeConfigFile },
    agent?:         { resolveAgentDir },
    tts?:           { textToSpeechTelephony({ text, cfg, prefsPath }) },
    videoAvatar?:   { synthesizeSpeech, transcribeAudio },    // ← first-class extension point
    mediaUnderstanding?: { transcribeAudioFile },
    stt:            { transcribeAudioFile },
  },
  logger:                   { info, warn, error, debug },
  registerGatewayMethod:    (method, handler) => void,
  registerService:          ({ id, start, stop }) => void,
  registerHttpRoute:        (route) => void,
  registerCli:              (definition, metadata?) => void,
  resolvePath:              (input: string) => string,
};
```

**`runtime.videoAvatar` is a reserved OpenClaw extension point** — upstream expects multi-provider video avatars, which validates our multi-backend design.

### 3. Plugin manifest patterns worth adopting

From `openclaw-avatar/openclaw.plugin.json`:

- **Provider enum pattern**: `avatar.provider: { "enum": ["lemonslice"] }` — extensible to `["lemonslice", "musetalk", "hunyuan", "echomimic"]`.
- **Secret indirection**: all API keys accept `string | { value: string } | { env: string }`. UI hints flag `sensitive: true`.
- **Config schema discipline**: `additionalProperties: false` at every level.
- **UI hints** per field (label, placeholder, helpText, sensitive) — OpenClaw settings UI renders these.

### 4. Canonical LiveKit agent — ~15 LOC

From `lemonslice-examples/03-livekit-app-python/agent/src/agent.py`:

```python
session = AgentSession(
    llm=inference.LLM(model="openai/gpt-4o-mini"),
    stt=inference.STT(model="deepgram/nova-3"),
    tts=inference.TTS(model="elevenlabs/eleven_turbo_v2_5"),
    turn_handling=TurnHandlingOptions(interruption={"resume_false_interruption": True}),
)
avatar = lemonslice.AvatarSession(agent_image_url=URL, agent_prompt="...")
await avatar.start(session, room=ctx.room)
await session.start(room=ctx.room, agent=Assistant(), room_options=RoomOptions(
    audio_input=AudioInputOptions(noise_cancellation=noise_cancellation.BVC()),
    audio_output=False,  # avatar publishes audio on behalf of agent
))
```

Noise cancellation, turn-taking, interruption recovery: all free from the framework.

---

## Design-spec deltas

### Δ1 — Use `@lemonsliceai/openclaw-avatar` for the intake demo

**Why**: installs via `openclaw plugin install @lemonsliceai/openclaw-avatar`, configured via CLI wizard (`openclaw openclaw-avatar-setup`), MIT-licensed, already production-tested. Unblocks intake demo without H100 GPUs.

**Requires**: LemonSlice API key (free-tier credits on signup appear to exist — needs user verification by signing up at `lemonslice.com/signup`), LiveKit project (LiveKit Cloud or self-hosted), ElevenLabs + Groq (or equivalent) TTS/STT keys.

**Tracks to update**: intake-coordinator checkpoint protocol should include an optional "avatar ready" check when `narrator.enabled=true`.

### Δ2 — Narrator-sidecar: swap raw aiortc for LiveKit participant

**Current plan** (pre-deep-dive): `infra/images/narrator-sidecar-base/app/webrtc.py` rolls its own aiortc peer + signaling + audio muxing (~500 LOC planned).

**Revised plan**: sidecar joins a LiveKit room as a bot participant (like LemonSlice's bot does). Browser `AvatarTile` joins the same room via `livekit-client`.

**Benefits**:
- Removes ~500 LOC of WebRTC plumbing
- Free: noise cancellation, turn detection, interruption-aware turn handling, recording, multi-participant
- Multi-backend toggle (MuseTalk ⇄ Hunyuan) becomes "which bot publishes the video track"
- Reuses existing LiveKit infra already deployed for intake

**Co-change surface** (if adopted):
- `infra/images/narrator-sidecar-base/requirements.txt` — add `livekit-agents`, drop `aiortc`/`av`
- `infra/images/narrator-sidecar-base/app/` — replace `webrtc.py` with `livekit_room.py`
- `apps/cto-sidebar/src/SidebarProvider.ts` AvatarTile — `livekit-client` instead of `RTCPeerConnection`
- Helm `infra/charts/openclaw-agent/templates/deployment.yaml` — plumb `LIVEKIT_URL` / `LIVEKIT_API_KEY` / `LIVEKIT_API_SECRET` to sidecar
- CRD co-change (per [`CLAUDE.md`](../CLAUDE.md) co-change table):
  - `crates/controller/src/tasks/code/resources.rs` (ephemeral sidecar bootstrap)
  - `crates/controller/src/crds/coderun.rs` (`CodeRunSpec`)
  - `infra/charts/cto/crds/coderun-crd.yaml` AND `infra/charts/cto-lite/crds/coderun-crd.yaml`

**Status**: **proposed, needs rubber-duck before committing.** High-leverage but touches in-flight work.

### Δ3 — Adopt OpenClaw plugin manifest + SDK pattern for internal extensions

For any new first-party plugin/extension we build:

1. Ship an `openclaw.plugin.json` with `configSchema` + `uiHints`
2. Accept secrets via the `string | {value} | {env}` indirection
3. Route RPC via `api.registerGatewayMethod`, not ad-hoc HTTP handlers
4. Register long-lived workers via `api.registerService({ id, start, stop })`
5. Expose plugin CLI subcommands via `api.registerCli`
6. Use `api.runtime.{tts, stt, videoAvatar}.*` instead of re-implementing provider selection per plugin

Rationale: centralized provider selection + sensitivity-aware UI + automatic security-scanner compatibility.

### Δ4 — Define our multi-provider avatar plugin (follow-on)

Fork `@lemonsliceai/openclaw-avatar` → `@5dlabs/openclaw-avatar-multi` (tentative) that extends the provider enum:

```json
{
  "avatar": {
    "provider": { "enum": ["lemonslice", "musetalk", "hunyuan", "echomimic"] },
    "lemonSlice": { /* existing */ },
    "musetalk":   { "workerUrl": "nats://...", "personaId": "..." },
    "hunyuan":    { "workerUrl": "nats://...", "personaId": "..." },
    "echomimic":  { "workerUrl": "nats://...", "personaId": "..." }
  }
}
```

Runtime implements the same `AvatarSession.start(session, room)` contract as `livekit-plugins-lemonslice`, but dispatches to our self-hosted workers. Our workers are LiveKit participants (Δ2).

**Contribution path**: the upstream plugin's provider field is already an enum — cleanly-added providers could be PR'd back to `lemonsliceai/openclaw-avatar` rather than maintained as a long-lived fork.

### Δ5 — Reuse `livekit-plugins-lemonslice` as reference implementation

The Python plugin at `agents/livekit-plugins/livekit-plugins-lemonslice/` is ~350 LOC. Two files:
- `api.py` — HTTP client for `https://lemonslice.com/api/liveai/sessions`
- `avatar.py` — `AvatarSession` class: mints LiveKit bot token, hooks agent audio output to bot via `DataStreamAudioOutput` + `ATTRIBUTE_PUBLISH_ON_BEHALF`

Our `musetalk` / `hunyuan` / `echomimic` plugins copy the structure, swap the API client for NATS-based worker dispatch, keep the LiveKit-participant pattern identical.

---

## Recommended next actions

| # | Action | Owner | Status |
|---|---|---|---|
| 1 | Sign up at `lemonslice.com/signup` to verify free-tier credit balance | User | Pending (zero cost) |
| 2 | Install `@lemonsliceai/openclaw-avatar` in a local OpenClaw dev instance end-to-end | Keeper / Bolt | Pending |
| 3 | Rubber-duck Δ2 (LiveKit retrofit) before committing to the sidecar pivot | Whoever picks up narrator-sidecar | Pending |
| 4 | Author Δ4 multi-provider plugin once MuseTalk / Hunyuan workers are live | Avatar track | Blocked on GPUs |
| 5 | Cross-reference this doc from `avatar-report.md` + intake-coordinator docs | Doc maintainers | Pending |

---

## Risks & open questions (rubber-duck findings)

Recorded from a critique of Δ2 (LiveKit retrofit of narrator-sidecar). These are **unresolved** — treat as gates before committing to the pivot as core architecture (vs. demo-only).

1. **LOC-simplification claim was overstated.** The "15 LOC" figure was LemonSlice's happy-path sample only. Real implementation (session lifecycle + deployment + TURN/TLS + browser auth + floor control + observability) is closer to rewriting existing `narrator-sidecar/main.py`, `webrtc.py`, and `SidebarProvider.ts:801+` into LiveKit-shaped glue. **Honest framing:** "move complexity from app code into infra + SDK conventions" — not "greatly reduce complexity."

2. **LiveKit does NOT solve committee turn-taking.** Its turn detection is per-`AgentSession` (one agent listening to one user), not N-bots-sharing-a-floor. Our committee still needs explicit conversation topology: who hears raw user audio, who hears transcripts only, can agents hear each other's rendered voice, who owns the talking stick. Dangerous failure mode: cascaded self-interruption if every member is a first-class room participant without floor control.

3. **Self-hosted LiveKit is real infra, not a casual deploy.** Production path needs Redis, TURN with public DNS + TLS certs, UDP 50000–60000 exposed, `use_external_ip: true`, TCP 7881 signaling, and load-balancer / ingress tuning. This is a support burden for customer self-deploy (very different from app bugs). **Recommendation:** use LiveKit Cloud for demo; only productize self-host after validating the room model.

4. **Self-hosted avatar-as-LiveKit-participant is not a trivial follow-on.** LemonSlice makes it look cheap because they own audio ingestion, frame generation, RTC publication, A/V sync, and bot-token flow. Our MuseTalk/V-Express/EchoMimic path needs all of that built. LiveKit removes raw SDP plumbing but doesn't make self-hosted avatars cheap.

5. **Cold-start UX (5-60s) isn't fixed by transport.** Intake tolerates it; committee interruption/rebuttal does not. We'll need prewarmed workers, persistent sessions, seat reservation, or idle/active quality modes regardless of LiveKit.

6. **"Provider-agnostic" leaks.** Adopting `livekit-client` in browser + `livekit-agents` in workers + LiveKit env in Helm + LiveKit-specific participant semantics in provider adapters makes this a **LiveKit-based platform**, not a LiveKit-portable one. To protect Hermes/Daily/Agora future, introduce internal interfaces now: `RoomTransport`, `ParticipantPublisher`, `AudioIngress`, `InterruptBus`, `AvatarRenderer`.

### Spike requirements before committing Δ2 to core architecture

The pivot is green-lit for the intake demo path. Before making it load-bearing for the committee product, a spike must prove:

- 3–5 AI participants + 1 user in one LiveKit room
- agents do **not** STT-trigger on each other's audio
- explicit floor-control policy (moderator arbitration, interruption rules) functional
- browser can render all seats acceptably
- one non-LemonSlice backend (e.g. MuseTalk in a LiveKit agent) can publish into the same room

**Reframed decision** (verbatim from critique):
> We are trading custom WebRTC code for managed room semantics and heavier infra, because transport is not our differentiator.

If that sentence stays true after the spike, pivot is sound.

---

## Appendix — upstream source locations

For anyone needing to re-clone or verify:

```bash
gh repo clone lemonsliceai/openclaw-avatar        # MIT — the reference plugin
gh repo clone lemonsliceai/lemonslice-examples    # Apache-2.0 — 5 reference apps
gh repo clone lemonsliceai/agents                  # livekit-plugins-lemonslice lives here
gh repo clone lemonsliceai/agents-js               # TS equivalent
gh repo clone lemonsliceai/pipecat                 # Pipecat transport (alternative path)
```

Notable paths inside `openclaw-avatar`:
- `openclaw.plugin.json` (172 lines) — manifest + configSchema + uiHints
- `types/openclaw-plugin-sdk.d.ts` (151 lines) — full SDK surface
- `avatar/index.ts` (6,136 lines) — plugin entry (gateway methods, HTTP routes, CLI)
- `avatar/avatar-agent-runner.js` (1,417 lines) — the spawned LiveKit agent sidecar
- `avatar/sidecar-process-control.ts` (245 lines) — spawn/stop lifecycle
- `web/app.js` (9,564 lines) — browser-side PiP LiveKit client
- `README.md` (302 lines) — includes minimum working OpenClaw config
