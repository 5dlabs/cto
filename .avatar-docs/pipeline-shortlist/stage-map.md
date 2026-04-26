# Stage Map

Use this as the working shortlist for the next implementation/research pass. Paths are local to `.avatar-docs/`.

## Stage 0: Runtime contract spike

Goal: prove that the browser can render, lip-sync, gesture, and stay in sync with LiveKit before custom Morgan generation.

| Candidate | Local/reference path | Handles | Why it is shortlisted |
| --- | --- | --- | --- |
| `met4citizen/TalkingHead` | `repos/met4citizen__TalkingHead` | Browser runtime: Three.js GLB avatar, lip sync, mood, animation, Mixamo/ARKit/Oculus contract | Closest direct match. README says it supports full-body GLB avatars with Mixamo-compatible rig and ARKit + Oculus viseme blendshapes. |
| `met4citizen/HeadTTS` | `repos/met4citizen__HeadTTS` | Browser or Node TTS with phoneme timestamps + Oculus visemes | Useful if Morgan TTS can be generated locally or through a compatible TTS bridge. |
| `met4citizen/HeadAudio` | referenced by `TalkingHead` README, not cloned | AudioWorklet PCM-to-viseme path | Strong LiveKit fit because it can drive visemes from streaming audio without text/timestamps. Needs verification/clone in next pass. |
| `lhupyn/motion-engine` | referenced by `TalkingHead` README, not cloned | Semantic gesture/expression layer | Candidate for LLM-driven gestures and moods once the base runtime works. Needs verification/clone in next pass. |
| `Seed3D/Puppeteer` | `repos/Seed3D__Puppeteer` | Puppeteering/control reference | Worth reading for control abstractions and animation state ideas, not necessarily first runtime dependency. |

Runtime contract to validate:

- full-body GLB or VRM loads in browser
- Mixamo-compatible or VRM humanoid bone naming
- ARKit-style facial expression blendshapes
- Oculus/viseme blendshapes or a mapping from visemes to morph targets
- idle/listen/speak/gesture animation clips
- LiveKit audio track can feed viseme timing

## Stage 1: Preprocessing and reference cleanup

Goal: prepare clean Morgan references before generation.

| Candidate | Local/reference path | Handles | Why it is shortlisted |
| --- | --- | --- | --- |
| `initml/sam-hq` | `repos/initml__sam-hq` | high-quality segmentation / masking | Good for foreground masks and crop cleanup. |
| HF `briaai/RMBG-2.0` | `index/huggingface-models.json` | background removal | Strong preprocessing candidate. |
| HF `ZhengPeng7/BiRefNet_demo` / BiRefNet family | `index/huggingface-models.json` | background/foreground separation | Useful if SAM-HQ/RMBG misses hair/edges. |
| HF CodeFormer / GFPGAN rows | `index/huggingface-models.json` | face restoration | Use cautiously for touch-up after identity is locked. |

## Stage 2: 3D base asset generation

Goal: generate candidate Morgan head/body geometry and textures.

| Candidate | Local/reference path | Handles | Why it is shortlisted |
| --- | --- | --- | --- |
| `VAST-AI-Research/AniGen` | `repos/VAST-AI-Research__AniGen` | one-shot image to animatable/rigged 3D asset | Highest-signal shortcut if it works; mesh preview may be imperfect but skeleton/skinning can still be useful. |
| HF `VAST-AI/AniGen` | `index/huggingface-models.json` | hosted model/space reference | Useful to compare UI/demo output vs headless/local output. |
| `VAST-AI-Research/TripoSG` | `repos/VAST-AI-Research__TripoSG` | high-fidelity 3D shape synthesis | Candidate geometry source before rigging. |
| `VAST-AI-Research/TripoSF` | `repos/VAST-AI-Research__TripoSF` | sparse/high-res arbitrary-topology shape modeling | Candidate mesh-generation/reference path. |
| `VAST-AI-Research/TripoSR` | `repos/VAST-AI-Research__TripoSR` | fast image-to-3D reconstruction | Baseline/fast iteration candidate. |
| HF Hunyuan3D rows | `index/huggingface-models.json` | image/text-to-3D model candidates | Strong alternative family; likely worth benchmarking with our own GPU. |
| HF TRELLIS rows | `index/huggingface-models.json` | image-to-3D generation | Strong Microsoft/TRELLIS candidate; benchmark alongside Hunyuan/AniGen/Tripo. |
| Scenario Tripo/Hunyuan apps | `index/scenario-models.json` | managed 3D generation, retopo, rigging | Good external comparison and possible paid shortcut. |

