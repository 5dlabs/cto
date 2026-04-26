# Avatar Session Protocol (Phase 4)

> **Status:** Reconciled with shipped code from PR #4790 (`avatar/web/lib/avatar-state.ts`).
> Source of truth is the TypeScript definitions in that module. This document
> describes those definitions in prose and flags extensions still on the roadmap.

## Protocol Overview

The avatar subsystem uses a single state-projection contract between the
LiveKit-backed avatar runtime and its host (the Tauri shell or an embedding
iframe). A runtime adapter projects LiveKit room state + utterance state into
an `AvatarStatePayload`, which is then published to the host.

**Protocol version constant:** `cto-avatar-state/v1`
(exported as `AVATAR_STATE_PROTOCOL` from `avatar/web/lib/avatar-state.ts`).

Three independent protocol namespaces are used today:

| Namespace | Producer | Consumer | Shape |
|---|---|---|---|
| `cto-avatar-state/v1` | Room adapter (`avatar/web/lib/avatar-state.ts`) | Host (Tauri / iframe parent) | `AvatarStatePayload` |
| `cto-voice-bridge` (implicit) | Voice bridge backend | Room adapter | `VoiceBridgeFrame` |
| (no namespace — in-process) | LiveKit audio track → HeadAudio | TalkingHead runtime | `MediaStreamTrack` |

