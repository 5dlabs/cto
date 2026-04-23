# Avatar Session Protocol (Phase 4)

## Protocol Overview

Defines how agent state maps to avatar rendering state, with explicit session lifecycle, metrics, and failure handling.

**Protocol version:** `cto-avatar-session/v1`

## Session Lifecycle

```
idle → connecting → connected → speaking ↔ listening → disconnecting → idle
                              ↘ error → reconnecting → connected
```

### State Transitions

| From | To | Trigger | Timeout |
|---|---|---|---|
| `idle` | `connecting` | `START_SESSION` | 10s |
| `connecting` | `connected` | `SESSION_READY` | 30s |
| `connecting` | `error` | Timeout / auth failure | — |
| `connected` | `listening` | User speaks / STT active | — |
| `connected` | `speaking` | Agent replies / TTS active | — |
| `speaking` | `listening` | `TURN_DONE` + user input | — |
| `speaking` | `error` | TTS failure | 5s |
| `listening` | `speaking` | Agent response ready | — |
| `listening` | `error` | STT failure | 5s |
| `*` | `error` | Network failure | — |
| `error` | `reconnecting` | Auto-retry | 3s delay |
| `reconnecting` | `connected` | Session recovered | 30s |
| `reconnecting` | `error` | Recovery failed | — |

### Frame Types

#### 1. `SESSION_STATE` (agent → avatar)

```json
{
  "protocol": "cto-avatar-session/v1",
  "type": "SESSION_STATE",
  "session_id": "abc123",
  "state": "speaking",
  "agent_name": "morgan",
  "timestamp_ms": 1714000000000
}
```

#### 2. `AVATAR_STATE` (avatar → host)

```json
{
  "protocol": "cto-avatar-state/v1",
  "type": "AVATAR_STATE",
  "connection_state": "connected",
  "voice_state": "speaking",
  "runtime": {
    "kind": "remote-video",
    "ready": true,
    "fallback_active": false,
    "cue_source": "elevenlabs-alignment"
  },
  "transcript": {
    "latest_user_text": "hello",
    "latest_agent_text": "Hi there!"
  },
  "media": {
    "audio_track_ready": true,
    "video_track_ready": true
  },
  "utterance": {
    "id": "turn-1",
    "started_at_ms": 1714000001000,
    "text": "Hi there!",
    "is_final": true
  },
  "cues": {
    "visemes": [
      {"at_ms": 0, "value": "E", "weight": 0.9}
    ],
    "gestures": [
      {"name": "speak", "intensity": 0.8}
    ]
  },
  "room": {
    "room_name": "default",
    "identity": "user-123"
  },
  "metrics": {
    "connection_requested_ms": 1714000000000,
    "room_connected_ms": 1714000000500,
    "audio_track_ready_ms": 1714000000800,
    "video_track_ready_ms": 1714000001000,
    "first_speaking_state_ms": 1714000001200,
    "agent_state": "speaking",
    "latest_transcript": "Hi there!"
  },
  "error": null,
  "track_debug": {}
}
```

#### 3. `VISeme_CUES` (TTS → avatar)

```json
{
  "protocol": "cto-avatar-visemes/v1",
  "type": "VISeme_CUES",
  "utterance_id": "turn-1",
  "cues": [
    {"at_ms": 0, "value": "E", "weight": 0.9},
    {"at_ms": 120, "value": "aa", "weight": 0.7}
  ]
}
```

#### 4. `ERROR` (any → host)

```json
{
  "protocol": "cto-avatar-session/v1",
  "type": "ERROR",
  "session_id": "abc123",
  "code": "TTS_FAILED",
  "message": "ElevenLabs rate limited",
  "recoverable": true,
  "timestamp_ms": 1714000002000
}
```

## Agent-State-to-Avatar-State Mapping

| Agent State | Avatar Voice State | Avatar Gesture | Viseme Source |
|---|---|---|---|
| `idle` | `idle` | `idle` | none |
| `connecting` | `connecting` | `think` | none |
| `listening` | `listening` | `listen` | none |
| `thinking` | `listening` | `think` | none |
| `speaking` | `speaking` | `speak` | elevenlabs-alignment |
| `error` | `error` | `acknowledge` | none |

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

### Asset Loading

- Use `asset://` protocol for local `.glb` files
- Stream large assets via `tauri::api::http`
- Cache with ServiceWorker + Tauri shared cache strategy

### Memory Management

- Cap loaded avatars at 1 active + 1 preloaded
- Explicit `dispose()` on swap
- Memory watcher with warning at 1.5GB, forced cleanup at 1.8GB

## Metrics and Failure Handling

### Key Metrics

| Metric | Description | Target |
|---|---|---|
| `connection_latency_ms` | Time from START_SESSION to SESSION_READY | < 2000ms |
| `audio_latency_ms` | Time from SESSION_READY to audio track ready | < 1500ms |
| `viseme_sync_ms` | Offset between audio play and viseme start | < 50ms |
| `frame_drop_rate` | Dropped frames / total frames | < 5% |
| `error_recovery_ms` | Time from error to recovered state | < 5000ms |
| `memory_usage_mb` | WebView process memory | < 1800MB |

### Failure Modes

| Failure | Recovery | User Impact |
|---|---|---|
| Network disconnect | Auto-reconnect (3 retries, exponential backoff) | Brief pause, then resume |
| TTS failure | Fallback to text-only (no visemes) | Avatar shows "speaking" without mouth movement |
| Asset load failure | Use deterministic fallback image | Static avatar, no animation |
| WebGL error | Canvas2D fallback message | Basic avatar display, no 3D |
| Memory pressure | Unload preloaded avatar, reduce LOD | Slightly longer load time for next avatar |

### Graceful Degradation Path

```
Full 3D avatar → Deterministic fallback → Static image → Text message
```

Each level provides less visual fidelity but maintains session continuity.
