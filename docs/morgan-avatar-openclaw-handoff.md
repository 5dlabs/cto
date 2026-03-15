# Morgan Avatar / Desktop Handoff

Last updated: March 8, 2026

This is the consolidated handoff for Morgan voice/avatar work across 5dlabs.
It captures:

- work completed
- decisions made and why
- current blockers and open work
- cost and provider notes
- desktop-app direction
- long-conversation architecture for requirement capture

This file is intended to be printable as a single briefing document.

## Executive Summary

The browser-based Morgan avatar proof of concept validated the core path:

- user joins LiveKit room
- Morgan agent joins
- user speech is transcribed
- Morgan text responses are generated through OpenClaw
- ElevenLabs can synthesize voice quickly enough for the current target

The main failures were not raw TTS quality. The real issues were:

- Morgan backend latency and queueing
- transport from generated response back to the client/avatar path
- LemonSlice availability and credits
- room wiring and participant/track visibility during live debugging

Directionally, the work is now shifting into the CTO desktop app. That is the
right move. CTO should own the backend session contract and Morgan transport,
while the avatar layer stays comparatively thin.

## What Has Been Done

## 1. Browser avatar proof of concept

Completed in `/Users/jonathon/5dlabs/cto/avatar`:

- Python LiveKit agent wired for STT, LLM, TTS, and LemonSlice avatar output
- Next.js web app for room join and local test loop
- token route that creates the room and dispatches the agent before join
- client-side room diagnostics and timing panel
- latency logging and summarization tooling

Key files:

- [agent.py](/Users/jonathon/5dlabs/cto/avatar/agent/agent.py)
- [config.py](/Users/jonathon/5dlabs/cto/avatar/agent/morgan_avatar_agent/config.py)
- [providers.py](/Users/jonathon/5dlabs/cto/avatar/agent/morgan_avatar_agent/providers.py)
- [Room.tsx](/Users/jonathon/5dlabs/cto/avatar/web/components/Room.tsx)
- [route.ts](/Users/jonathon/5dlabs/cto/avatar/web/app/api/token/route.ts)

## 2. OpenClaw integration clarified

Morgan is not using the older CTO desktop `/api/chat` session protocol.
Morgan exposes an authenticated OpenAI-compatible endpoint:

- `POST /v1/chat/completions`

The transport requirements were verified:

- `Authorization: Bearer openclaw-internal`
- `x-openclaw-agent-id: morgan`
- model routing consistent with Morgan gateway configuration

This matters because some earlier failures looked like persona drift or silent
timeouts when requests were not correctly routed.

## 3. Desktop backend alignment started

Completed in CTO desktop/Tauri:

- local gateway resolution logic made ingress-first
- desktop client updated to speak the real Morgan gateway contract
- local bootstrap extended for kind + ingress based access
- Morgan local ingress manifest added for `morgan.localhost`

Key files:

- [openclaw.rs](/Users/jonathon/5dlabs/cto/crates/cto-lite/tauri/src/commands/openclaw.rs)
- [install.rs](/Users/jonathon/5dlabs/cto/crates/cto-lite/tauri/src/commands/install.rs)
- [openclaw-morgan-ingress.yaml](/Users/jonathon/5dlabs/cto/crates/cto-lite/tauri/resources/manifests/openclaw-morgan-ingress.yaml)

## 4. Live debugging and regression work

Observed during live tests:

- greeting path worked earlier
- user speech was received and transcribed
- OpenClaw often produced a valid textual response
- response frequently failed to make it back to the avatar/client
- LemonSlice later regressed due to insufficient funds, which prevented avatar
  video participation entirely

To tighten debugging, the following was added:

- clearer LemonSlice startup failure logging
- audio-only fallback when LemonSlice returns insufficient funds
- temporary remote track debug output in the room UI

This let the system continue in audio mode instead of failing completely when
avatar rendering was unavailable.

## Important Decisions

## 1. Keep LiveKit + Morgan + ElevenLabs as the baseline

Decision:

- stay on the current voice stack while fixing reliability

Reason:

- the critical failures were orchestration and transport, not proof that the
  voice providers were fundamentally wrong
- replacing the whole stack before measuring the bottlenecks would increase
  risk and slow delivery

## 2. Treat LemonSlice as optional for debugging

Decision:

- do not let avatar rendering block voice-loop debugging

Reason:

- if LemonSlice credits or availability fail, we still need the audio loop to
  work so we can isolate the actual response path

## 3. Move backend ownership into CTO desktop

Decision:

- desktop app should own Morgan sessioning, gateway transport, and memory

Reason:

- the desktop app is the actual product direction
- CTO can provide a cleaner session contract than the browser PoC
- the avatar should consume backend state, not reinvent it

## 4. Do not optimize around full raw transcript replay

Decision:

- long requirement-capture sessions should use buffered transcript assembly and
  rolling summaries, not full transcript resend on every turn

Reason:

- 5 to 10 minute clips will otherwise create latency, token, and cost blowups

## Current State

As of March 8, 2026:

- Morgan backend contract is understood and partially aligned in CTO
- browser avatar PoC proved the loop in pieces but not yet with stable
  end-to-end reliability
- OpenClaw queue delay is still a meaningful issue
- LemonSlice funding/availability can remove the avatar entirely
- the desktop path is now the correct place to continue active work

## Known Findings

## 1. OpenClaw queue latency is real

Observed logs showed repeated queueing diagnostics such as lane wait exceeded.
That means some "Morgan never replied" failures were actually "Morgan replied
too late for the interactive loop."

Impact:

- stale answers
- interrupted in-flight turns
- silent user experience if the frontend expects a faster response window

