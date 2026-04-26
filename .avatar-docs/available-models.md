# Morgan avatar available model catalog

This catalog tracks models and service offerings that could participate in the
Morgan avatar provisioning pipeline. It is intentionally broader than the
current open-source path so we can swap in paid providers when credits or budget
are available.

Sources used:

- Scenario MCP `recommend` calls for image-to-3D, text-to-3D, image editing,
  image-to-video/talking-avatar fallback, and a full gated avatar workflow plan.
- Scenario MCP model search/list calls for public avatar, 3D, rigging, texture,
  lip-sync, and video models.
- Existing local catalogs in `.avatar-docs/index/scenario-models.json` and
  `.avatar-docs/index/huggingface-models.json`.

Scenario's public catalog includes many generic style LoRAs and art-direction
models. Those are excluded unless they map directly to this avatar pipeline.

## Scenario recommendation signals

| Pipeline stage | Scenario recommendation signal | Practical meaning for us |
| --- | --- | --- |
| Image/reference prep | `model_google-gemini-3-1-flash` | Scenario recommends Gemini 3.1 Flash for image editing/reference workflows with many reference images. Use for source cleanup, reference-sheet generation, texture/skin/fur prompt refinement, and QA images. |
| Image-to-3D | `model_hunyuan-3d-pro-3-1-i23d` | Scenario's default quality recommendation for image-to-3D is Hunyuan 3D 3.1 Pro, with Tripo P1 noted for game-ready low-poly. This is the top paid benchmark after the open-source AniGen pass. |
| Text-to-3D | Meshy Text-to-3D, Rodin Gen-1/2 | Scenario recommends image-first, but if text-to-3D is required, Meshy and Rodin are the main paid candidates. Rodin has T/A-pose and PBR options that matter for rigging. |
| Video fallback | `model_kling-v3-i2v-pro` | Scenario's quality default for image-to-video is Kling V3 I2V Pro; it supports elements, end frames, generated audio, and 3-15s clips. Use only for hero/fallback videos, not the runtime GLB path. |
| Full workflow plan | Gemini-led reference workflow | Scenario's plan call mapped our request to a character-sheet/reference workflow and suggested a talking-head fallback. That reinforces our DAG: source/reference quality first, then 3D, then rig/face/runtime. |

## Model table

