# 3D Avatar Runtime Plan (Option A)

## Decision

We will implement the primary live avatar experience as a **deterministic 3D avatar runtime**, not a server-side generative video system.

This is the right architecture for our actual requirements:

- real-time voice-agent interaction
- basic gestures, not dancing/choreography
- lip sync
- humanoid human and humanoid-animal characters
- cost-efficiency
- operational robustness
- compatibility with our existing WebRTC/media stack

## Why this wins

Compared with runtime video generation, this path is:

- cheaper to operate
- lower latency
- more robust in production
- more consistent in avatar identity
- better suited to humanoid animal agents
- less dependent on scarce GPU inference capacity

The main tradeoff is that we need a stronger asset pipeline for rigs, visemes, blendshapes, and basic animation packs.

## Core architecture

### Runtime model

At runtime, we do **not** generate fresh video frames with a GPU model.

Instead, we:

1. load a rigged 3D avatar in the client
2. play basic pre-authored animations
3. drive mouth/face movement from audio or viseme data
4. switch body/expression states based on agent state
5. render live in browser/Tauri via GPU/WebGL on the client device

This makes the avatar layer behave more like a game/VTuber runtime than a diffusion video pipeline.

## Primary component choices

### Media transport
- Existing **WebRTC** stack
- Existing live audio/session infrastructure
- LiveKit-style transport remains optional infrastructure, not the avatar engine

### Language model
- Existing hosted LLM providers
- No self-hosted LLM required for the avatar runtime plan

### Speech-to-text
- Existing hosted or already-integrated real-time STT path
- Requirement: streaming or near-streaming transcription for live conversation

### Text-to-speech
- Hosted TTS provider
- Requirement: low-latency audio output
- Preferred enhancement: word timings, phoneme timings, visemes, or blendshape metadata when available

### Avatar runtime
- **TalkingHead** as the primary runtime foundation
- Add-on modules to evaluate/use:
  - **HeadAudio** for real-time audio-driven viseme detection
  - **HeadTTS** for timing/viseme-aware speech integration
  - **MotionEngine** for richer gesture semantics when useful

### Asset format
- Standardize on **GLB first**
- Keep VRM compatibility as a possible later extension if helpful

### Client targets
- browser
- Tauri desktop app

## Lip sync approach

### Primary path
Use **HeadAudio** or equivalent audio-driven lip-sync logic as the default path.

Why:

- reduces TTS vendor lock-in
- keeps lip sync working even when provider-specific viseme/timestamp support changes
- simplifies provider switching

### Optional enhancement path
If the chosen TTS provider exposes word timings, phoneme timings, visemes, or blendshape data, use that as an enhancement layer.

## GPU requirements

### No server GPU required for the primary avatar runtime
The following do **not** require server GPU in the primary path:

- avatar rendering runtime
- lip sync playback
- basic gestures
- idle/speaking/listening states
- orchestration/session handling
- asset serving

### Client-side graphics still apply
The client device will use normal local graphics acceleration where available:

- browser WebGL/WebGPU
- Tauri/webview graphics stack

### Where GPU may still be used
GPU is only needed for:

- optional asset generation workflows
- optional self-hosted STT/TTS/LLM later
- optional generative avatar/video fallback R&D

## What we already have

- existing WebRTC/live media setup
- hosted LLM access
- hosted model provider routing
- working direction toward a voice-agent architecture
- agreement that a combined 3D stack is the right path

## What we still need

### 1. Runtime proof of concept
A minimal end-to-end avatar session that:

- loads one avatar
- joins an existing media/session flow
- plays speech audio
- performs lip sync
- switches between idle/listening/speaking states

### 2. Gesture behavior layer
A small deterministic state machine for:

- idle
- listening
- thinking
- speaking
- basic emotional variations

We do **not** need advanced choreography.

### 3. Asset pipeline
We need a standard for:

- humanoid rig requirements
- facial blendshapes / viseme targets
- required expression set
- basic animation pack
- validation for human and humanoid-animal characters

### 4. Voice-agent integration
We need a clean integration boundary between:

- media session state
- STT/TTS
- agent state
- avatar runtime state

## Proposed execution model

### Phase 1 — Runtime proof
Goal: get one avatar speaking in real time.

Deliverables:
- TalkingHead-based test app
- one rigged avatar
- audio playback
- lip sync
- idle/listening/speaking states
- connection to existing media/session flow

Success criteria:
- stable real-time conversation demo
- believable lip sync
- no server GPU requirement

### Phase 2 — Basic behavior
Goal: make the avatar feel alive enough for real use.

Deliverables:
- basic gesture state machine
- blink/gaze/head movement
- small posture shifts
- neutral/happy/concerned/excited/thoughtful expression mapping

Success criteria:
- more than just mouth movement
- behavior feels intentional, not noisy

### Phase 3 — Asset standardization
Goal: make new agent avatars repeatable to add.

Deliverables:
- avatar asset spec
- rig/blendshape checklist
- import validation checklist
- compatibility rules for humanoid-animal avatars

Success criteria:
- repeatable onboarding of new avatars without custom engineering each time

### Phase 4 — Full integration
Goal: connect the avatar runtime cleanly to agent state and shipping clients.

Deliverables:
- avatar session protocol
- agent-state-to-avatar-state mapping
- Tauri/browser integration validation
- metrics and failure handling

Success criteria:
- production-shape architecture in place
- graceful degradation when assets/audio/timing metadata are imperfect

## Provider constraints and fit

This plan is specifically designed to work with our current constraints:

- continue using hosted LLMs
- continue using hosted STT/TTS where practical
- do not depend on OVH AI Deploy or DigitalOcean GPU droplets for the primary live avatar runtime
- reserve GPU-backed avatar/video models for optional future fallback or research paths

This is a better match for current infrastructure and budget discipline.

## What is explicitly out of the primary path

The following are **not** part of the main runtime architecture:

- HunyuanVideo-Avatar as live runtime
- MuseTalk as primary avatar system
- LiveAvatar as primary runtime
- runtime server-side video generation
- GPU-backed avatar serving on OVH/DO for the normal interactive path

These remain optional R&D or fallback paths only.

## CLI / agent execution plan

When executing implementation work, prefer CLIs we have already verified as functional in this environment.

### Verified working CLI paths
- **Gemini CLI** — working
- **GitHub Copilot CLI** — working

### Conditionally available
- **Claude Code** — configured but currently rate-limited in this environment

### Not reliable right now
- **OpenCode** — configured models visible, but live inference broken and DigitalOcean provider absent
- **Kimi CLI** — installed, but no usable model configured
- **Cursor Agent** — installed, not authenticated

### Recommended execution routing
- **Gemini ACP / subagent**: architecture and implementation-plan drafting, documentation synthesis, component-boundary analysis
- **Copilot ACP / subagent**: asset-pipeline checklisting, browser/Tauri integration risks, implementation detail review
- **Claude Code ACP / subagent**: use once quota is available for repo-aware implementation tasks and code changes

## Parallelization plan

When we move from planning to implementation, split work into parallel tracks:

1. **Runtime integration track**
   - TalkingHead integration
   - WebRTC/media session hook-up
   - lip sync wiring
   - owner: working ACP coding CLI

2. **Asset pipeline track**
   - avatar requirements
   - viseme/blendshape checklist
   - humanoid-animal compatibility validation
   - owner: working ACP coding CLI

3. **Behavior/orchestration track**
   - idle/listening/speaking state machine
   - gesture mapping
   - emotion mapping
   - owner: working ACP coding CLI

4. **Client integration track**
   - browser/Tauri validation
   - performance and degradation paths
   - owner: working ACP coding CLI

## Coordination note

We should coordinate implementation with subagents/ACP runs in parallel, but keep the plan itself explicit about which CLI/agent is assigned to each track so execution and verification are easy to follow.

## Recommendation

Lock **Option A** as the primary avatar implementation strategy.

Use a deterministic 3D runtime as the production path, and treat generative avatar/video systems as optional future enhancements rather than the core experience.