## 2. ElevenLabs was not the first bottleneck

Earlier measurements showed TTS first byte was reasonably fast. That means the
first optimization target should not be random TTS provider churn.

Impact:

- keep ElevenLabs in scope
- optimize transport, turn handling, and backend latency first

## 3. LemonSlice failures can mask the real issue

When LemonSlice failed with insufficient funds, the avatar participant never
joined, which made the system look more broken than it actually was.

Impact:

- continue supporting audio-only fallback
- keep avatar rendering separated from response delivery debugging

## Work Outstanding

## 1. Complete desktop app conversation path

Need:

- desktop UI state machine for `idle`, `listening`, `thinking`, `speaking`,
  `error`
- desktop audio capture and push-to-talk or controlled always-listening mode
- desktop submission of turns to Morgan through CTO backend
- desktop playback path for synthesized audio and later avatar video

## 2. Add local Morgan deployment that mirrors production

Need:

- local Morgan deployment path in kind using the same chart family or a clearly
  intentional mirror of production behavior

Avoid:

- treating the older non-production chart as equivalent when it is not

## 3. Improve observability in CTO

Need:

- visible queue depth / gateway status / response timing
- trace of each turn from STT to Morgan response to TTS playback
- structured session diagnostics for failed or slow turns

## 4. Stabilize the return path from Morgan text to client speech

Need:

- explicit instrumentation around the handoff from backend response to speech
  output
- confirmation of where response text is dropped in the desktop path

## 5. Remove temporary debug surfaces when stable

Need:

- remove or gate temporary track debug UI and ad hoc fallback logging after the
  desktop implementation is reliable

## 5dlabs Optimizations To Carry Forward

## 1. Voice-stack optimization

Carry forward:

- keep `eleven_flash_v2_5` as default low-latency TTS baseline
- keep replies concise by default
- avoid long spoken paragraphs unless explicitly requested
- add pronunciation dictionaries for 5dlabs terms like OpenClaw, ClawHub,
  LemonSlice, Morgan, CTO

## 2. Cost optimization

Carry forward:

- use avatar mode only where visual presence matters
- support audio-only flows for routine conversations
- add per-session spend telemetry for synthesized characters, STT usage, and
  session duration

Reference:

- [elevenlabs-cost-savings-and-scope.md](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-cost-savings-and-scope.md)

## 3. Product optimization

Carry forward:

- make CTO the single backend contract for Morgan interactions
- keep avatar rendering as a presentation layer, not the system of record
- persist conversation state centrally so desktop and future clients can share
  the same session truth

## Long-Conversation Strategy

For 5 to 10 minute clips, this should not be treated as a normal short-turn
voice chat problem. It needs a buffered requirement-capture pipeline.

## Recommended architecture

Split into two modes:

- `live conversation`
- `long-form dictation / requirement capture`

For long-form capture:

- continuously capture audio in the desktop app
- stream STT in rolling windows with overlap
- assemble stable transcript segments with timestamps
- maintain a rolling summary and extracted structured memory
- only generate a full Morgan response once a strong end-of-speech signal is
  observed

## Structured memory model

Do not resend the entire raw transcript every turn. Maintain:

- goals
- constraints
- decisions
- open questions
- action items
- recent verbatim tail

That is the right way to preserve fidelity without destroying latency or cost.

## Suggested backend services

These can start in-process and later split if needed:

1. `audio ingestion service`
2. `transcript assembler`
3. `conversation memory service`
4. `response orchestrator`
5. `persistence store`

## Turn handling rules

Recommended first-pass rules:

- pauses under about 1.5 to 2 seconds should be treated as continuation
- transcript should be committed in overlapping windows
- summaries should refresh every 1k to 2k tokens
- final response generation should wait for longer silence, manual stop, or a
  clear end-of-thought signal

## Risks for long conversations

- premature turn finalization
- duplicated transcript from overlap
- summary drift over time
- reprocessing too much raw transcript
- unclear debugging when client and backend disagree on turn boundaries

## ElevenLabs Notes

The key point is that ElevenLabs should be used more deliberately, not
necessarily replaced.

Useful additions still left out of scope:

- pronunciation dictionaries
- Scribe v2 realtime evaluation versus current STT path
- voice cloning / remixing for a stronger Morgan voice identity
- selective future evaluation of ElevenAgents features if orchestration moves
  there

What not to do:

- do not switch providers blindly before measuring where the latency really is

References:

- [elevenlabs-playbook.md](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-playbook.md)
- [elevenlabs-enhancement-plan.md](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-enhancement-plan.md)
- [elevenlabs-cost-savings-and-scope.md](/Users/jonathon/5dlabs/cto/avatar/docs/elevenlabs-cost-savings-and-scope.md)

## Recommended Next Steps

1. Continue active implementation in CTO desktop, not the Next.js web app.
2. Define the desktop conversation contract and UI state machine.
3. Add end-to-end tracing for each turn from microphone to spoken response.
4. Build the buffered long-conversation pipeline for requirement capture.
5. Add local Morgan deployment parity for kind so desktop debugging is not tied
   to ad hoc remote behavior.
6. Reintroduce avatar rendering only after the desktop audio loop is reliable.

## Supporting Documents

- [HANDOFF.md](/Users/jonathon/5dlabs/cto/avatar/docs/HANDOFF.md)
- [decision-review.md](/Users/jonathon/5dlabs/cto/avatar/docs/decision-review.md)
- [provider-spikes.md](/Users/jonathon/5dlabs/cto/avatar/docs/provider-spikes.md)
- [morgan-openclaw-setup.md](/Users/jonathon/5dlabs/cto/avatar/docs/morgan-openclaw-setup.md)

