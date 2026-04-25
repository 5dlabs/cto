# CTO Avatar Provider Switch — Implementation Guide

> **Audience:** operators and frontend/backend agents who need to switch the
> Morgan/CTO avatar between the **current EchoMimic self-hosted** path and the
> **future LemonSlice / LiveKit-participant** path without re-reading the full
> architecture doc.
>
> **Scope:** this doc explains where the switch points live in the existing
> code today, what each path actually does end-to-end, and how a future
> unified `AVATAR_PROVIDER` flag should hook in. It does **not** prescribe new
> abstractions — those land via [`docs/plans/avatar-provider-failover.md`](../plans/avatar-provider-failover.md).
>
> **Companion docs:**
> - [`docs/avatar/validation.md`](validation.md) — Datadog + browser validation gate to run before any readiness test or deploy.
> - [`docs/avatar-architecture.md`](../avatar-architecture.md) — LemonSlice deep-dive + multi-provider design spec.
> - [`avatar/docs/provider-spikes.md`](../../avatar/docs/provider-spikes.md) — STT/TTS/LLM swaps **inside** the LiveKit path.
> - [`docs/plans/avatar-provider-failover.md`](../plans/avatar-provider-failover.md) — three-tier failover plan (LemonSlice → OVH → DO).
> - [`avatar/README.md`](../../avatar/README.md) — LiveKit + LemonSlice PoC quick start.

---

## TL;DR

There are two **separate, parallel avatar pipelines** in this repo today.
They are not selected by a single `AVATAR_PROVIDER` env yet — they are
selected by **which Next.js route the browser hits**:

| Path | Browser route | Provider | Status |
|---|---|---|---|
| **Current production** | `/echo-turn` | EchoMimic (self-hosted, batch MP4) | Active |
| **Future / reference** | `/` (LiveKit room UI) | LemonSlice (hosted, LiveKit participant) | PoC, not active in production |

To switch end-users between the two, deploy both UIs and link the desired
one. To swap providers **within** the LiveKit path (TTS/STT/LLM), use
`avatar/docs/provider-spikes.md`. To swap the avatar **renderer** between
EchoMimic, LemonSlice, MuseTalk, etc., the work tracked in
[`avatar-provider-failover.md`](../plans/avatar-provider-failover.md) is what
will eventually expose a single `avatar.provider` enum.

---

## 1. Current path — EchoMimic (self-hosted, batch render)

End-to-end flow:

```
Browser /echo-turn page
  → POST /api/echo-turn/chat   (Next.js)  → OpenClaw gateway (Morgan persona, SSE)
  → POST /api/echo-turn/tts    (Next.js)  → ElevenLabs streaming TTS (MP3)
  → POST /api/echo-turn/avatar (Next.js)  → EchoMimic FastAPI /animate (returns MP4)
  ← MP4 played in <video>, audio already playing while video renders.
```

### 1.1 Files (do not touch — owned by another agent)

| File | Purpose |
|---|---|
| `avatar/web/app/echo-turn/page.tsx` | UI: textarea, run-turn button, three result panes. |
| `avatar/web/app/api/echo-turn/chat/route.ts` | Streams Morgan reply via OpenClaw `/v1/chat/completions`. |
| `avatar/web/app/api/echo-turn/tts/route.ts` | Streams ElevenLabs audio (or fallback MP3). |
| `avatar/web/app/api/echo-turn/avatar/route.ts` | Posts source image + audio to EchoMimic `/animate`. |

These are the **echo-turn implementation files**. Changes to them are owned
by the EchoMimic implementation track; this doc only describes the public
env contract they consume.

### 1.2 Environment switch points

Set in `avatar/web/.env.local` (next.js server-only):

| Variable | Required? | Effect |
|---|---|---|
| `ECHOMIMIC_APP_URL` | **Required** for real renders | Base URL for the EchoMimic FastAPI `/animate` endpoint (e.g. `https://your-echomimic-app.app.gra.ai.cloud.ovh.net`). If unset, `/api/echo-turn/avatar` returns 500. |
| `MORGAN_GATEWAY_URL` or `MORGAN_LLM_BASE_URL` | Optional | OpenClaw gateway base. If unset, `/api/echo-turn/chat` uses a deterministic streamed fallback reply (UI still works). |
| `MORGAN_GATEWAY_TOKEN` or `OPENCLAW_TOKEN` | Optional | Bearer token for the OpenClaw gateway. |
| `MORGAN_LLM_AGENT_ID` | Optional | Sets `x-openclaw-agent-id` header (use `morgan` against the shared gateway at `morgan.5dlabs.ai`). |
| `MORGAN_MODEL` | Optional | Defaults to `openclaw/morgan`. |
| `ELEVENLABS_API_KEY` | Optional | If unset, `/api/echo-turn/tts` falls back to `voice_clone_sample.mp3`. |
| `MORGAN_VOICE_ID` | Optional | ElevenLabs voice id. Default `iP95p4xoKVk53GoZ742B`. |
| `MORGAN_DEMO_FORCE_STUB=1` | Optional | Forces both chat and TTS to deterministic fallbacks regardless of other envs (useful for local UI dev with no keys). |

