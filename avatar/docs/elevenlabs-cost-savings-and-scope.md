# ElevenLabs Cost Savings and Scope Notes (Morgan Avatar)

Date: 2026-03-08

This note captures cost levers and feature opportunities we can revisit after
the two-way voice path is stable.

## Current Implementation Snapshot

From current config and provider wiring:

- TTS defaults to direct ElevenLabs plugin (`MORGAN_TTS_MODE=elevenlabs`) with
  `eleven_flash_v2_5`.
- STT currently defaults to Deepgram Flux via LiveKit inference
  (`MORGAN_STT_MODE=livekit-flux`).
- Avatar rendering is LemonSlice-dependent when enabled.

Implication: near-term cost focus should be TTS/STT usage and turn length, not
avatar model quality tuning.

## Cost Savings Levers (No Architecture Rewrite)

## 1. Keep `eleven_flash_v2_5` as production default

Why:

- ElevenLabs positions Flash v2.5 as low-latency and lower-cost than higher
  quality tiers.
- For conversational UX, this is usually the best quality-per-dollar baseline.

Action:

- Keep `MORGAN_ELEVEN_MODEL=eleven_flash_v2_5` for default traffic.
- Only escalate specific traffic segments to higher-cost voices/models.

## 2. Add response-length controls in Morgan prompts

Why:

- ElevenLabs TTS cost scales with generated text volume.
- Open-ended responses create direct cost growth and slower turn completion.

Action:

- Keep default spoken replies short (1-3 sentences).
- Add concise mode for normal turns and allow long-form only when explicitly
  requested by user intent.

## 3. Route only high-value flows to avatar mode

Why:

- Avatar sessions add additional provider cost and failure surface.
- Many interactions only need audio.

Action:

- Default to audio-only for routine conversations.
- Use avatar mode for demos, onboarding, sales-facing interactions, or when
  user explicitly requests visual presence.

## 4. Add spend telemetry by session

Why:

- We need hard numbers before optimization decisions.

Action:

- Track: characters synthesized, STT minutes, call duration, and failures by
  session ID.
- Build weekly cost report split by environment (`dev`, `staging`, `prod`).

## 5. If we move to ElevenAgents orchestration, use silence discounts

Why:

- ElevenLabs Agents docs state silent periods longer than 10s are billed at 5%
  of normal per-minute call cost.

Action:

- Only relevant if/when conversation orchestration moves to ElevenAgents.
- Keep as a future option, not a current assumption for the direct plugin path.

## Additional ElevenLabs Functionality Worth Adding

These are realistic upgrades for our voice stack that are currently out of
scope but high-value.

## 1. Pronunciation dictionaries (high value, low effort)

Use for product names, acronyms, and proper nouns ("OpenClaw", "ClawHub",
"LemonSlice", team names). This directly improves perceived quality.

## 2. Scribe v2 Realtime spike (medium effort)

Run a controlled comparison versus current Deepgram path for:

- end-of-turn latency
- transcript correctness on domain terms
- interruption behavior

Adopt only if it materially improves latency or recognition quality.

## 3. Voice cloning/remixing for brand voice consistency (medium effort)

Use ElevenLabs voice cloning/remixing to create a stable "Morgan house voice"
that matches brand tone and remains consistent across demos and production.

## 4. Voice changer for post-processing and patch workflows (low-medium effort)

Useful for selective phrase correction and higher-quality prerecorded content,
not for first-line real-time loop replacement.

## 5. Text-to-dialogue and expressive v3 experiments (future)

Promising for richer emotional delivery but not a day-one dependency for
real-time reliability. Keep as a quality optimization track after transport and
latency stability.

## Recommendation

Near term (now):

- Optimize for reliability + low cost with Flash v2.5 and concise responses.
- Add pronunciation dictionaries and spend telemetry first.

Next phase:

- Run Scribe v2 Realtime spike and voice-clone/remix evaluation.
- Revisit ElevenAgents orchestration only if we want integrated turn-taking,
  cost controls, and built-in conversation tooling.

## Sources

- [ElevenLabs TTS capabilities](https://elevenlabs.io/docs/overview/capabilities/text-to-speech)
- [ElevenLabs models overview](https://elevenlabs.io/docs/overview/models)
- [ElevenLabs API pricing](https://elevenlabs.io/pricing/api)
- [ElevenLabs plan pricing](https://elevenlabs.io/pricing)
- [ElevenAgents intro and pricing behavior](https://elevenlabs.io/docs/conversational-ai/docs/introduction)
- [Scribe v2 Realtime announcement](https://elevenlabs.io/blog/introducing-scribe-v2-realtime)
- [Speech-to-text capabilities](https://elevenlabs.io/docs/capabilities/speech-to-text)
- [Pronunciation dictionaries cookbook](https://elevenlabs.io/docs/eleven-api/guides/cookbooks/text-to-speech/pronunciation-dictionaries)
- [Voice cloning docs](https://elevenlabs.io/docs/creative-platform/voices/voice-cloning)
- [Voice changer docs](https://elevenlabs.io/docs/capabilities/voice-changer)
