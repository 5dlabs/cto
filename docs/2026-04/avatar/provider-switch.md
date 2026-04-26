# CTO Avatar Provider Switch — Implementation Guide

> **Audience:** operators and frontend/backend agents who need to switch the
> Morgan/CTO avatar between the **current `/echo-turn` production** path
> (OpenClaw Morgan text, ElevenLabs TTS, async EchoMimic MP4 on OVH AI
> Deploy) and the **reference LemonSlice / LiveKit-participant** path without
> re-reading the full architecture doc.
>
> **Scope:** this doc explains where the switch points live in the existing
> code today, what each path actually does end-to-end, and how a future
> unified `AVATAR_PROVIDER` flag should hook in. It does **not** prescribe new
> abstractions — those land via [`docs/plans/avatar-provider-failover.md`](../plans/avatar-provider-failover.md).
>
> **Companion docs:**
> - [`docs/avatar/validation.md`](validation.md) — Datadog + browser validation gate to run before any readiness test or deploy.
> - [`docs/avatar-architecture.md`](../avatar-architecture.md) — LemonSlice deep-dive + multi-provider design spec (historical/reference, not the live cutline).
> - [`avatar/docs/provider-spikes.md`](../../../avatar/docs/provider-spikes.md) — STT/TTS/LLM swaps **inside** the LiveKit path.
> - [`docs/plans/avatar-provider-failover.md`](../plans/avatar-provider-failover.md) — three-tier failover plan (LemonSlice → OVH → DO).
> - [`avatar/README.md`](README.md) — LiveKit + LemonSlice PoC quick start.

---

## TL;DR

There are two **separate, parallel avatar pipelines** in this repo today.
They are not selected by a single `AVATAR_PROVIDER` env yet — they are
selected by **which Next.js route the browser hits**:

| Path | Browser route | Provider | Status |
|---|---|---|---|
| **Current production** | `/echo-turn` | OpenClaw Morgan text → ElevenLabs TTS → EchoMimic async MP4 on OVH AI Deploy | Active |
| **Future / reference** | `/` (LiveKit room UI) | LemonSlice (hosted, LiveKit participant) | PoC, not active in production |

The production cutline is deliberately narrow:

- **Text:** OpenClaw Morgan gateway (`x-openclaw-agent-id: morgan`); no
  `OPENAI_API_KEY` is required for the public avatar path.
- **Voice:** ElevenLabs streaming TTS with in-process cache and 429
  backoff; fallback MP3 is local/degraded only, not a green production signal.
- **Video:** async EchoMimic `/animate` MP4 on the configured OVH AI Deploy
  app; the Kubernetes cluster and desktop client do not need a GPU.
- **Not user-facing:** LemonSlice, TalkingHead/3D, MuseTalk, NATS, and
  `model_q8.onnx` are not requirements for `/echo-turn` production.

To switch end-users between the two route families, deploy both UIs and link the desired
one. To swap providers **within** the LiveKit path (TTS/STT/LLM), use
`avatar/docs/provider-spikes.md`. To swap the avatar **renderer** between
EchoMimic, LemonSlice, MuseTalk, etc., the work tracked in
[`avatar-provider-failover.md`](../plans/avatar-provider-failover.md) is what
will eventually expose a single `avatar.provider` enum.

---

## 1. Current path — `/echo-turn` (async EchoMimic MP4 on OVH AI Deploy)

End-to-end flow:

```
Browser /echo-turn page
  → POST /api/echo-turn/chat   (Next.js)  → OpenClaw gateway (Morgan persona, SSE)
  → POST /api/echo-turn/tts    (Next.js)  → ElevenLabs streaming TTS (MP3, cache/backoff)
  → POST /api/echo-turn/avatar (Next.js)  → EchoMimic FastAPI /animate on OVH AI Deploy
  ← MP4 played in <video>, audio already playing while video renders.
```

### 1.1 Files (do not touch — owned by another agent)

| File | Purpose |
|---|---|
| `avatar/web/app/echo-turn/page.tsx` | UI: textarea, run-turn button, three result panes. |
| `avatar/web/app/api/echo-turn/chat/route.ts` | Streams Morgan reply via OpenClaw `/v1/chat/completions`. |
| `avatar/web/app/api/echo-turn/tts/route.ts` | Streams ElevenLabs audio, caches repeat turns, and backs off on rate limits (or fallback MP3 for local/degraded runs). |
| `avatar/web/app/api/echo-turn/avatar/route.ts` | Posts source image + audio to EchoMimic `/animate`. |

