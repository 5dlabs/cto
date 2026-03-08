# Provider Swap Spikes

These swaps are already wired into the agent via environment variables so latency spikes can happen without code changes.

## Baseline

```env
MORGAN_STT_MODE=livekit-flux
MORGAN_TTS_MODE=elevenlabs
MORGAN_LLM_BACKEND=openclaw
```

Why: fastest path to a working room-backed avatar while keeping the OpenClaw reasoning path intact.

## STT spike: direct Deepgram Flux

```env
MORGAN_STT_MODE=deepgram-flux
DEEPGRAM_API_KEY=your_deepgram_key
MORGAN_DEEPGRAM_EAGER_EOT_THRESHOLD=0.4
MORGAN_DEEPGRAM_EOT_TIMEOUT_MS=1500
```

Why: exposes Deepgram’s conversational endpointing controls directly for tighter end-of-turn tuning.

## STT fallback: LiveKit or direct Nova

```env
MORGAN_STT_MODE=livekit-nova
```

or

```env
MORGAN_STT_MODE=deepgram-nova
DEEPGRAM_API_KEY=your_deepgram_key
```

Why: useful if Flux is unstable or not available in the target region.

## TTS spike: Cartesia

```env
MORGAN_TTS_MODE=cartesia
CARTESIA_API_KEY=your_cartesia_key
MORGAN_CARTESIA_MODEL=sonic-turbo
```

Why: quickest low-latency swap candidate when ElevenLabs becomes the bottleneck.

## Inference-only fallback

If you want to remove extra provider keys during setup, use LiveKit Inference-backed descriptors:

```env
MORGAN_STT_MODE=livekit-flux
MORGAN_TTS_MODE=livekit-cartesia
MORGAN_LLM_BACKEND=inference
MORGAN_LLM_MODEL=openai/gpt-4.1-mini
```

Why: reduces setup friction when validating the room and avatar path first.

## Integrated voice contingency

If measured latency still misses target after the above swaps:

- Keep the browser and room experience as the reference UX.
- Run a separate OpenAI Realtime or equivalent integrated voice spike.
- Compare only three numbers across both stacks:
  - p50 end-of-turn to first audio
  - p95 end-of-turn to first audio
  - interruption responsiveness

Do not replace the baseline stack unless the integrated spike wins clearly on measured latency without breaking avatar quality or operational simplicity.
