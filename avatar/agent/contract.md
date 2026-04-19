# MuseTalk Avatar NATS Contract

> Scope: authoritative request / response schema for talking to the
> `musetalk-worker` GPU service from the avatar agent (client side).
> Source of truth: [`infra/images/musetalk-worker/worker.py`](../../infra/images/musetalk-worker/worker.py)
> (module docstring). Do not change that file from the client side — raise a
> coordinated PR in the worker repo instead.

## Transport

- **Server:** `musetalk-nats.cto.svc.cluster.local:4222` (NATS + JetStream).
- **Stream:** `AVATAR` (configured on the server side). Subjects attached to
  the stream: `avatar.render.request`, `avatar.render.result`.
- **Client publish path:** `js.publish("avatar.render.request", ...)` — the
  worker consumes via a JetStream pull subscription (`musetalk-worker`
  consumer), so requests must land in the stream, not on core NATS.
- **Result path:** worker publishes the response on **core NATS** on the
  configured `callback_subject` (default `avatar.render.result`). Clients
  subscribe on core NATS and filter by `request_id` in the payload.

## Request (agent → worker)

Subject: `avatar.render.request`

```json
{
  "request_id": "uuid-v4",
  "persona_id": "morgan",
  "reference_image_url": "https://.../morgan.png",
  "audio_url": "https://.../utterance.wav",
  "audio_hash": "sha256:<hex>",
  "fps": 25,
  "callback_subject": "avatar.render.result"
}
```

Field notes:

- `reference_image_url` — HTTP(S) or `file://` URL the worker can `GET`.
  Agents currently source this from `MORGAN_IMAGE_URL` /
  `MORGAN_PLACEHOLDER_IMAGE_URL` via
  `AgentConfig.avatar_image_url` / `AgentConfig.musetalk_reference_image_url`.
- `audio_url` — HTTP(S) or `file://` URL for the full utterance audio. See
  **Known gaps** below.
- `audio_hash` — optional SHA-256 of the audio bytes. Used by the worker for
  caching. Safe to send `null` / omit during early wiring.
- `fps` — target render FPS. The worker honours it when it can; the client
  must still time its `capture_frame` calls at its own `target_fps`.
- `callback_subject` — reply subject; overridable per-request.

## Response (worker → agent)

Subject: the `callback_subject` supplied by the request (core NATS publish).

```json
{
  "request_id": "uuid-v4",
  "persona_id": "morgan",
  "video_url": "https://.../out.mp4",
  "render_time_s": 12.3,
  "cached": false,
  "bootstrap_only": false,
  "gpu": "NVIDIA A10G",
  "dtype": "float16",
  "error": null
}
```

Field notes:

- `video_url` — MP4 H.264 file. In the dev harness this may be a `file://`
  path on a shared volume; in prod it is expected to be an S3 HTTPS URL.
  The client must be tolerant of both.
- `bootstrap_only: true` — worker warmed up but did **not** render a clip.
  The client should skip frame decode and treat the call as a no-op.
- `error` — string when rendering failed. Client should log + surface, not
  crash.

## Latency / streaming mismatch

The worker currently renders **one MP4 per request** — it is batch, not a
streaming per-frame feed. The client must:

1. Publish the request with a resolvable `audio_url`.
2. Await the response (JSON on `callback_subject`).
3. Download the MP4 from `video_url`.
4. Decode frames (e.g. via PyAV / OpenCV) and hand them to the LiveKit
   `VideoSource` on the agent-side cadence.

First-frame latency is therefore bounded below by the full-utterance render
time. True low-latency streaming requires a different worker protocol (e.g. a
per-chunk streaming subject) and is **not** in scope for this wiring PR.

## Known gaps / TODO (client side)

- **Audio upload.** LiveKit TTS plugins emit raw PCM frames, not URLs. Until
  the agent has a place to upload those bytes (S3 presigned PUT, in-cluster
  MinIO, or the worker gains an inline-audio subject), the NATS client takes
  a caller-provided `audio_url`. Wiring TTS → upload → NATS is follow-up
  work.
- **MP4 frame decode.** Pinning PyAV / OpenCV bloats the image; the current
  implementation returns the raw response (video URL + metadata) and leaves
  frame decode as a hook. The idle LiveKit track is driven by the CPU stub
  frame generator so the track is still visibly live in the room.
- **Streaming protocol.** Track a follow-up with the worker owners if /
  when per-chunk streaming is needed.

## Stub fallback

Set `MUSETALK_USE_STUB=1` (or `MORGAN_MUSETALK_USE_STUB=true`) to bypass NATS
entirely and keep the CPU procedural frame generator. This is the default in
CI / local unit tests and is required on any machine without network access
to the NATS cluster.