These are the **echo-turn implementation files**. Changes to them are owned
by the EchoMimic implementation track; this doc only describes the public
env contract they consume.

### 1.2 Environment switch points

Set in `avatar/web/.env.local` (next.js server-only):

| Variable | Required? | Effect |
|---|---|---|
| `ECHOMIMIC_APP_URL` | **Required** for real renders | Base URL for the EchoMimic FastAPI `/animate` endpoint on OVH AI Deploy (e.g. `https://your-echomimic-app.app.gra.ai.cloud.ovh.net`). If unset, `/api/echo-turn/avatar` returns 500. |
| `MORGAN_GATEWAY_URL` or `MORGAN_LLM_BASE_URL` | Required for production text | OpenClaw gateway base. If unset, `/api/echo-turn/chat` uses a deterministic streamed fallback reply (local/degraded only). |
| `MORGAN_GATEWAY_TOKEN` or `OPENCLAW_TOKEN` | Optional | Bearer token for the OpenClaw gateway. |
| `MORGAN_LLM_AGENT_ID` | Optional | Sets `x-openclaw-agent-id` header (use `morgan` against the shared gateway at `morgan.5dlabs.ai`). |
| `MORGAN_MODEL` | Optional | Defaults to `openclaw/morgan`. |
| `ELEVENLABS_API_KEY` | Required for production voice | If unset, `/api/echo-turn/tts` falls back to `voice_clone_sample.mp3` and marks the response with `X-Morgan-TTS-Fallback`. |
| `MORGAN_VOICE_ID` | Optional | ElevenLabs voice id. Default `iP95p4xoKVk53GoZ742B`. |
| `MORGAN_DEMO_FORCE_STUB=1` | Optional | Forces both chat and TTS to deterministic fallbacks regardless of other envs (useful for local UI dev with no keys). |

The source image is read from `avatar/morgan.jpg` (resolved relative to the
Next.js working directory at `../morgan.jpg`). To rebrand the avatar source,
replace that file — no env required.

### 1.2.1 Production cutline validation

The public `/echo-turn` cutline is the web route plus the configured OVH AI
Deploy EchoMimic app. Historical `morgan-avatar-agent`/LiveKit deployment
settings can still be useful for agent validation, but they are not
user-facing prerequisites for the production avatar/desktop path. Never copy
secret values into docs, logs, or PR comments.

| Capability | Env names | Production source |
|---|---|---|
| OpenClaw Morgan text | `MORGAN_GATEWAY_URL` or `MORGAN_LLM_BASE_URL`, `OPENCLAW_TOKEN`, `MORGAN_LLM_AGENT_ID=morgan` | Non-empty production gateway/token values; no OpenAI key |
| ElevenLabs TTS | `ELEVENLABS_API_KEY`, `MORGAN_VOICE_ID` | API key present for production; route emits `X-Morgan-TTS-Cache` or `X-Morgan-TTS-Fallback` |
| EchoMimic OVH renderer | `ECHOMIMIC_APP_URL` plus per-request `video_length`, `sample_height`, `sample_width`, `weight_dtype` | Non-empty OVH AI Deploy `/animate` base URL |
| Forbidden production prerequisites | `OPENAI*`, NATS URL/client env, Kubernetes GPU scheduling, desktop GPU, `model_q8.onnx` | Must not be required for public `/echo-turn` readiness |

Safe validation pattern (prints only source type, secret/key names, and
presence when you are checking the legacy agent deployment):

```bash
kubectl config use-context ovh-cluster
kubectl -n cto get deploy morgan-avatar-agent -o json \
  | jq -r '.spec.template.spec.containers[].env[]?
      | if has("valueFrom") then
          "ENV \(.name) secretRef \(.valueFrom.secretKeyRef.name) \(.valueFrom.secretKeyRef.key)"
        else
          "ENV \(.name) literal \(if (.value | length) > 0 then "non-empty" else "empty" end)"
        end'
kubectl -n cto get secret livekit-keys openclaw-api-keys -o json \
  | jq -r '.items[] | .metadata.name as $secret
      | .data | keys[] as $key
      | "SECRET \($secret) KEY \($key) present-\(if .[$key] | length > 0 then "non-empty" else "empty" end)"'
```