Quality rule:

- If identity/proportions/base geometry are wrong, rerun or tune generation.
- If topology/materials/weights are weak but base form is good, post-process.

## Stage 3: Rigging, skinning, facial controls

Goal: produce the actual runtime asset, not just a mesh.

| Candidate | Local/reference path | Handles | Why it is shortlisted |
| --- | --- | --- | --- |
| `VAST-AI-Research/SkinTokens` | `repos/VAST-AI-Research__SkinTokens` | skeleton hierarchy + dense skin weights from mesh | Best current auto-rigging signal; likely successor to UniRig. |
| `VAST-AI-Research/UniRig` | `repos/VAST-AI-Research__UniRig` | automatic rigging baseline | Proven baseline; compare against SkinTokens. |
| HF UniRig/SkinTokens rows | `index/huggingface-models.json` | hosted/repacked rigging references | Useful for availability and reproducing model artifacts. |
| `makehumancommunity/makehuman` | `repos/makehumancommunity__makehuman` | base human topology/proportions/reference rig | Fallback if generated geometry needs human template wrapping. |
| Scenario Tripo Rigging / Retopology | `index/scenario-models.json` | managed skeleton, skinning, retopo | Possible paid shortcut for biped rigging and cleanup. |

Important warning:

Body rigging is not facial performance. SkinTokens/UniRig/AniGen can help with skeleton and body skinning, but Morgan still needs a face layer: ARKit-style blendshapes, Oculus visemes, or a compatible facial rig. That may require FLAME/SMPL-X/ARKit transfer, template wrapping, Ready Player Me/VRoid-style service, or a dedicated face authoring pass.

## Stage 4: Body motion, gestures, and behavior

Goal: make Morgan feel alive after the runtime contract works.

| Candidate | Local/reference path | Handles | Why it is shortlisted |
| --- | --- | --- | --- |
| Mixamo animations | external service / TalkingHead-compatible | idle, listen, gestures, full-body animation clips | TalkingHead expects Mixamo-compatible rigs/animations. |
| `lhupyn/motion-engine` | referenced by TalkingHead README | semantic gesture/expression layer | Could map LLM/tool events to gestures. |
| `Seed3D/Puppeteer` | `repos/Seed3D__Puppeteer` | puppeteering/control ideas | Research reference for control flow. |
| HF HY-Motion rows | `index/huggingface-models.json` | 3D motion/animation generation | Potential later enhancement after static asset exists. |

## Stage 5: Plan B / hero video generation

Goal: keep high-quality talking portrait output available for demos, marketing, and fallback.

| Candidate | Local/reference path | Handles | Why it remains useful |
| --- | --- | --- | --- |
| `Fantasy-AMAP/fantasy-talking` | `repos/Fantasy-AMAP__fantasy-talking` | high-quality image+audio talking portrait video | Strong Plan B / hero clip lane. |
| `fudan-generative-vision/hallo2` | `repos/fudan-generative-vision__hallo2` | audio-driven portrait video | Explicitly included; good modern Plan B candidate. |
| `TMElyralab/MuseTalk` | `repos/TMElyralab__MuseTalk` | realtime-ish lipsync/video | Useful fallback if server-side video is needed. |
| `KwaiVGI/LivePortrait` | `repos/KwaiVGI__LivePortrait` | portrait animation / motion transfer | Useful for generated clips and comparison. |
| `ByteDance/LatentSync` | `repos/ByteDance__LatentSync` | video lip-sync refinement | Useful after a Morgan video already exists. |
| `antgroup/echomimic_v3` | `repos/antgroup__echomimic_v3` | audio-driven portrait generation | Modern hero-video candidate. |
| `OpenTalker/SadTalker`, `Rudrabha/Wav2Lip` | local repo dirs | older talking-head/lipsync baselines | Keep as baselines, not first-pass runtime path. |