| Model name | Provider | Open source | Should we use it? | Service offerings if not open source |
| --- | --- | --- | --- | --- |
| AniGen | VAST-AI Research | Yes, with license caveats | **Yes, first open-source head/bust benchmark.** It is the current primary path because it emits skinned GLB-like assets and we already have a local preview to diagnose. | N/A. Self-host on DigitalOcean GPU Droplet, RunPod, Modal, OVH, or Hugging Face GPU Space. |
| Hunyuan 3D 3.1 Pro | Tencent / Hunyuan | No | **Yes, paid benchmark after AniGen or if AniGen fails geometry/material quality.** Scenario recommends it as the default image-to-3D quality model. | Scenario, Tencent/Hunyuan ecosystem if directly accessible, possible partner APIs. |
| Hunyuan3D 2 / 2.1 | Tencent / Hunyuan | Yes | **Yes, open-source fallback/benchmark.** Useful if we want self-hosted Hunyuan-style generation without paying Scenario. | N/A. Self-host; Hugging Face models/spaces; DigitalOcean/RunPod/Modal GPU. |
| Tripo 3.1 | Tripo AI | No | **Yes, paid fallback for game-ready 3D mesh output.** Scenario notes PBR textures and multi-view inputs; useful if AniGen identity or material quality is weak. | Scenario, Tripo API/platform. |
| Tripo 3.0 | Tripo AI | No | **Maybe.** Good cheaper/free-tier Scenario baseline, but 3.1 is preferred where available. | Scenario, Tripo API/platform. |
| Tripo P1 | Tripo AI | No | **Maybe, for low-poly/game-ready route.** Scenario recommendation mentioned it as the game-ready low-poly option. | Scenario and/or Tripo platform. |
| Tripo Rigging 1.0 | Tripo AI | No | **Yes if mesh is good but rigging/skinning is weak.** Directly maps to the biped auto-rigging stage. | Scenario, Tripo platform. |
| Tripo Retopology | Tripo AI | No | **Yes if generated meshes are too dense or unsuitable for realtime.** This is a targeted repair stage, not a first pass. | Scenario, Tripo platform. |
| Meshy Text-to-3D | Meshy | No | **Maybe, text-to-3D fallback only.** Scenario recommends image-first for quality; Meshy is useful for quick concept volume or non-Morgan props. | Scenario, Meshy API/platform. |
| Rodin Gen-1 / Hyper3D | Deemos / Rodin | No | **Maybe.** Use when paid text/image-to-3D alternatives are needed; T/A-pose and PBR options are relevant. | Scenario, Rodin/Hyper3D platform, Blender Hyper3D integration if credentials are available. |
| Rodin Gen-1 HighPack | Deemos / Rodin | No | **Maybe, higher-quality Rodin path.** Use only if Rodin standard looks promising. | Scenario, Rodin/Hyper3D platform. |
| Rodin Gen-2 | Deemos / Rodin | No | **Maybe, paid fallback.** Scenario notes quality tiers including quad and high-poly triangle outputs. | Scenario, Rodin/Hyper3D platform. |
| TRELLIS / TRELLIS.2 | Microsoft / TRELLIS community | Yes | **Yes, fallback mesh generator.** Useful if AniGen fails, especially for object/character geometry experiments. | N/A for open-source; Hugging Face Spaces, self-host on GPU; possible Azure route if Microsoft hosts an offering later. |
| TripoSR | Stability AI / TripoSR community | Yes | **Benchmark only.** Fast open-source baseline, likely weaker than Hunyuan/Tripo/AniGen for final Morgan. | N/A. Hugging Face/self-host. |
| InstantMesh | TencentARC/community | Yes | **Benchmark/archive candidate.** Useful for comparison and multiview experiments, but not primary. | N/A. Hugging Face/self-host. |
| Wonder3D | xxlong/community | Yes | **Benchmark/archive candidate.** Older image-to-3D reference; likely not the best 2026 route. | N/A. Hugging Face/self-host. |
| UniRig | VAST-AI Research/community | Yes | **Yes if skeleton/weights are weak.** Candidate rigging/skinning layer after a good mesh exists. | N/A. Hugging Face/self-host. |
| SkinTokens | VAST-AI Research | Yes | **Maybe.** Keep as rig/skinning/body-representation research path, not first production pass. | N/A. Hugging Face/self-host. |
| Blender + Python automation | Blender Foundation | Yes | **Yes.** Required for validation, material baking, retopology glue, GLB normalization, and possibly blendshape transfer. | N/A. Self-host/desktop/headless runner. |
| Faceit / Blender face-authoring tools | Faceit / Blender ecosystem | Mixed | **Yes if head is good but lacks facial controls.** This is likely a bounded post-processing stage. | Blender marketplace/tools; local Blender automation. |
| ARKit 52 blendshape transfer | Apple standard / tooling ecosystem | Mixed | **Yes as a target contract.** Not a model, but the best facial-control standard for broad tooling. | Apple ecosystem, Reallusion/Character Creator, Blender add-ons, custom transfer tooling. |
| Oculus viseme transfer | Meta/Oculus standard / tooling ecosystem | Mixed | **Yes as a runtime lip-sync contract.** Use for browser/runtime mouth control if we can author morphs. | Meta/Oculus ecosystem, Rhubarb/OVR-style tooling, custom transfer. |
| VRM expressions / three-vrm | VRM consortium / OSS ecosystem | Yes | **Yes for runtime compatibility if we choose VRM.** Strong browser fit, but generation pipeline must author compatible expressions. | N/A. Self-host/browser. |
| Ready Player Me | Ready Player Me | No | **Maybe as template fallback.** If generated torso fails, use a template body/rig and graft/approximate Morgan head/style. | Ready Player Me platform/API. |
| VRoid Studio / VRM export | pixiv / VRM ecosystem | Mixed | **Maybe as template fallback.** Good for VRM-compatible humanoid bodies, less ideal for Morgan identity. | VRoid Studio, VRM ecosystem. |
| MakeHuman / MPFB | MakeHuman / Blender ecosystem | Yes | **Maybe as torso fallback.** Useful for humanoid rig/body if the head pipeline works separately. | N/A. Self-host/Blender. |
| Gemini 3.1 Flash | Google | No | **Yes for source conditioning and texture/reference prep.** Scenario recommends it for image editing/reference workflows. | Scenario, Google Vertex AI/Gemini API. |
| Gemini / Imagen family | Google | No | **Yes, later for reference/image generation and texture cleanup.** Use credits before paying Scenario where possible. | Google Vertex AI, Google AI Studio, Scenario wrappers. |
| Flux.1 Kontext / Kontext LoRAs | Black Forest Labs / Scenario | No | **Maybe for expression/reference sheets.** Scenario search surfaced facial-expression sheet models; useful for image reference QA, not GLB generation. | Scenario, BFL partners/APIs, possibly Replicate/Fal depending availability. |
| Photoroom background removal | Photoroom | No | **Maybe.** Scenario workflows use it; we can substitute OSS RMBG/BiRefNet first. | Photoroom API, Scenario workflows. |
| RMBG-2.0 | BRIA AI | Yes-ish / open weights | **Yes for background removal baseline.** Better than our current geometric placeholder masks. | Hugging Face, BRIA platform; self-host if license permits. |
| BiRefNet | ZhengPeng/community | Yes | **Yes for high-quality subject isolation.** Good candidate to replace placeholder alpha masks. | Hugging Face/self-host. |
| SAM2 | Meta | Yes | **Maybe for segmentation refinement.** Useful if RMBG/BiRefNet fails on Morgan fur/ears. | Hugging Face/self-host; Meta ecosystem. |
| Sapiens pointmap | Meta/Facebook | Research/open | **Maybe later.** Could help body/pose/geometry understanding, but not a primary avatar generator. | Hugging Face/self-host. |
| SeedVR2 | ByteDance/Seed ecosystem | No | **Maybe for video/reference restoration.** Scenario low-poly character app mentions it; use only if image/video quality needs repair. | Scenario, ByteDance/Seed partners if available. |
| P-Image Edit / Pruna AI | Pruna AI / Scenario | No | **Maybe for image edit/cleanup.** Scenario sketch-to-render uses it; Gemini likely covers this for us first. | Scenario, Pruna AI offerings. |
| Kling V3 I2V Pro | Kling / Kuaishou | No | **Yes as paid video fallback/hero generator, not GLB runtime.** Scenario recommends it as quality default for image-to-video. | Scenario, Kling platform/API if accessible. |
| Kling V3 I2V Standard | Kling / Kuaishou | No | **Maybe for cheaper video iteration.** Use when Pro is too expensive. | Scenario, Kling platform/API. |
| Kling V3 Omni Video | Kling / Kuaishou | No | **Maybe for broader scene/video generation.** Scenario uses it to add characters into video. | Scenario, Kling platform/API. |
| Kling AI Avatar 2 Pro | Kling / Kuaishou | No | **Maybe for talking-video benchmark.** Good quality comparison if we need MP4 output. | Scenario, Kling platform/API. |
| Kling Lipsync | Kling / Kuaishou | No | **Maybe for redubbing existing video clips.** Not useful for GLB runtime controls. | Scenario, Kling platform/API. |
| Veo 3.1 | Google | No | **Maybe for hero/fallback video with native audio.** Expensive, but we may have Google credits. | Google Vertex AI/Veo, Scenario. |
| LTX 2.3 Pro / Fast | Lightricks | Mixed/open model family | **Maybe for video fallback.** Scenario lists strong image/video-to-video options; useful if open/self-host variant is accessible. | Scenario, Lightricks ecosystem, Hugging Face/self-host for open LTX variants. |
| LTX-2 19B / Fast | Lightricks | Mixed/open model family | **Maybe for longer video fallback.** Not a runtime avatar solution. | Scenario, Lightricks ecosystem, Hugging Face/self-host for open variants. |
| LTX-2.3 AV LoRA talking-head | Community / Lightricks base | Yes/openrail | **Maybe as open talking-head video fallback.** Hugging Face candidate with high downloads. | Hugging Face/self-host. |
| HeyGen Avatar 4 | HeyGen | No | **Maybe for paid talking-avatar benchmark only.** Useful as a quality bar, not our runtime asset path. | Scenario, HeyGen platform/API. |
| Veed Fabric Lipsync 1.0 | VEED | No | **Maybe for paid lip-sync video benchmark.** MP4 fallback only. | Scenario, VEED/Fabric. |
| Sync-3 Lipsync | Sync Labs | No | **Maybe for paid lip-sync/redub benchmark.** Scenario uses Sync Labs style lip-sync; video only. | Scenario, Sync Labs API. |
| Sync Lipsync React-1 | Sync Labs | No | **Maybe for paid audio-to-video lip-sync.** Public Scenario workflow names it; not a GLB runtime tool. | Scenario, Sync Labs. |
| Pixverse Lipsync | Pixverse | No | **Maybe for stylized video lip-sync fallback.** Not relevant to GLB runtime. | Scenario, Pixverse platform/API. |
| LivePortrait | Kuaishou/community | Yes | **Maybe for open talking-portrait fallback.** Good for animating a 2D Morgan portrait if GLB path slips. | Hugging Face Spaces/self-host; possibly provider wrappers. |
| LivePortrait animal variants | Community | Yes/mixed | **Maybe because Morgan is canine.** Worth testing only if 2D fallback becomes important. | Hugging Face Spaces/self-host. |
| OmniAvatar | Community/various | Mixed | **Maybe for recent talking-head fallback.** Needs license/source review before adoption. | Hugging Face Spaces/self-host if repo available. |
| Hallo2 / Hallo4 | Fudan generative vision | Yes/mixed research | **Maybe for high-quality talking portrait fallback.** Video only; useful as quality reference. | GitHub/Hugging Face/self-host. |
| FantasyTalking | Fantasy-AMAP | Yes/mixed research | **Maybe as talking portrait baseline.** Video fallback only. | GitHub/self-host/Hugging Face if available. |
| EchoMimic | Ant Group / community | Yes/mixed research | **Already benchmarked enough for now.** Keep as fallback reference; not the new GLB direction. | GHCR image existed; self-host. OVH app has been removed. |
| MuseTalk | Tencent/community | Yes | **Maybe for lipsync video fallback.** Not a runtime GLB solution. | Hugging Face/self-host. |
| LatentSync | ByteDance/community | Yes/mixed | **Maybe for lipsync video fallback.** Useful if we have a base video to redub. | Hugging Face/self-host. |
| Wav2Lip / Wav2Lip-HD | Community | Yes/mixed | **Archive/benchmark only.** Older but reliable baseline for lip sync. | Hugging Face/self-host. |
| SadTalker | Community | Yes/mixed | **Archive/benchmark only.** Older talking-head baseline. | Hugging Face/self-host. |
| GFPGAN | Tencent ARC | Yes | **Maybe for 2D face restoration only.** Less relevant for canine Morgan and 3D runtime. | Hugging Face/self-host. |
| CodeFormer | S-Lab / community | Yes | **Maybe for 2D restoration only.** Not primary for Morgan 3D. | Hugging Face/self-host. |
| Real-ESRGAN | xinntao/community | Yes | **Yes as generic upscale/touch-up utility.** Use for source/reference QA, not model generation. | Hugging Face/self-host. |
| Voxel Crafter 1.0 | Scenario | No | **No for Morgan.** Scenario recommended it under text-to-3D, but voxel style is wrong for our target. | Scenario. |
| Sequence-to-Video | Scenario | No | **Maybe for workflow glue/video fallback.** Not a core avatar generation model. | Scenario. |
| Facial Expression Sheet - Kontext | Scenario / Flux Kontext | No | **Maybe for expression-reference sheets.** Useful for prompt/reference prep before face-control authoring. | Scenario. |