Render-time tuning is sent **per request** from the browser as form fields
(`prompt`, `video_length`, `sample_height`, `sample_width`, `weight_dtype`)
and forwarded verbatim by `/api/echo-turn/avatar`. The defaults baked into
`page.tsx` (512×512, `video_length=32`, `weight_dtype=float16`, golden
retriever prompt) are tuned for the current OVH AI Deploy worker and should
not be changed without coordinating with the EchoMimic owner. Do not add
cluster GPU, desktop GPU, NATS, OpenAI, or `model_q8.onnx` prerequisites to
this path.

### 1.3 Operational behavior

- **Cold path:** latest public `/echo-turn` E2E returned `video/mp4` after
  ~215s total, with ~205s spent upstream in EchoMimic. Audio plays as soon
  as TTS resolves; video appears when the MP4 finishes polling through
  `/api/echo-turn/avatar`.
- **Failure modes:** EchoMimic 502 surfaces as `EchoMimic render failed` in
  the UI. OpenClaw failure silently falls through to the canned reply.
- **No interruption:** this is a turn-by-turn flow. The user must wait for
  the MP4 to finish before sending another message.
- **Observability:** the response includes `X-EchoMimic-Elapsed-S` and
  `X-EchoMimic-Job-Id` headers passed through from the FastAPI worker.
- **Clean worker expectation:** the latest avatar worker rollout was clean
  and did **not** require `model_q8.onnx`; treat any doc/runbook that asks
  for that file as stale for the production cutline.
- **Async job state is in-process only.** The async render path
  (`POST /api/echo-turn/avatar` returning `{ jobId }` plus
  `GET /api/echo-turn/avatar?jobId=…` for polling) keeps job state in an
  in-memory `Map` inside the Next.js process
  (`avatar/web/app/api/echo-turn/avatar/route.ts`). It is **not** durable
  and is **not** shared across replicas. See §1.4 for the operational
  constraint this imposes for the 3-day production bridge.

### 1.4 Operational constraint — single-replica, no rolling restart

The async job store described in §1.3 is intentionally in-process for the
3-day production bridge. To run this safely until a durable store lands,
the EchoMimic Next.js deployment **must** operate under these constraints:

1. **Exactly one replica**, or sticky-session routing pinning a given
   `jobId` to the replica that created it. Any replica without the
   originating `Map` entry will return `unknown job` for that id.
2. **No rolling restart, redeploy, or pod eviction during active
   sessions.** Process restart wipes the `Map`; in-flight and
   recently-completed jobs become unrecoverable. Drain or schedule
   restarts in a maintenance window with no live readiness traffic.
3. **No horizontal autoscaling on this deployment** until the durable
   store follow-up lands. HPA / replica bumps will silently break
   polling for jobs whose owning pod they do not hit.

**User-visible behavior on restart / replica miss.** The polling endpoint
returns an `unknown job` response for any `jobId` whose owning process
has restarted or whose request lands on a different replica. The
`/echo-turn` UI must degrade gracefully in that case: the audio leg
(ElevenLabs / fallback MP3) has already played independently of the
avatar render, so the page should surface "audio available, video
unavailable — please retry" rather than spinning indefinitely. Treat an
`unknown job` from the poller as a terminal error for that turn, not a
retryable transient.

**Follow-up before scaling beyond one replica.** Replace the in-memory
`Map` with a durable, shared job store before enabling multi-replica or
rolling restarts. Acceptable shapes (in priority order):

- **Redis** (or compatible) keyed by `jobId`, with the same TTL semantics
  the `Map` already enforces (`JOB_TTL_MS`, `MAX_JOBS`).
- **Persistent volume** (PV) backed file/sqlite store if Redis is not
  available in the target environment.
- **EchoMimic-side job ownership** — push job lifecycle into the
  EchoMimic FastAPI worker so the Next.js layer becomes a stateless
  proxy for `GET /jobs/<id>`.

Until one of those lands, the constraints above are load-bearing. Do not
remove them by raising `replicas` in Helm values without first wiring a
durable store and deleting this section.

