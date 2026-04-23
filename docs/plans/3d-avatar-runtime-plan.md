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

### Component list

| Component | Role |
|---|---|
| **Client SPA** | Three.js/WebGL 3D viewer (browser or Tauri) |
| **3D asset format** | `.glb` with Draco mesh compression + KTX2 texture compression |
| **Animation system** | Client-side state machine for blending pre-baked animations |
| **Asset host** | CDN for avatar models, textures, animation packs |
| **State sync** | Existing WebRTC transport for real-time state coordination |
| **TalkingHead runtime** | Avatar rendering, viseme/lip sync, gesture orchestration |
| **HeadAudio** | Real-time audio-driven viseme detection |
| **HeadTTS** | Timing/viseme-aware speech integration (optional enhancement) |
| **MotionEngine** | Richer gesture semantics and LLM-driven motion control |

### Data flow

1. Client downloads the 3D viewer SPA and avatar `.glb` from CDN
2. Browser/Tauri loads model + textures into Three.js/WebGL
3. All rendering and animation happen **exclusively on the client device**
4. Existing WebRTC media session provides:
   - live audio streams
   - agent state events (listening/thinking/speaking)
   - TTS audio output
5. HeadAudio derives visemes from the audio stream in real-time
6. TalkingHead maps visemes → mouth blendshapes + triggers body gesture states
7. For multi-avatar scenarios, only small state packets are synced via WebRTC
8. Server responsibilities remain:
   - serving static assets (CDN)
   - relaying minimal avatar state data
   - hosting LLM/STT/TTS providers

### Why server GPU is eliminated from the primary path

- The rendering workload is delegated entirely to the end-user's device
- The server only serves static files and relays tiny state packets
- No real-time per-user GPU rendering is needed on the server side
- Offline avatar creation/retargeting may use GPU, but it's not in the user-facing runtime path
- GPU is reserved only for optional self-hosted model inference or generative fallback R&D

### Top risks

1. **Rig universalism trap** — committing to one skeleton before defining a retarget layer will force a rewrite for animal characters. Ship rig families (biped-human, biped-anthro, quadruped) with shared semantic interface.
2. **WebKitGTK parity** — Linux Tauri builds will differ from Chrome dev experience. Budget testing time on actual Linux WebView.
3. **Memory ceiling** — without enforced asset budgets + LODs + KTX2 from day one, WebView will OOM before visual quality goals are met.
4. **Audio sync drift** — WebAudio clock vs. animation clock drift is real in WebView. Use `AudioContext.currentTime` as master clock, lock animation delta to it.
5. **Cross-device performance** — client hardware is the bottleneck. Aggressive LOD and quality settings are mandatory.
6. **Animal viseme mapping** — non-human faces may not map cleanly to ARKit 52-shape sets. Need per-species blendshape manifests.

### Risk mitigations

- **LOD system**: 3 levels (gltfpack/meshopt); critical for browser memory ceilings (~2 GB practical cap on 32-bit WASM)
- **Texture budget**: KTX2 + BasisU (UASTC for normals, ETC1S for albedo) — 5–10× smaller than PNG
- **Asset versioning**: content-addressed blobs behind manifest; Tauri cache + browser ServiceWorker share strategy
- **CI validation**: headless `gltf-validator` + visual diff on PR
- **Tauri custom protocol**: use `asset://` not `file://` to bypass CSP issues and enable streaming

### Tauri/browser integration specifics

- **Renderer choice:** Three.js + `three-vrm` or Babylon.js; **avoid Unity/Unreal WebGL builds** in Tauri (50+ MB runtime, WebGL2 quirks on macOS WKWebView)
- **WKWebView (macOS Tauri):** no WebGPU until macOS 14+ stable; WebGL2 only — plan shader complexity accordingly. KTX2 transcoding via Basis WASM works but adds ~500 KB
- **Linux WebKitGTK:** WebGL2 conformance gaps (especially NVIDIA proprietary drivers); test on Ubuntu 22.04 + Fedora; provide canvas2D fallback message, not broken render
- **Asset loading:** Tauri `asset://` protocol required for local `.glb` (CSP blocks `file://`); whitelist scope in `tauri.conf.json`; stream large assets via `tauri::api::http` rather than embedding in bundle (keeps installer <30 MB)
- **WASM size budget:** Draco + Basis + physics ≈ 1.5 MB WASM cold start; lazy-load post-window-paint to keep TTI <2 s
- **Offscreen canvas:** WKWebView lacks `OffscreenCanvas` in workers — main-thread render only on macOS; budget frame time accordingly (target 16 ms with headroom)
- **Audio sync drift:** `AudioContext.currentTime` vs `requestAnimationFrame` clock drift causes viseme desync >50 ms; drive blendshape timeline from audio clock, not RAF
- **Memory pressure:** Tauri webview shares process memory with Rust side; one 25 MB `.glb` decoded ≈ 150 MB GPU+CPU. Cap loaded avatars at 1 active + 1 preloaded; explicit `dispose()` on swap
- **Codesigning/notarization:** large WASM blobs trigger longer macOS notarization; pre-compile and ship as separate sidecar resource, not embedded in main binary
- **Hot-reload of avatars:** Three.js GPU resource leaks on `.glb` swap — wrap in disposal helper; leak detector in dev builds
- **CSP:** `wasm-unsafe-eval` required for Basis transcoder; document and scope tightly to avatar origin only

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