## Current shortlist by stage

| Stage | First choice | Second choice | Notes |
| --- | --- | --- | --- |
| Source cleanup/background | Gemini 3.1 Flash + RMBG/BiRefNet | SAM2 | Replace placeholder alpha masks before expensive GPU runs. |
| Primary head/asset generation | AniGen | Hunyuan 3D 3.1 Pro | AniGen is open-source/current path; Hunyuan 3D 3.1 Pro is Scenario's recommended paid image-to-3D quality model. |
| Mesh fallback | Hunyuan3D 2/2.1, TRELLIS.2 | Tripo 3.1 / Rodin | Prefer open-source if we are optimizing credits; use paid if quality is clearly better. |
| Retopo/optimization | Tripo Retopology | Blender/InstantMesh-style cleanup | Only after a visually good mesh exists. |
| Rigging/skinning | Tripo Rigging 1.0 | UniRig / Blender | Use paid rigging if it prevents a long manual Blender pass. |
| Face controls | ARKit/Oculus/VRM transfer | Faceit/Blender | We still need a reliable morph/viseme authoring path; this remains a major unknown. |
| Texture/material cleanup | Gemini 3.1 Flash / Imagen | Blender bake + Real-ESRGAN | Use paid image models only after shape/identity are good. |
| Video fallback | Kling V3 I2V Pro | Veo 3.1 / LTX / LivePortrait / Hallo | For hero videos or fallback, not the browser GLB runtime. |