> **NOTE:** Earlier drafts of this document referenced a separate
> `cto-avatar-session/v1` envelope for `SESSION_STATE` and structured `ERROR`
> frames. No such envelope is implemented today — session state is inferred
> from the projected `AvatarStatePayload.connectionState` / `voiceState`
> fields, and errors are carried as the optional `error` string on that
> payload. The envelope remains on the roadmap; see
> [Reserved for future extension](#reserved-for-future-extension).

## Session Lifecycle

The shipped protocol exposes two orthogonal state machines on the payload:

### `connectionState: AvatarConnectionState`

```
idle → connecting → connected
                 ↘ error
```

Values: `"idle" | "connecting" | "connected" | "error"`.

### `voiceState: AvatarVoiceState`

```
idle → connecting → listening ↔ speaking
                 ↘ error
```

Values: `"idle" | "connecting" | "listening" | "speaking" | "error"`.

> **NOTE:** There is no dedicated `disconnecting` or `reconnecting` state in
> the current implementation. Reconnect is handled transparently by the
> LiveKit client; on a fatal failure the adapter surfaces `connectionState:
> "error"` with an `error` string. Explicit lifecycle transitions (including
> retry accounting) are part of the `SESSION_STATE` envelope tracked under
> [Reserved for future extension](#reserved-for-future-extension).

## Payload: `AvatarStatePayload`

Emitted by `Room.tsx` on every adapter projection via `window.parent.postMessage({ type: "cto-avatar-state", payload }, "*")`.

**Field names are camelCase — they match the exported TypeScript types verbatim.**

```jsonc
{
  "protocol": "cto-avatar-state/v1",
  "connectionState": "connected",
  "voiceState": "speaking",
  "runtime": {
    "kind": "talkinghead",
    "ready": true,
    "fallbackActive": false,
    "cueSource": "ovrlipsync-wasm"
  },
  "transcript": {
    "latestUserText": "hello",
    "latestAgentText": "Hi there!"
  },
  "media": {
    "audioTrackReady": true,
    "videoTrackReady": false
  },
  "utterance": {
    "id": "turn-1",
    "startedAtMs": 1714000001000,
    "text": "Hi there!",
    "isFinal": true
  },
  "cues": {
    "visemes": [
      { "atMs": 0, "value": "E", "weight": 0.9 }
    ],
    "gestures": [
      { "name": "speak", "intensity": 0.9 }
    ]
  },
  "room": {
    "roomName": "default",
    "identity": "user-123"
  },
  "error": null,
  "metrics": { /* free-form, adapter-specific */ },
  "trackDebug": { /* free-form, diagnostic */ }
}
```

### Enum values

| Field | Values |
|---|---|
| `connectionState` | `idle`, `connecting`, `connected`, `error` |
| `voiceState` | `idle`, `connecting`, `listening`, `speaking`, `error` |
| `runtime.kind` | `deterministic-fallback`, `remote-video`, `talkinghead` |
| `runtime.cueSource` | `none`, `derived-text`, `elevenlabs-alignment`, `ovrlipsync-wasm` |
| `cues.gestures[].name` | `idle`, `listen`, `speak`, `think`, `acknowledge` |
| `cues.visemes[].value` (OVR set) | `sil`, `PP`, `FF`, `TH`, `DD`, `kk`, `CH`, `SS`, `nn`, `RR`, `aa`, `E`, `I`, `O`, `U` |

### Cue units

- `atMs` — milliseconds since `utterance.startedAtMs`.
- `durationMs` (optional) — duration of the viseme cue.
- `weight` — `0.0–1.0` blend target.
- `intensity` — `0.0–1.0` gesture amplitude.

## Voice Bridge Frames (`VoiceBridgeFrame`)

The voice bridge emits a discriminated union of frames over its data channel.
The adapter may implement `ingestBridgeFrame(frame)` to fold them into the
projected payload. Current frame types (exact as shipped):

```ts
type VoiceBridgeFrame =
  | { type: "started";     session_id: string; agent: string }
  | { type: "transcript";  text: string; agent: string }
  | { type: "reply_delta"; text: string; agent: string }
  | { type: "reply_text";  text: string; agent: string }
  | { type: "turn_done";   agent: string }
  | { type: "error";       error: string }
  | {
      type: "alignment";
      atMs: number;
      chars: string[];
      char_start_ms: number[];
      char_end_ms: number[];
      agent: string;
    };
```

> **NOTE:** Field naming here is mixed — `session_id`, `char_start_ms`, and
> `char_end_ms` are `snake_case` because they match the legacy backend wire
> format, while `atMs` follows the avatar-state convention. This is
> intentional and preserved for backwards compatibility. Normalisation into
> the avatar-state payload happens inside the adapter.

### Lip-sync sourcing

Two cue sources coexist:

1. **`ovrlipsync-wasm` (preferred, hot path).** The TalkingHead runtime
   (`HeadAudio`) analyses the raw agent `MediaStreamTrack` via the Web Audio
   graph and drives visemes client-side. `LiveKitAudioBridge.tsx` wires the
   remote audio publication into `TalkingHead.attachAudio(...)` for exactly
   this purpose. **No server-side alignment frames are required for
   lip-sync** when this path is active — HeadAudio handles timing locally.
2. **`elevenlabs-alignment` / `derived-text` (fallback, non-hot).** The
   adapter may derive `AvatarVisemeCue[]` from `VoiceBridgeFrame` of type
   `alignment`, or scaffold them from text via `deriveVisemeScaffold()`. This
   is used when the TalkingHead runtime is unavailable or the audio track
   is not yet "flowing".

Playback of Morgan's audio is delegated to LiveKit's native `AudioTrack`
renderer (`AssistantAudioRenderer` in `Room.tsx`), which both produces audible
output and keeps the track flowing so the Web Audio graph can tap it.

## Adapter contract

```ts
interface AvatarRuntimeAdapter {
  readonly kind: AvatarRuntimeKind;
  readonly cueSource: AvatarCueSource;

  project(input: AvatarRuntimeInput): AvatarStatePayload;
  ingestBridgeFrame?(frame: VoiceBridgeFrame): void;
}
```

`AvatarRuntimeInput` carries the raw LiveKit state (`lk`), timing anchors
(`timing.*At` timestamps), the current `utterance` if any, and an optional
`error` string. See `avatar-state.ts` for the full shape.

## Reserved for future extension

The following items are **intentionally not implemented** in the shipped
protocol. They are tracked as separate work items; this spec documents the
intent so they can be added without a breaking version bump.

| Extension | Todo ID | Sketch |
|---|---|---|
| `SESSION_STATE` envelope (agent → avatar) | `avatar-session-state-frame` | Explicit lifecycle frames with `session_id`, `state`, `agent_name`, `timestamp_ms`. Today this is inferred from `AvatarStatePayload`. |
| Structured error frame | `avatar-error-frame` | Replace free-form `error: string` with `{ code, message, recoverable, timestamp_ms }`. |
| Metrics counters schema | `avatar-protocol-metrics` | Formalise `metrics` field: `connection_latency_ms`, `audio_latency_ms`, `viseme_sync_ms`, `frame_drop_rate`, `error_recovery_ms`, `memory_usage_mb` with documented targets. |
| Server-side emotion → `AVATAR_STATE` mapping | `avatar-state-mapping-layer` | Agent emits semantic emotion/state; server maps to `voiceState` + `gestures` so adapters don't need per-agent logic. |

Until these land, consumers must treat `error` as opaque text, `metrics` and
`trackDebug` as free-form diagnostic payloads, and derive lifecycle from
`connectionState` / `voiceState`.

## Tauri Integration

### Window Configuration

```json
{
  "tauri": {
    "windows": [
      {
        "label": "avatar",
        "title": "Morgan Avatar",
        "width": 400,
        "height": 600,
        "resizable": false,
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true
      }
    ]
  }
}
```

### Host Transport

The Room surface posts payloads to its parent with
`window.parent.postMessage({ type: "cto-avatar-state", payload }, "*")`. Tauri
embeddings should subscribe via `window.addEventListener("message", ...)` and
filter by `event.data.type === "cto-avatar-state"`.

### Asset Loading

- Use `asset://` protocol for local `.glb` files.
- Stream large assets via `tauri::api::http`.
- Cache with ServiceWorker + Tauri shared cache strategy.

### Memory Management

- Cap loaded avatars at 1 active + 1 preloaded.
- Explicit `dispose()` on swap.
- Memory watcher with warning at 1.5 GB, forced cleanup at 1.8 GB.

## Failure Handling

With no structured error frame yet (see
[Reserved for future extension](#reserved-for-future-extension)), failures
surface as:

- `connectionState: "error"` and/or `voiceState: "error"`,
- `runtime.fallbackActive: true` when degrading,
- `runtime.cueSource` demoted toward `none`,
- free-form `error` string on the payload.

### Failure Modes

| Failure | Recovery | User Impact |
|---|---|---|
| Network disconnect | LiveKit auto-reconnect | Brief pause, then resume |
| TTS failure | `voiceState` stays `speaking` without fresh cues; HeadAudio idles | Avatar visible but mouth static |
| Asset load failure | Adapter falls back to `deterministic-fallback` | Static avatar, scaffolded gestures only |
| WebGL error | Non-3D runtime (e.g. `remote-video`) | 2D rendering, no mesh |
| Memory pressure | Unload preloaded avatar, reduce LOD | Slightly longer load time for next avatar |

### Graceful Degradation Path

```
talkinghead → remote-video → deterministic-fallback
```

Each level maps to an `AvatarRuntimeKind` and a matching `cueSource`. Session
continuity is preserved at every level — the host receives a well-formed
`AvatarStatePayload` regardless of which runtime is active.
