# Phase 4 Disposition (WS-F)

**Author:** Morgan (intake) · **Branch:** `docs/ws-f-phase4-disposition` · **Related PR:** #4790 (merged)

> **Current production cutline (supersedes this historical disposition for
> live readiness):** public avatar traffic uses `/echo-turn`:
> OpenClaw Morgan text → ElevenLabs TTS with cache/backoff → async EchoMimic
> MP4 on OVH AI Deploy. LemonSlice, TalkingHead/3D, MuseTalk, NATS,
> Kubernetes/desktop GPU scheduling, OpenAI keys, and `model_q8.onnx` are not
> live production prerequisites. Keep this memo as WS-F archaeology only.

> **Note on the requested path.** WS-F pointed at `docs/kimi-k2-6-self-hosting/` for
> "Phase 4 session protocol" content. That path resolves to a single file
> (`docs/kimi-k2-6-self-hosting.md`) about Kimi K2.6 GPU/cost sizing — **no Phase 4
> session-protocol material.** The actual Phase-4 artefacts in the repo are:
>
> - `docs/specs/avatar-session-protocol.md` — **"Avatar Session Protocol (Phase 4)"** — the
>   agent ↔ avatar ↔ host wire protocol.
> - `docs/prds/coder-gpu-musetalk-phase4.md` — the **"Phase 4 GPU + MuseTalk"** PRD (later
>   scope-expanded to cover Phases 1–6 of a fully self-hosted avatar pipeline).
>
> Two different docs, both labelled "Phase 4", with very different subjects. This memo
> treats them separately.

## TL;DR

**SPLIT.** The two "Phase 4" documents cover unrelated concerns and need different
dispositions:

1. **`docs/specs/avatar-session-protocol.md` — MERGE into the avatar track.** The spec is
   already partially implemented (`cto-avatar-state/v1` is the shipping envelope in
   `avatar/web/lib/avatar-state.ts`); the remaining gaps (SESSION_STATE agent→avatar frame,
   explicit agent-state→avatar-state mapping, protocol metrics, Tauri window config) are
   concrete follow-up tasks for the avatar workstream. **Do not spin up a new initiative.**
2. **`docs/prds/coder-gpu-musetalk-phase4.md` — OBSOLETE. Archive with a pointer.** At
   the time, PR #4790 completed the pivot to a browser-side 3D runtime
   (TalkingHead + HeadAudio) per
   `docs/plans/3d-avatar-runtime-plan.md` "Option A", which **explicitly eliminates
   server-side GPU rendering from that historical primary path.** Current
   production has since cut over to `/echo-turn` + EchoMimic on OVH AI Deploy,
   so do not read this as saying TalkingHead/3D is live. MuseTalk / V100S
   provisioning / Kubernetes GPU autoscaling are still not public-avatar prerequisites.
   Phase 1 (self-hosted LiveKit), Phase 2
   (agent wired to self-hosted LK) and Phase 3.5 (persona admin + Better Auth) already
   shipped and stand on their own — only the GPU/MuseTalk phases are affected.

## Rationale

### What "Phase 4 session protocol" actually is

`docs/specs/avatar-session-protocol.md` defines the runtime contract between the agent,
the avatar, and the host shell:

- **Lifecycle** — `idle → connecting → connected → speaking ↔ listening → …`, with
  timeouts and a recovery path (`error → reconnecting → connected`).
- **Frame types** — `SESSION_STATE` (agent→avatar), `AVATAR_STATE` (avatar→host),
  `VISeme_CUES` (TTS→avatar), `ERROR` (any→host); each pinned to a versioned protocol
  string (`cto-avatar-session/v1`, `cto-avatar-state/v1`, `cto-avatar-visemes/v1`).
- **Agent-state → avatar-state mapping** — table from agent mode (idle/thinking/speaking…)
  to avatar voice state + gesture + viseme source.
- **Tauri integration** — transparent always-on-top window, `asset://` loading, memory
  watcher (1.5GB warn / 1.8GB force-unload).
- **Metrics & failure handling** — targets for connection latency, audio latency, viseme
  sync, frame drops, error recovery, memory usage; graceful degradation path
  `Full 3D → deterministic → static → text`.

### What just shipped in PR #4790

PR #4790 (`feat(avatar): TalkingHead runtime + HeadAudio audio-driven lip-sync`) replaces
the prior voice-bridge + ElevenLabs-alignment ingestion with a browser-local pipeline:

> `LiveKit RemoteAudioTrack → MediaStreamAudioSourceNode → TalkingHead audioCtx →
> HeadAudio AudioWorklet → viseme morph targets.`

Key changes (`git show 97c9cbd5 --stat`):

- **Added**: `avatar/web/components/TalkingHeadView.tsx`,
  `avatar/web/components/LiveKitAudioBridge.tsx`,
  `avatar/web/public/headaudio/{headworklet.min.mjs,model-en-mixed.bin}`,
  `avatar/web/types/{headaudio,talkinghead}.d.ts`.
- **Removed**: `avatar/web/components/VoiceBridgeIngestion.tsx`,
  `avatar/web/lib/runtimes/elevenlabs-alignment.ts` — the alignment hot-path is gone;
  `NEXT_PUBLIC_LIPSYNC_MODE=alignment` is now a *future* flag, not a shipping code path.
- **Touched**: `avatar/web/lib/avatar-runtime.ts`, `avatar/web/components/Room.tsx`,
  `infra/images/voice-bridge/app/main.py`, `infra/manifests/voice-bridge/deployment.yaml`.

### Overlap with `docs/specs/avatar-session-protocol.md`

| Spec element | Status after #4790 | Evidence |
|---|---|---|
| `AVATAR_STATE` envelope (`cto-avatar-state/v1`) | **Implemented** (camelCase) | `avatar/web/lib/avatar-state.ts` — `AVATAR_STATE_PROTOCOL`, `AvatarStatePayload`, runtime kinds `talkinghead \| remote-video \| deterministic-fallback`. |
| `AvatarConnectionState` / `AvatarVoiceState` | **Implemented** | Same file. Matches the spec's lifecycle labels. |
| Runtime kinds, cue sources, viseme enum (OVR-style) | **Implemented** | Same file — cueSource includes `elevenlabs-alignment`, `ovrlipsync-wasm`, `derived-text`. |
| `SESSION_STATE` agent→avatar frame | **Not implemented** | No usage of the identifier in `avatar/` or `infra/`. |
| `VISeme_CUES` push channel (`cto-avatar-visemes/v1`) | **Not implemented / deferred** | Moot for the then-current TalkingHead path (HeadAudio derives visemes client-side from PCM). Re-opens if/when an alignment-based cue source ships. |
| Tauri window config, `asset://` loading, memory caps | **Not implemented** | `rg -l tauri avatar/` → no hits. Browser-only today. |
| Metrics table (`connection_latency_ms`, `viseme_sync_ms`, `frame_drop_rate`…) | **Not instrumented** | No Prom/trace wiring found in the avatar surface. |
| Agent-state → avatar-state mapping table | **Partial** | Voice states exist; no explicit mapping layer or tests. |

**The protocol is half-shipped.** The `AVATAR_STATE` half is live; the `SESSION_STATE`,
`VISeme_CUES`, Tauri and metrics halves are not. These are 4–6 small merges, not a new
initiative.

### Why the MuseTalk PRD is now obsolete

`docs/prds/coder-gpu-musetalk-phase4.md` was written before the runtime-architecture
decision. The 3D Avatar Runtime Plan is explicit:

> "At runtime, we do **not** generate fresh video frames with a GPU model. … The rendering
> workload is delegated entirely to the end-user's device"
> — `docs/plans/3d-avatar-runtime-plan.md` §"Runtime model", §"Why server GPU is
> eliminated from the primary path"

PR #4790 executed that historical plan — `TalkingHeadView.tsx` drives a Three.js GLB in
the browser, HeadAudio runs as a WebAudio worklet on the client, and the voice-bridge pod
shrinks to STT/LLM/TTS orchestration only. This is not the live production cutline now:
current `/echo-turn` production renders async EchoMimic MP4 on OVH AI Deploy. The PRD's
Phases 4–6 (V100S provisioning, MuseTalk streaming plugin, Kubernetes GPU autoscaling,
LemonSlice cutover) are not public-avatar prerequisites.

What the MuseTalk PRD did leave behind that is **still useful** (do not delete):

- `infra/charts/openclaw-agent/skills/openclaw/{reboot-continuity,session-persistence}.md`
  — autonomy skills (independent of MuseTalk).