## Provider/credit map

| Provider/credit pool | Candidate models/services from this catalog |
| --- | --- |
| Google / Vertex / Gemini | Gemini 3.1 Flash, Gemini/Imagen, Veo 3.1 |
| Microsoft / Azure | TRELLIS/TRELLIS.2 if available via Microsoft ecosystem; Azure Speech later for visemes/text-to-speech, not generation |
| Alibaba | No specific Scenario hit yet; investigate Qwen/Wan/Tongyi 3D/video offerings if credits are available |
| DigitalOcean | Ephemeral GPU Droplets for self-hosting AniGen, Hunyuan3D, TRELLIS, UniRig, RMBG/BiRefNet, LivePortrait, etc. |
| Hugging Face | Hosted Spaces/models for AniGen, Hunyuan3D, TRELLIS, UniRig, SkinTokens, LivePortrait, LatentSync, MuseTalk, Wav2Lip, RMBG/BiRefNet |
| Fireworks | No direct 3D avatar hit yet; likely useful for LLM/prompting/image-adjacent hosted models if supported |
| Scenario | Paid orchestration/wrappers for Hunyuan 3D 3.1 Pro, Tripo, Rodin, Kling, Veo, LTX, Sync/Veed/HeyGen/Pixverse, Gemini workflows |
| Tripo | Tripo 3D generation, rigging, retopology |
| RunPod / Modal | True short-lived/serverless-style GPU execution for open-source worker images if DigitalOcean is too VM-like |

## Notes

- The next execution blocker is still publishing a runnable `avatar-anigen-worker`
  image or choosing a hosted equivalent.
- DigitalOcean is currently best treated as an ephemeral GPU VM runner, not true
  serverless GPU. RunPod Serverless or Modal remain better matches for
  automatic scale-to-zero semantics.
- Use Scenario paid models as **repair/benchmark nodes** in the DAG, not as a
  replacement for the whole open-source plan unless quality or time savings make
  it obvious.
- Before running any Scenario model, call `scenario-get_model_schema` for the
  selected model ID and log its exact parameter contract in the run folder.