### 1.5 Desktop / CTO Lite integration contract (Pixel)

CTO Lite should consume the same product APIs as `/echo-turn`; it should not
link directly against the Morgan agent mesh, EchoMimic worker, ElevenLabs, or
NATS. The desktop app may call these APIs from the webview with `fetch` or via
a thin Tauri command proxy, but the wire contract stays HTTP/SSE plus media
URLs/blobs.

#### 1.5.1 Desktop host configuration

| Setting | Scope | Default / guidance |
|---|---|---|
| `VITE_MORGAN_ECHO_TURN_BASE_URL` | UI build/runtime config | Optional base URL for browser `fetch`. If unset, use same-origin (`""`) when the avatar web app is embedded, or the production product origin when packaged. Do not put server secrets in `VITE_*`. |
| `MORGAN_ECHO_TURN_BASE_URL` | Optional Tauri/Rust proxy config | Same base URL if CTO Lite proxies requests through Tauri. Store only product/user auth material in the OS keychain; do not store ElevenLabs, EchoMimic, OpenClaw, or NATS secrets in desktop. |
| Product auth | HTTP only | Use the product's existing cookie/session/bearer mechanism for the API origin. Production desktop traffic must use HTTPS. |

The desktop client must not require `OPENAI_API_KEY`, NATS credentials, a
Kubernetes/desktop GPU, LiveKit/LemonSlice credentials, or direct
`ECHOMIMIC_APP_URL` access for this EchoMimic cutline.

#### 1.5.2 Turn flow and state machine

Recommended UI states:

```
idle
  → streamingText
  → synthesizingAudio
  → renderingVideo   (audio is already playable)
  → ready
  ↘ error / videoUnavailable
```

`chat` failure stops the turn before TTS. `tts` failure leaves the text reply
visible. `avatar` failure leaves text + audio usable and marks only the video
leg unavailable.

#### 1.5.3 `POST /api/echo-turn/chat`

Request:

```http
POST {base}/api/echo-turn/chat
Accept: text/event-stream
Content-Type: application/json
```

```json
{
  "message": "Give me a short update on the avatar path."
}
```

Only `message` is required by the current route. Desktop may track its own
`turnId`, `sessionId`, `projectId`, or `agentId` locally, but must not depend
on the route echoing those fields until the API adds them.

Response:

```http
200 OK
Content-Type: text/event-stream; charset=utf-8
Cache-Control: no-store
```

SSE frames are `data:` lines containing one JSON object:

```ts
type EchoTurnChatEvent =
  | { type: "delta"; text: string }
  | { type: "done" }
  | { type: "error"; message: string; code?: string; recoverable?: boolean };
```

Because this is a `POST`, use `fetch` + `ReadableStream` parsing rather than
`EventSource`. Accumulate `delta.text` into `finalReply`. Treat `error` frames
as terminal for the turn and do not call TTS unless a future route explicitly
marks the event recoverable.

Timeout guidance:

- 10s connect / first-byte warning in UI.
- 120s total stream timeout, matching the server's OpenClaw gateway timeout.
- Abort when the user cancels or starts a new turn.

#### 1.5.4 `POST /api/echo-turn/tts`

Request:

```http
POST {base}/api/echo-turn/tts
Accept: audio/mpeg
Content-Type: application/json
```

```json
{ "text": "Morgan's final text reply." }
```

Responses:

| Status | Body | Notes |
|---|---|---|
| `200` | `audio/mpeg` (or upstream audio content type) | Build an object URL and play immediately. Revoke the object URL on turn reset/unmount. |
| `400` | `{ "error": "text is required" }` | Terminal client error. |

Headers to surface in diagnostics:

| Header | Meaning |
|---|---|
| `X-Morgan-TTS-Cache: hit \| miss` | Repeat text may reuse cached audio. Cache is server-side; desktop should not implement its own dedupe first. |
| `X-Morgan-TTS-Fallback` | A local/degraded fallback MP3 was used. Play it, but mark the voice leg degraded. |
| `X-Morgan-TTS-Fallback-Reason` | `missing-api-key`, `forced-stub`, `rate-limited`, `rate-limit-backoff`, or `upstream-<status>`. |
| `Retry-After` | Present during ElevenLabs backoff; use it only for copy/diagnostics, not aggressive client retry. |

