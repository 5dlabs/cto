# ElevenLabs Playbook for Morgan Avatar

This doc is the practical companion to `docs/elevenlabs-enhancement-plan.md`.
That plan is future-looking. This playbook is for the current debugging cycle:
how to use ElevenLabs well right now, what it can fix, and what it cannot.

## Current Read

Recent local runs show Morgan's greeting reaching TTS first-byte quickly,
roughly `0.39 s` in the latest session log. That means the current hard
failures are more likely in room wiring, avatar playback, or LLM latency than
in raw ElevenLabs synthesis speed.

Use ElevenLabs aggressively for voice quality and control. Do not treat it as
the first suspect when:

- the greeting text exists but no audio is heard
- the avatar never appears
- OpenClaw replies arrive long after the worker has already timed out

Those are usually transport, room, or LLM problems, not TTS problems.

## The Two Useful Modes

### 1. `livekit-elevenlabs`

Use this when you want the simplest path while stabilizing the room.

```env
MORGAN_TTS_MODE=livekit-elevenlabs
MORGAN_ELEVEN_MODEL=eleven_flash_v2_5
MORGAN_ELEVEN_VOICE_ID=iP95p4xoKVk53GoZ742B
```

Why this is good:

- minimal setup
- keeps TTS on ElevenLabs
- works through LiveKit Inference
- good for proving the browser, room, and avatar path first

Limits:

- LiveKit Inference does not expose custom or community ElevenLabs voices
- it is not the right mode if we want to use our own ElevenLabs account
  features deeply, including custom voice assets

Official reference:

- LiveKit Inference ElevenLabs TTS:
  [docs.livekit.io/agents/models/tts/inference/elevenlabs](https://docs.livekit.io/agents/models/tts/inference/elevenlabs/)

### 2. `elevenlabs`

Use this when you want to spend our ElevenLabs credits directly and unlock more
control.

```env
MORGAN_TTS_MODE=elevenlabs
ELEVEN_API_KEY=your_elevenlabs_key
MORGAN_ELEVEN_MODEL=eleven_flash_v2_5
MORGAN_ELEVEN_VOICE_ID=your_voice_id
MORGAN_ELEVEN_STREAMING_LATENCY=3
MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE=80,120,200,260
```

Why this is better once the room is stable:

- uses our own ElevenLabs account and credits directly
- supports the direct ElevenLabs LiveKit plugin
- gives us the tuning knobs already wired in this repo:
  `MORGAN_ELEVEN_STREAMING_LATENCY` and
  `MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE`
- is the path to custom or cloned voices

Important gotcha:

- ElevenLabs docs often show `ELEVENLABS_API_KEY`
- the LiveKit ElevenLabs plugin used by this repo expects `ELEVEN_API_KEY`

Repo proof:

- direct mode is wired in
  [providers.py](/Users/jonathon/5dlabs/cto/avatar/agent/morgan_avatar_agent/providers.py)
- the direct mode tuning env vars already exist
  [config.py](/Users/jonathon/5dlabs/cto/avatar/agent/morgan_avatar_agent/config.py)

Official references:

- LiveKit ElevenLabs plugin:
  [docs.livekit.io/agents/models/tts/elevenlabs](https://docs.livekit.io/agents/models/tts/elevenlabs/)
- ElevenLabs latency optimization:
  [elevenlabs.io/docs/eleven-api/best-practices/latency-optimization](https://elevenlabs.io/docs/eleven-api/best-practices/latency-optimization)

## Best Model Order for Morgan

### `eleven_flash_v2_5`

Start here.

Why:

- ElevenLabs positions it as the fastest low-latency model for real-time use
- LiveKit supports it in both inference docs and the plugin path
- it is the best fit while Morgan is still conversational and interruption
  sensitive

Reference:

- ElevenLabs models overview:
  [elevenlabs.io/docs/overview/models](https://elevenlabs.io/docs/overview/models)

### `eleven_turbo_v2_5`

Test this after `flash` if you want a little more quality and can tolerate
slightly more delay.

Use it when:

- the room is stable
- OpenClaw latency is under control
- the next complaint is voice quality rather than speed

### `eleven_v3` / newer conversational models

Treat this as a follow-up experiment, not today's fix.

Why:

- availability depends on the LiveKit path we use
- the current repo is already tuned around low-latency conversational output
- the first win is getting a reliable two-way loop, not richer expressiveness

See the future plan:

- [elevenlabs-enhancement-plan.md](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-enhancement-plan.md)

## Recommended Order for Current Work

1. Keep `livekit-elevenlabs` while fixing room capacity, avatar subscription,
   and OpenClaw timeout behavior.
2. Once greeting and reply audio are consistently audible, run one A/B test with
   direct `elevenlabs`.
3. If direct mode wins on voice quality without hurting TTFB meaningfully, keep
   it and spend credits there.
4. Only after that consider higher-quality model swaps or ElevenLabs STT.

## What We Can Get from ElevenLabs Next

### Better voices

If Morgan should sound more like a specific person, direct `elevenlabs` is the
practical path because LiveKit Inference only exposes default voices.

### Better control over latency

The direct plugin path already lets us tune:

- `MORGAN_ELEVEN_STREAMING_LATENCY`
- `MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE`

That makes it the best path if we want to spend credits to optimize perceived
responsiveness without changing the rest of the stack.

### STT consolidation

ElevenLabs STT is available through both LiveKit Inference and the direct
plugin, including `scribe_v2_realtime`.

This is worth testing only if we want to simplify operations by consolidating
STT and TTS on one provider, or if Deepgram becomes the weak link.

Reference:

- [docs.livekit.io/agents/models/stt/elevenlabs](https://docs.livekit.io/agents/models/stt/elevenlabs/)

## Quick Recipes

### Safest debug baseline

```env
MORGAN_TTS_MODE=livekit-elevenlabs
MORGAN_ELEVEN_MODEL=eleven_flash_v2_5
```

### Direct ElevenLabs spend-our-credits path

```env
MORGAN_TTS_MODE=elevenlabs
ELEVEN_API_KEY=your_elevenlabs_key
MORGAN_ELEVEN_MODEL=eleven_flash_v2_5
MORGAN_ELEVEN_STREAMING_LATENCY=3
MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE=80,120,200,260
```

### Quality-first follow-up

```env
MORGAN_TTS_MODE=elevenlabs
ELEVEN_API_KEY=your_elevenlabs_key
MORGAN_ELEVEN_MODEL=eleven_turbo_v2_5
```

## Measurement Rules

When comparing ElevenLabs options, only compare these numbers:

- TTS TTFB from `agent/runs/*-latency.ndjson`
- end-of-turn to first audio
- one subjective note on voice quality

If the failure is "Morgan never speaks at all", fix room or LLM issues first.
Do not swap TTS providers blindly.
