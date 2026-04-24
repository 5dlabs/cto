# morgan-avatar-minimal

**Purpose:** Animation pipeline validation only. Registers as `morgan-avatar` LiveKit worker, streams `sample.mp3` (Morgan's cloned voice) as a raw audio track, then keeps the worker alive. No STT, no LLM, no TTS, no avatar session.

## What this validates
Browser at `https://avatar.5dlabs.ai/` POSTs `/api/token` → LK dispatches to this worker → worker publishes audio track → HeadAudio worklet drives TalkingHead visemes.

## Run locally
```bash
cp ../../praxis_visa_reading.mp3 sample.mp3   # already done if baked into image

export LIVEKIT_URL=wss://lk.5dlabs.ai
export LIVEKIT_API_KEY=...
export LIVEKIT_API_SECRET=...

pip install .
python worker.py start
```

## Docker
```bash
docker build --platform linux/amd64 -t ghcr.io/5dlabs/morgan-avatar-minimal:v0.1.0 .
docker push ghcr.io/5dlabs/morgan-avatar-minimal:v0.1.0
```

## Status
**Temporary validation tool.** Replace with the full `avatar/agent/agent.py` worker (openclaw-morgan-avatar) once Phase 2 STT/LLM/TTS/avatar wiring is unblocked (see PR #4812).