Timeout guidance: 60s hard timeout, 75s UI grace max. If TTS fails, show text
and offer "retry voice"; do not submit an avatar render without audio.

#### 1.5.5 `POST /api/echo-turn/avatar`

Request:

```http
POST {base}/api/echo-turn/avatar
Content-Type: multipart/form-data
```

Form fields:

| Field | Required? | Value |
|---|---|---|
| `audio` | **Yes** | MP3 blob from `/tts`, filename like `morgan-turn.mp3`. |
| `prompt` | Optional | Defaults to the Morgan golden-retriever prompt in the route. |
| `video_length` | Optional | Current UI default: `32`. |
| `sample_height` | Optional | Current UI default: `512`. |
| `sample_width` | Optional | Current UI default: `512`. |
| `weight_dtype` | Optional | Current UI default: `float16`. |

Success:

```http
202 Accepted
Content-Type: application/json; charset=utf-8
```

```json
{ "jobId": "uuid", "status": "queued" }
```

Submit timeout guidance: 15s for the `202` response. The expensive render runs
asynchronously after the job is accepted.

#### 1.5.6 `GET /api/echo-turn/avatar?jobId=...`

Poll with:

```http
GET {base}/api/echo-turn/avatar?jobId={encodeURIComponent(jobId)}
Cache-Control: no-store
```

Responses:

| Status | Body | Desktop behavior |
|---|---|---|
| `202` | `{ jobId, status: "queued" \| "running", createdAt, startedAt? }` | Continue polling. |
| `200` | `video/mp4` (or returned video content type) | Create/revoke object URL, set `<video controls playsInline autoplay>`, keep the audio player available. |
| `400` | `{ error: "jobId query parameter is required" }` | Client bug; terminal. |
| `404` | `{ error: "unknown jobId", jobId }` | Terminal. Usually process restart/replica miss; show audio-only fallback and allow a fresh render. |
| `410` | `{ error: "EchoMimic artifact is no longer available", jobId, status: "expired" }` | Terminal expired artifact; offer rerender from cached audio if still available. |
| `502` | `{ error, status, jobStatus, failureStage, detail, createdAt, finishedAt, jobId }` | Terminal render failure; preserve text/audio. |

Headers on `200`:

| Header | Meaning |
|---|---|
| `X-EchoMimic-Elapsed-S` | Upstream render duration. |
| `X-EchoMimic-Job-Id` | Upstream EchoMimic job id, if provided. |
| `X-EchoMimic-Local-Job-Id` | Next.js job id; match it to the desktop turn. |

Polling guidance:

- Poll every 3s with `cache: "no-store"`.
- Display elapsed time after the first poll; current cold renders are commonly
  ~3.5 minutes.
- Stop at 12 minutes in the UI. Server render timeout is 15 minutes, but users
  should get a clear "video unavailable" state before waiting indefinitely.
- Treat `unknown jobId` as terminal, not retryable.

#### 1.5.7 User-facing error copy

Use leg-specific copy so a video outage does not make the whole Morgan turn
look broken:

| Failure | Suggested copy |
|---|---|
| Chat HTTP/SSE failure | "Morgan could not start this text turn. Check your connection and try again." |
| Chat emits `error` | `Morgan stopped the text turn: {message}` |
| TTS failure | "Morgan replied in text, but voice synthesis failed. You can retry voice for this turn." |
| TTS fallback header | "Using a temporary fallback voice. Morgan's text is still valid." |
| Avatar submit failure | "Morgan's voice is ready, but the avatar render could not start." |
| Avatar poll timeout | "Morgan's voice is ready, but the avatar video took too long. You can retry the video render." |
| Avatar `404` / `410` | "Morgan's voice is ready, but this video job is no longer available. Retry the avatar render." |
| Avatar `502` | "Morgan's voice is ready, but EchoMimic could not render this turn." |

#### 1.5.8 Desktop storage, cache, and privacy rules

- Keep reply text in the existing Morgan session state.
- Keep audio/video as revocable object URLs by default; persist MP3/MP4 only
  after an explicit user action.
- Revoke old object URLs when a turn resets, a new project is selected, or the
  component unmounts.
