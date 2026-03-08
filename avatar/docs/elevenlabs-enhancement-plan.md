# ElevenLabs Enhancement Plan for Morgan Avatar

A plan to adopt newer ElevenLabs features identified from the docs and changelog (March 2026) to improve Morgan’s voice quality, latency, and future integration options.

Companion note:
- [ElevenLabs cost savings and scope notes](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-cost-savings-and-scope.md)

---

## 1. Eleven v3 Conversational TTS (Priority: High)

**What**: Switch Morgan’s TTS to the `eleven_v3_conversational` model and optionally add `suggested_audio_tags` for more expressive delivery.

**Why**: Better voice quality and expressiveness in agent conversations; tags (e.g. “confident”, “warm”) guide how the model delivers lines.

**Tasks**:
- [ ] Confirm whether LiveKit Inference / ElevenLabs plugin exposes `eleven_v3_conversational` as a model option.
- [ ] If yes: update `MORGAN_ELEVEN_MODEL` in `agent/.env` (or equivalent config) to `eleven_v3_conversational`.
- [ ] If no: check ElevenLabs direct API or LiveKit plugin docs for v3 model availability.
- [ ] Optional: define 3–5 `suggested_audio_tags` for Morgan’s persona (e.g. “confident”, “calm”, “collaborative”) and wire them into the TTS config if the API supports it.
- [ ] Run a short latency/quality comparison (greeting + 2–3 turns) vs current model.

**References**: [Changelog – Eleven v3 conversational](https://elevenlabs.io/docs/changelog), [ElevenAgents overview](https://elevenlabs.io/docs/eleven-agents/overview)

---

## 2. Global Routing and Shorter Audio Chunks (Priority: Medium)

**What**: Use ElevenLabs’ default global routing and, if we ever go direct to Eleven’s streaming API, adopt 100 ms audio chunks instead of 250 ms.

**Why**: Global routing reduces latency by choosing the nearest region; shorter chunks improve perceived responsiveness.

**Tasks**:
- [ ] Confirm we use the default `api.elevenlabs.io` base URL (global routing is now default).
- [ ] If we add a custom ElevenLabs streaming client: use 100 ms chunk length (per React/client SDK changes).
- [ ] Document any region or base-URL overrides in `docs/provider-spikes.md` or env example.

**References**: [Changelog – Global servers out of beta](https://elevenlabs.io/docs/changelog)

---

## 3. STT: Scribe v2 and Keyterms (Priority: Low / Spike)

**What**: Evaluate ElevenLabs Scribe v2 for STT, with optional `keyterms` for Morgan-specific phrases.

**Why**: Scribe v2 is a newer transcription model; keyterms can improve recognition of names and domain terms.

**Tasks**:
- [ ] Document current STT path (Deepgram Flux via LiveKit Inference) in `docs/provider-spikes.md`.
- [ ] If we add an ElevenLabs STT option: implement Scribe v2 with `keyterms` (e.g. “Morgan”, “CTO”, “roadmap”) and compare accuracy/latency vs Deepgram Flux.
- [ ] Gate this on whether we want to consolidate STT+TTS on ElevenLabs for simpler ops.

**References**: [Changelog – Scribe v2](https://elevenlabs.io/docs/changelog), [ElevenAPI quickstart](https://elevenlabs.io/docs/eleven-api/quickstart)

---

## 4. ElevenAgents Patterns (Priority: Future / Reference)

**What**: Use ElevenAgents as a reference for turn-taking, guardrails, and tool integration—not a full migration.

**Why**: Our stack (LiveKit + OpenClaw + LemonSlice) is working; ElevenAgents offers patterns we can borrow.

**Tasks**:
- [ ] Review ElevenAgents turn config (speculative turn, spelling patience, TurnModel v2/v3) and compare to our Deepgram endpointing and `MORGAN_FALSE_INTERRUPTION_TIMEOUT`.
- [ ] If we add output guardrails: consider custom guardrails (LLM-based) or content moderation patterns from ElevenAgents.
- [ ] If we integrate Morgan with telephony or WhatsApp: evaluate ElevenAgents SIP/WhatsApp flows as an alternative or complement.

**References**: [ElevenAgents overview](https://elevenlabs.io/docs/eleven-agents/overview), [Changelog – Custom guardrails, MCP](https://elevenlabs.io/docs/changelog)

---

## 5. Direct Streaming API Spike (Priority: Optional)

**What**: Time-boxed spike to call ElevenLabs streaming TTS directly (bypassing LiveKit Inference) and measure latency.

**Why**: Validate whether a direct path reduces TTS TTFB vs current pipeline.

**Tasks**:
- [ ] Implement a minimal Python client that streams TTS from ElevenLabs WebSocket/streaming API.
- [ ] Measure p50/p95 TTFB for a fixed prompt vs current `elevenlabs` mode via LiveKit.
- [ ] Document findings in `docs/decision-review.md` or a short spike report.
- [ ] Only pursue integration if the gain is meaningful (e.g. >100 ms improvement).

**References**: [API Introduction](https://elevenlabs.io/docs/api-reference/introduction), [ElevenAPI quickstart](https://elevenlabs.io/docs/eleven-api/quickstart)

---

## Summary

| Item                         | Priority | Effort | Dependencies                    |
|-----------------------------|----------|--------|---------------------------------|
| Eleven v3 conversational    | High     | Low    | LiveKit/plugin support          |
| Global routing + 100 ms     | Medium   | Low    | None (or custom client)         |
| Scribe v2 + keyterms        | Low      | Medium | New STT provider path           |
| ElevenAgents patterns       | Future   | Low    | Design decisions                |
| Direct streaming API spike  | Optional | Medium | Python spike, measurements      |

**Recommended order**: Start with (1) Eleven v3; if that’s unavailable or gains are small, run (5) direct streaming spike. Defer (3) and (4) until we have a clear need.