- Phase 1 self-hosted LiveKit chart + DNS + cert wiring (`lk.5dlabs.ai`) — already live.
- Phase 2 env-driven LiveKit configuration in `avatar/web/app/api/token/route.ts`.
- Phase 3.5 persona admin (`avatar/web/app/admin/personas/*`,
  `avatar/agent/morgan_avatar_agent/persona_preprocess.py`, Better Auth) — still valid as
  source-image/persona tooling, even though the current production renderer is
  EchoMimic MP4 on OVH AI Deploy rather than the historical TalkingHead deterministic
  avatar rig or MuseTalk inference.
- `avatar/agent/morgan_avatar_agent/musetalk_*.py` — currently dead code guarded by
  `MORGAN_AVATAR_MODE=musetalk`; archival decision below.

## Follow-up tasks (merge-path)

If the session-protocol disposition is accepted, these become concrete todos in the
avatar workstream. Suggested IDs are kebab-case and mapped to the ws-F intake style.

- `avatar-session-state-frame` — Implement `SESSION_STATE` (`cto-avatar-session/v1`)
  agent→avatar emission from the voice-bridge and subscription in `TalkingHeadView` /
  `AvatarRuntimeSurface`. Carries `session_id`, `state`, `agent_name`, `timestamp_ms`.
- `avatar-state-mapping-layer` — Extract the "agent state → avatar voice state + gesture +
  cue source" mapping from `avatar-session-protocol.md` §"Agent-State-to-Avatar-State
  Mapping" into a single pure function with unit tests; wire both runtimes
  (`talkinghead`, `deterministic-fallback`) through it.
- `avatar-protocol-metrics` — Instrument the six protocol metrics
  (`connection_latency_ms`, `audio_latency_ms`, `viseme_sync_ms`, `frame_drop_rate`,
  `error_recovery_ms`, `memory_usage_mb`) against the targets in
  `avatar-session-protocol.md` §"Key Metrics". Emit via existing voice-bridge Prom path +
  browser `performance.mark` for the client-side ones.
- `avatar-error-frame` — Emit `ERROR` frames (`code`, `recoverable`, `timestamp_ms`) from
  voice-bridge + TalkingHead surface; map to the reconnect path defined in the state
  machine. Replaces ad-hoc error strings in `avatar-state.ts`.
- `avatar-session-protocol-spec-refresh` — Update
  `docs/specs/avatar-session-protocol.md` to: (a) match the camelCase envelope the client
  actually ships; (b) drop `VISeme_CUES` push channel as the *default* path and mark it
  "deferred behind `NEXT_PUBLIC_LIPSYNC_MODE=alignment`"; (c) replace the Tauri section
  with a "browser-first, Tauri-later" note pointing to the cto-lite app if/when it
  re-enters scope.
- `avatar-musetalk-archive` — Move `docs/prds/coder-gpu-musetalk-phase4.md` to
  `docs/archive/prds/coder-gpu-musetalk-phase4.md` with a front-matter note linking to
  this memo and to PR #4790. Keep `musetalk_*.py` in-tree but add a module-level
  deprecation docstring and remove it from the default `MORGAN_AVATAR_MODE` value set
  (already `disabled` by default in `agent.py`, so low-risk).
- `avatar-gpu-cleanup` — Tear down / release the `musetalk-gpu-1` V100S if still held, and
  drop GPU-specific ArgoCD Applications (`avatar-agent-gpu`) that are no longer reachable
  from any enabled env. **Blocks on ops owner confirmation.**

## Archival-path items (if "obsolete" is accepted for the MuseTalk PRD)

- Move `docs/prds/coder-gpu-musetalk-phase4.md` → `docs/archive/prds/`.
- Leave `docs/specs/avatar-session-protocol.md` in place — it is **not** obsolete; only
  reframe and refresh per `avatar-session-protocol-spec-refresh` above.
- Preserve Phase 1 / Phase 2 / Phase 3.5 deliverables — they are already merged and
  independent of MuseTalk.

## Non-goals

- Re-litigating the historical 3D vs. generative-video architecture decision. That
  decision lives in `docs/plans/3d-avatar-runtime-plan.md` and was ratified by PR #4790,
  but it is not the current public `/echo-turn` production cutline.
- Deleting MuseTalk code paths in a single sweep. They can cool off behind
  `MORGAN_AVATAR_MODE=musetalk` until the next avatar-agent cleanup pass.