- Do not log raw user prompts, audio blobs, or MP4 bytes to app logs by
  default. Log `turnId`, phase timings, status codes, cache headers, and job
  ids instead.
- Do not upload or scan local files for this contract; the API owns the Morgan
  source image and EchoMimic source parameters.

#### 1.5.9 Current CTO Lite hook points and follow-up implementation todos

Inspected desktop surfaces:

| Surface | Current behavior | Required Pixel change |
|---|---|---|
| `crates/cto-lite/ui/src/App.tsx` | Shared Chat/Call/Video session shell. | Preserve this shell and add echo-turn media state to the shared Morgan session. |
| `crates/cto-lite/ui/src/components/AgentChat/index.tsx` | Text uses Tauri `openclaw_send_message`. | Either keep for local ACP chat or switch Morgan production chat to `/api/echo-turn/chat` for parity with avatar turns. |
| `crates/cto-lite/ui/src/components/AgentsView/index.tsx` | Video tab embeds `MorganAvatarRoom` LiveKit token flow. | Replace production Video tab with the echo-turn HTTP/SSE + MP4 flow. Keep LiveKit only as future realtime/reference. |
| `crates/cto-lite/ui/src/components/VoiceView/index.tsx` | Call tab also embeds `MorganAvatarRoom` in voice mode. | Do not make Call/Video depend on NATS. Future realtime should use LiveKit/WebRTC/WebSocket media APIs. |
| `crates/cto-lite/ui/src/components/MorganAvatarRoom/index.tsx` | LiveKit token, audio track, optional video track. | Leave as reference/future realtime component; do not use for EchoMimic batch MP4 unless renamed/wrapped to avoid transport confusion. |
| `crates/cto-lite/tauri/src/commands/openclaw.rs` | Local ACP/OpenClaw bridge, health, memory, and NATS readiness diagnostics. | Do not add a NATS client for avatar. Add an HTTP product API client only if desktop needs a Tauri proxy. |

Implementation-ready todos:

1. `desktop-echo-turn-client` — add `crates/cto-lite/ui/src/lib/morgan-echo-turn.ts`
   with `streamChat`, `synthesizeTts`, `submitAvatar`, and `pollAvatarJob`
   helpers implementing §§1.5.3–1.5.6.
2. `desktop-echo-turn-video-view` — replace the production `AgentsView` video
   body with an EchoMimic turn component that streams text, plays audio as soon
   as it is ready, polls MP4, and preserves audio-only fallback on render
   failure.
3. `desktop-echo-turn-session-state` — extend `MorganSessionState` with
   `turnPhase`, `turnId`, `audioCacheStatus`, `ttsFallbackReason`,
   `avatarJobId`, `videoStatus`, timing metrics, and terminal error detail.
4. `desktop-echo-turn-config` — add `VITE_MORGAN_ECHO_TURN_BASE_URL`
   (and optional `MORGAN_ECHO_TURN_BASE_URL` for a Rust proxy) with no server
   secrets in desktop env.
5. `desktop-avatar-readiness-health` — split avatar readiness from local
   cluster/NATS health; the production EchoMimic path is ready when the product
   HTTP API is reachable and authenticated, not when a local NATS deployment is
   present.

Future realtime media remains separate: use LiveKit/WebRTC/WebSocket media APIs
when the product needs interruption-aware duplex media. Do not use NATS as a
desktop-facing media or avatar transport.

### 1.6 What "switching off EchoMimic" looks like today

There is **no kill switch**. To take EchoMimic offline:

1. Unset `ECHOMIMIC_APP_URL` (or scale the OVH AI Deploy EchoMimic worker to 0). The UI
   will then return 500 from `/api/echo-turn/avatar` while still streaming
   text + voice — useful as a partial-degradation mode.
2. Or remove the link to `/echo-turn` from the public site and route users
   to the LiveKit PoC at `/` instead only for an explicitly scoped
   non-production demo.

---

## 2. Future / reference path — LemonSlice on LiveKit

End-to-end flow:

```
Browser /  (Room.tsx, livekit-client)
  → POST /api/token  (Next.js)  → LiveKit room created + agent dispatched
  → joins LiveKit room
  ← agent (Python, avatar/agent/agent.py) joins same room as participant
       avatar (LemonSlice) joins same room as bot, publishes lip-synced video
  ↔ full-duplex WebRTC audio + video, interruption-aware turn handling
```