The source image is read from `avatar/morgan.jpg` (resolved relative to the
Next.js working directory at `../morgan.jpg`). To rebrand the avatar source,
replace that file — no env required.

Render-time tuning is sent **per request** from the browser as form fields
(`prompt`, `video_length`, `sample_height`, `sample_width`, `weight_dtype`)
and forwarded verbatim by `/api/echo-turn/avatar`. The defaults baked into
`page.tsx` (512×512, `video_length=32`, `weight_dtype=float16`, golden
retriever prompt) are tuned for the V100 baseline and should not be changed
without coordinating with the EchoMimic owner.

### 1.3 Operational behavior

- **Cold path:** EchoMimic batch render takes ~3.5 minutes on the current
  V100. Audio plays as soon as TTS resolves; video appears when the MP4
  finishes streaming back through `/api/echo-turn/avatar`.
- **Failure modes:** EchoMimic 502 surfaces as `EchoMimic render failed` in
  the UI. OpenClaw failure silently falls through to the canned reply.
- **No interruption:** this is a turn-by-turn flow. The user must wait for
  the MP4 to finish before sending another message.
- **Observability:** the response includes `X-EchoMimic-Elapsed-S` and
  `X-EchoMimic-Job-Id` headers passed through from the FastAPI worker.
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

### 1.5 What "switching off EchoMimic" looks like today

There is **no kill switch**. To take EchoMimic offline:

1. Unset `ECHOMIMIC_APP_URL` (or scale the EchoMimic worker to 0). The UI
   will then return 500 from `/api/echo-turn/avatar` while still streaming
   text + voice — useful as a partial-degradation mode.
2. Or remove the link to `/echo-turn` from the public site and route users
   to the LiveKit PoC at `/` instead.

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
| `MORGAN_STT_MODE` / `MORGAN_TTS_MODE` | Optional | Swaps STT/TTS providers. See [`provider-spikes.md`](../../avatar/docs/provider-spikes.md). |

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
   targets listed in [`AGENTS.md`](../../AGENTS.md) §"Co-change requirements"
   when ephemeral CodeRun sidecars need them.

Until those hook points exist, treat "provider switch" as **a deployment
choice, not a runtime toggle**.

---

## 4. Operational guidance — picking a path

| Situation | Use | Why |
|---|---|---|
| Investor / intake demo today | EchoMimic `/echo-turn` | Active; deterministic; no live LiveKit infra to babysit. |
| Latency / interruption spike | LemonSlice `/` | Full-duplex room model is what we measure for committee work. |
| Customer-facing self-hosted | EchoMimic | No LemonSlice subscription dependency; runs entirely on our GPUs. |
| Multi-agent committee floor | Neither, yet | Spike requirements in `avatar-architecture.md` §"Spike requirements before committing Δ2" must pass first. |

### 4.1 Roll-back

- If LemonSlice ever becomes the default and we need to fall back: redeploy
  the avatar web app with the EchoMimic envs from §1.2 set, and route
  traffic to `/echo-turn`. EchoMimic is intentionally kept hot in
  `avatar/echomimic/` so it can be re-enabled without a cold rebuild.
- If EchoMimic goes down mid-demo: unset `ECHOMIMIC_APP_URL` to surface a
  clean 500 from `/api/echo-turn/avatar` (text + voice still work), or
  redirect `/echo-turn` → `/` to fall through to the LiveKit PoC.

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
MORGAN_GATEWAY_URL=https://morgan.5dlabs.ai  # optional (else fallback reply)
MORGAN_LLM_AGENT_ID=morgan
OPENCLAW_TOKEN=...
MORGAN_MODEL=openclaw/morgan
ELEVENLABS_API_KEY=...                       # optional (else canned audio)
MORGAN_VOICE_ID=iP95p4xoKVk53GoZ742B
# MORGAN_DEMO_FORCE_STUB=1                   # local UI dev w/o keys

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