### Asset format and pipeline
- **Source format:** **glTF 2.0 (.glb)** — single-file, PBR-native, broad WebGL support; reject FBX at runtime boundary (license + bloat)
- **Authoring flow:** DCC (Blender) → FBX/USD intermediate → glTF with KHR extensions (`KHR_draco_mesh_compression`, `KHR_materials_*`, `KHR_mesh_quantization`, `KHR_texture_basisu`)
- **Texture budget:** KTX2 + BasisU (UASTC for normals, ETC1S for albedo) — 5–10× smaller than PNG, GPU-decoded
- **LODs:** 3 levels per character — LOD0 ~30k tris (head-focused), LOD1 ~12k, LOD2 ~5k; auto-generate via `gltfpack`/`meshoptimizer` in CI
- **Texture budget per asset:** 2K base color / 1K normal+ORM per region (head, body, hair); atlas where possible; enforce **≤25 MB per character `.glb`** post-compression
- **Versioning:** content-addressed storage (CID) for `.glb` + manifest (`avatar.json`) declaring rig type, blendshape set, LOD map, and license
- **CI validation gate:** headless `gltf-validator` + custom linter checking bone names, blendshape names, UV channels, material count; fail PR on schema drift
- **Visual diff:** Playwright + WebGL screenshot on PR for regression detection
- **VRM compatibility:** keep as later extension if useful for ecosystem

### Rig and blendshape requirements
- **Skeleton standard:** VRM 1.0 (humanoid-bone-mapped) or Mixamo/Unity Humanoid rig
- **Bone naming contract:** Mixamo/VRM-compatible humanoid base (Hips, Spine, Spine1, Spine2, Neck, Head + standard arms/legs); **≤80 bones** for mobile/Tauri perf
- **Naming enforcement:** lowerCamelCase ARKit names exactly (`browInnerUp`, not `Brow_Inner_Up`); validate in CI
- **Blendshapes (morph targets):**
  - **ARKit 52-shape standard (mandatory):** full ARKit shape set so any audio-driven viseme/emotion model (Audio2Face, NVIDIA ACE, Resemble, custom) is plug-in compatible
  - **Oculus visemes (15):** sil, PP, FF, TH, DD, kk, CH, SS, nn, RR, aa, E, I, O, U — pipeline converts between systems
  - Keep morph target count ≤ 60 active per draw call (WebGL2 attribute limit)
- **Eye rig:** dedicated `LeftEye`/`RightEye` bones (not blendshape-only) for gaze IK + saccades; jaw bone separate from `jawOpen` blendshape (additive layering)
- **Tongue:** at least 3 tongue blendshapes (`tongueOut`, `tongueUp`, `tongueCurl`) — frequently missing in cheap assets, blocks realistic dental visemes
- **Inverse kinematics:** two-bone IK at runtime (Three.js `CCDIKSolver` / Babylon `IKController`); don't bake
- **Bone count cap:** ≤60–80 bones; cloth/hair via spring-bone constraints, not extra skinned bones

### Humanoid ↔ animal compatibility
- **Shared rig spec:** define an "anthro humanoid" profile — quadruped-derived characters must be re-rigged to bipedal Mixamo skeleton; tail/ears/wings as **extension bones** under a `aux_*` namespace the runtime can ignore safely
- **Muzzle/snout problem:** ARKit blendshapes assume flat human face; for muzzled species, author **dual blendshape sets**:
  - `arkit_*` (semantic, retargetable) — driven by lip-sync model
  - `species_*` (corrective deltas applied additively) — mask jaw/lip geometry for animal faces
- **Ear/tail secondary motion:** runtime jiggle physics (spring bones, VRM `SpringBone` style) rather than baked anim, so it composes with body anim
- **Retargeting:** all animation clips authored on a single reference rig; runtime retargets to species rig via bone-name map. Avoid per-character anim libraries
- **Proportions:** non-human limb-length ratios break IK; ship per-character IK calibration in `avatar.json` (shoulder width, arm length, eye offset)
- **Risk:** single "universal" rig is a trap — ship **rig families** (biped-human, biped-anthro, quadruped-mammal, quadruped-avian) with shared semantic interface
- **Quadruped gait:** procedural foot-placement (raycast IK) is more compatible than baked clips; one system serves both bipeds and quadrupeds

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