This is the path the deep-dive in
[`docs/avatar-architecture.md`](../avatar-architecture.md) documents. It is
**not** the active production path; do not assume LemonSlice is in the live
hot-path until a deployment task explicitly flips it.

### 2.1 Files

| File | Purpose |
|---|---|
| `avatar/web/app/page.tsx` + `avatar/web/components/Room.tsx` | LiveKit room UI. |
| `avatar/web/app/api/token/route.ts` | Mints LiveKit access tokens, ensures room exists, dispatches `morgan-avatar` agent. |
| `avatar/agent/morgan_avatar_agent/` | Python LiveKit agent (joins room, runs STT/LLM/TTS, drives LemonSlice avatar). |
| `avatar/agent/morgan_avatar_agent/providers.py` + `config.py` | Resolves `MORGAN_STT_MODE` / `MORGAN_TTS_MODE` / `MORGAN_LLM_BACKEND`. |
| `infra/gitops/agents/morgan-avatar-values.yaml` | Helm values for the in-cluster Morgan avatar deployment. |

### 2.2 Environment switch points

Server-side (Next.js, `avatar/web/.env.local`):

| Variable | Required? | Effect |
|---|---|---|
| `LIVEKIT_URL` | **Required** | wss URL (e.g. `wss://lk.5dlabs.ai`). |
| `LIVEKIT_API_KEY` | **Required** | Server-side only. |
| `LIVEKIT_API_SECRET` | **Required** | Server-side only. |

Python agent (`avatar/agent/.env`):

| Variable | Required? | Effect |
|---|---|---|
| `LIVEKIT_URL`, `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET` | **Required** | Same as above; agent joins the same room. |
| `MORGAN_LEMONSLICE_AGENT_ID` *or* `MORGAN_IMAGE_URL` | **Required** | Selects the LemonSlice avatar persona. |
| `MORGAN_LLM_BACKEND` | Optional | `openclaw` (default) or `inference`. See `provider-spikes.md`. |
| `MORGAN_LLM_BASE_URL`, `MORGAN_LLM_API_KEY` (or `OPENCLAW_TOKEN`) | Required when backend=`openclaw` | Routes the reasoning hop. |
| `MORGAN_STT_MODE` / `MORGAN_TTS_MODE` | Optional | Swaps STT/TTS providers. See [`provider-spikes.md`](../../../avatar/docs/provider-spikes.md). |

The browser does **not** receive any LiveKit credentials directly — it only
receives a short-lived participant token from `/api/token`.

### 2.3 Operational behavior

- **Cold path:** room connect ~1–2s, first avatar audio ~5–9s.
- **Full duplex:** noise cancellation, turn detection, interruption recovery
  are provided by `livekit-agents`.
- **Self-hosted LiveKit caveats:** Redis, TURN with public DNS + TLS, UDP
  50000–60000, `use_external_ip: true`, TCP 7881 signaling. Rubber-duck
  findings in `avatar-architecture.md` §"Risks & open questions" before
  committing to self-host.

---

## 3. Hook points for a unified provider switch (future work)

When the failover work in
[`docs/plans/avatar-provider-failover.md`](../plans/avatar-provider-failover.md)
lands, a single `avatar.provider` enum will replace "which route did the
browser hit." The hook points it must integrate with are:

1. **Next.js API surface.** Today `/api/echo-turn/avatar` calls EchoMimic
   directly. The future replacement should:
   - Read a server-side `AVATAR_PROVIDER` env (`echomimic` | `lemonslice` |
     `musetalk` | `hunyuan`).
   - Dispatch to a per-provider adapter module under (suggested)
     `avatar/web/app/api/avatar/<provider>/`.
   - Keep the existing request shape (multipart `audio` + form options) so
     the `/echo-turn` UI continues to work unchanged when `AVATAR_PROVIDER`
     remains `echomimic`.
2. **Python agent (`avatar/agent/morgan_avatar_agent/providers.py`).** Add a
   `MORGAN_AVATAR_PROVIDER` parallel to `MORGAN_TTS_MODE` so the LiveKit
   path can switch between LemonSlice / MuseTalk / Hunyuan without code
   edits, mirroring the spike pattern.
