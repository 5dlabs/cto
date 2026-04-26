# Research Command Prompt

The next research pass should search for the missing/runtime-specific layer, not more generic talking-head generators.

```text
/research Deeply search GitHub and the web for modern, maintained, high-star repositories and docs for a browser-based LiveKit/WebRTC avatar runtime using GLB/VRM assets. Focus on TypeScript/JavaScript/WebGL/WebGPU stacks that can drive a humanoid avatar from streaming audio and/or TTS text. Required concepts and synonyms: GLB avatar runtime, VRM avatar runtime, Three.js avatar, Babylon.js avatar, Mixamo-compatible rig, ARKit 52 blendshapes, Oculus visemes, viseme timeline, audio-to-viseme, PCM AudioWorklet lip sync, WebRTC avatar, LiveKit avatar, talking avatar client side, morph targets, blendshape transfer, SMPL-X/FLAME/ARKit face rig, retargeting, idle gestures, Mixamo animation blending, Ready Player Me SDK, VRoid/VRM, Rhubarb Lip Sync, OVR LipSync, Azure Speech visemes, ElevenLabs timestamps.

Compare against our current shortlisted references in .avatar-docs/pipeline-shortlist:
- met4citizen/TalkingHead
- met4citizen/HeadTTS
- met4citizen/HeadAudio
- lhupyn/motion-engine
- VAST-AI-Research/SkinTokens
- VAST-AI-Research/UniRig
- VAST-AI-Research/AniGen
- Hunyuan3D / TRELLIS / TripoSG / TripoSF / TripoSR

Find candidates that are recent or actively maintained, preferably 2025/2026 updates, popular, production-used, or clearly documented. For each candidate, capture:
1. repo/doc URL
2. stars and last commit/update
3. language/runtime
4. supported avatar format (GLB, glTF, VRM, RPM, FBX, Live2D, Rive, etc.)
5. lip-sync input (audio PCM, TTS visemes, word timestamps, phonemes, text only)
6. facial control standard (ARKit, Oculus visemes, VRM expressions, custom morphs)
7. body animation support (Mixamo, VRM humanoid, retargeting, animation mixer)
8. LiveKit/WebRTC integration fit
9. license/commercial concerns
10. whether it should be shortlisted, archived, or benchmarked.

Also search specifically for provisioning/authoring tools that convert generated meshes into runtime-ready avatars:
- ARKit blendshape transfer to custom head mesh
- FLAME or SMPL-X fitting to generated face/body
- VRM export pipeline from Blender
- GLB retopology + rigging + morph target generation
- Ready Player Me or VRoid export automation
- automatic viseme/morph target authoring

Do not focus on pure server-side video synthesis unless it directly helps with fallback/hero video generation. Return a concise shortlist and a separate archive list.
```
