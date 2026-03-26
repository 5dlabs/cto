# ElevenLabs Optimization Plan for Morgan Avatar

This plan is intentionally limited to behavior that is already implemented and configurable in this repo.

## Current implementation (code-verified)

From `avatar/agent/morgan_avatar_agent/config.py` and `providers.py`:

- Default TTS path is direct ElevenLabs plugin:
  - `MORGAN_TTS_MODE=elevenlabs`
  - `MORGAN_ELEVEN_MODEL=eleven_flash_v2_5`
  - `MORGAN_ELEVEN_STREAMING_LATENCY=3`
  - `MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE=80,120,200,260`
- Alternate ElevenLabs path is LiveKit Inference:
  - `MORGAN_TTS_MODE=livekit-elevenlabs`
- Voice and language are already configurable:
  - `MORGAN_ELEVEN_VOICE_ID`
  - `MORGAN_TTS_LANGUAGE`

## Goal

Reduce end-of-turn to first audio while keeping speech quality acceptable and preserving current architecture (`LiveKit + LemonSlice + OpenClaw`).

## Phase 1: Tune existing ElevenLabs knobs (highest value, low risk)

1. Baseline with current defaults using `scripts/summarize_latency.py`.
2. Sweep `MORGAN_ELEVEN_STREAMING_LATENCY` with values `1,2,3,4`.
3. Sweep `MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE` presets, for example:
   - `60,100,160,220`
   - `80,120,200,260` (current)
   - `100,140,220,300`
4. Keep each run to the same prompt set (greeting + 3 turns) and compare:
   - `p50` and `p95` `end_of_turn_to_first_audio`
   - subjective speech smoothness

Exit criteria:
- Keep a new setting only if `p95` improves without clear speech regressions.

## Phase 2: Compare ElevenLabs transport modes

Compare these two configs under identical prompts:

A) Direct ElevenLabs plugin (current default)
```env
MORGAN_TTS_MODE=elevenlabs
```

B) LiveKit Inference ElevenLabs
```env
MORGAN_TTS_MODE=livekit-elevenlabs
```

Exit criteria:
- Adopt the faster mode only if latency gains are consistent across multiple runs.

## Phase 3: Voice/model variants already supported by env

1. Keep transport mode fixed from Phase 2.
2. Evaluate a small matrix of voice/model values via env:
   - `MORGAN_ELEVEN_VOICE_ID`
   - `MORGAN_ELEVEN_MODEL`
3. Reject variants that improve latency but reduce persona fit or intelligibility.

## Known constraints

- This document does not assume availability of new upstream ElevenLabs features unless validated in code first.
- Any new API fields (for example additional expressive controls) should be treated as a separate implementation task in `providers.py`, not a docs-only change.

## Deliverables after running the plan

- Final recommended `.env` values for ElevenLabs mode/model/voice/chunking.
- Before/after latency summary from `scripts/summarize_latency.py`.
- Short decision note in `avatar/docs/decision-review.md` describing why the chosen settings won.