3. **OpenClaw plugin manifest** (`@5dlabs/openclaw-avatar-multi`, planned).
   The `avatar.provider` enum and `string | {value} | {env}` secret
   indirection are described in `avatar-architecture.md` §Δ4. This is the
   long-term home of the switch — the Next.js env above is the bridging
   strategy until the plugin lands.
4. **Helm / Argo values.** `infra/gitops/agents/morgan-avatar-values.yaml`
   is the canonical Helm input for the in-cluster Morgan worker. New
   provider envs must be plumbed there, **and** mirrored in the co-change
   targets listed in [`AGENTS.md`](../../../AGENTS.md) §"Co-change requirements"
   when ephemeral CodeRun sidecars need them.

Until those hook points exist, treat "provider switch" as **a deployment
choice, not a runtime toggle**.

---

## 4. Operational guidance — picking a path

| Situation | Use | Why |
|---|---|---|
| Investor / intake demo today | EchoMimic `/echo-turn` | Active; deterministic; no live LiveKit infra to babysit. |
| Latency / interruption spike | LemonSlice `/` | Full-duplex room model is what we measure for committee work. |
| Customer-facing production avatar | EchoMimic `/echo-turn` | No LemonSlice, no user-facing NATS, no OpenAI key, and no Kubernetes/desktop GPU requirement; GPU work is isolated to OVH AI Deploy. |
| Multi-agent committee floor | Neither, yet | Spike requirements in `avatar-architecture.md` §"Spike requirements before committing Δ2" must pass first. |

### 4.1 Roll-back

- If LemonSlice ever becomes the default and we need to fall back: redeploy
  the avatar web app with the EchoMimic envs from §1.2 set, and route
  traffic to `/echo-turn`. EchoMimic is intentionally kept hot in
  `avatar/echomimic/` so it can be re-enabled without a cold rebuild.
- If EchoMimic goes down mid-demo: unset `ECHOMIMIC_APP_URL` to surface a
  clean 500 from `/api/echo-turn/avatar` (text + voice still work). Do not
  route production traffic to the LiveKit PoC unless that is the explicit
  demo scope.

### 4.2 What **not** to do

- Do **not** add cross-cutting `if (provider === "lemonslice")` branches
  inside `avatar/web/app/api/echo-turn/*` — those routes are scoped to
  EchoMimic and owned by another agent. New providers belong in new
  routes / a new top-level `avatar/web/app/api/avatar/` namespace per §3.1.
- Do **not** point the EchoMimic UI at a LemonSlice endpoint — the request
  shapes are incompatible (multipart batch vs. LiveKit participant).
- Do **not** plumb LemonSlice / LiveKit secrets into the EchoMimic
  deployment (and vice-versa). Keep the env surfaces in §1.2 and §2.2
  separate per deployment until the unified plugin in §3 lands.

---

## 5. Quick reference — env summary

```bash
# === Current production: EchoMimic ===
ECHOMIMIC_APP_URL=https://...                # required
MORGAN_GATEWAY_URL=https://morgan.5dlabs.ai  # required for production text
MORGAN_LLM_AGENT_ID=morgan
OPENCLAW_TOKEN=...
MORGAN_MODEL=openclaw/morgan
ELEVENLABS_API_KEY=...                       # required for production voice
MORGAN_VOICE_ID=iP95p4xoKVk53GoZ742B
# MORGAN_DEMO_FORCE_STUB=1                   # local UI dev w/o keys
# No OPENAI_API_KEY, NATS URL, Kubernetes GPU, or model_q8.onnx required.

# === Future / reference: LemonSlice on LiveKit ===
LIVEKIT_URL=wss://lk.5dlabs.ai               # required
LIVEKIT_API_KEY=...                          # required
LIVEKIT_API_SECRET=...                       # required

# Python agent (avatar/agent/.env)
MORGAN_LEMONSLICE_AGENT_ID=...               # or MORGAN_IMAGE_URL=
MORGAN_LLM_BACKEND=openclaw                  # see provider-spikes.md
MORGAN_LLM_BASE_URL=https://morgan.5dlabs.ai
MORGAN_LLM_API_KEY=...                       # or OPENCLAW_TOKEN
MORGAN_STT_MODE=livekit-flux
MORGAN_TTS_MODE=livekit-elevenlabs
```
