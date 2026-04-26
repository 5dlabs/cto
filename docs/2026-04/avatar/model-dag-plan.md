# Morgan avatar model DAG plan

## Strategy

Get a working high-quality avatar pipeline first, then optimize cost. Use
open-source models where they are ready, but spend available paid credits when a
hosted model can save infrastructure time or manual touch-up.

The pipeline should be a gated DAG, not a single fixed model chain. Each node
produces artifacts and scores; downstream nodes run only when they repair a
specific failed gate or improve an accepted candidate.

## Current decision

| Decision | Current stance |
| --- | --- |
| Runtime goal | Browser-side GLB/VRM-style avatar over LiveKit/WebRTC, not continuous server-rendered video. |
| Execution goal | Short-lived/serverless-style jobs; no long-running GPU apps. |
| Cost stance | Use credits/paid models to get a working pipeline quickly; optimize to open-source/self-hosted after quality is proven. |
| Model stance | Keep open-source and paid candidates ready so we can run parallel tests instead of stopping to reprovision. |
| Current blocker | Need an execution path for the next model run: Scenario hosted call and/or published worker image for self-hosted AniGen. |

## DAG

```text
Morgan source image
  -> source cleanup / reference sheet
  -> primary 3D head/asset candidates
  -> validate + render
  -> branch:
       good mesh, weak rig      -> rig/retopo pass
       good head, weak face     -> face-control pass
       weak geometry/materials  -> alternate 3D generator
       good candidate           -> torso/unified runtime asset
  -> final GLB/VRM package
  -> browser lab + LiveKit runtime proof
```

## Pipeline stages

| Stage | First tests | Why | Output gate |
| --- | --- | --- | --- |
| Source cleanup / references | Gemini 3.1 Flash, RMBG/BiRefNet, SAM2 if needed | Better inputs improve every generator and reduce manual cleanup. | Head/bust/full refs with clean edges, no white halo, stable Morgan identity. |
| Primary open-source asset | AniGen | Best current open-source candidate for skinned/animatable GLB-like output. | Recognizable Morgan head, clean material path, usable mesh, skeleton/skinning signal. |
| Primary paid asset | Hunyuan 3D 3.1 Pro via Scenario | Scenario recommends it as image-to-3D quality default; likely fastest way to test high-quality geometry. | Better geometry/materials than AniGen or clear reason to reject. |
| Paid game-ready route | Tripo 3.1 / Tripo P1 | Good candidate for game/realtime mesh if Hunyuan/AniGen are too dense or messy. | Usable topology/materials and compatible export. |
| Mesh fallback | Hunyuan3D 2/2.1, TRELLIS.2, Rodin | Keep ready for parallel tests if primary candidates fail. | Better shape/material than current best candidate. |
| Retopo / optimization | Tripo Retopology, Blender automation | Run only after a visually good mesh exists. | Realtime-acceptable triangle count and preserved texture/material quality. |
| Rig / skinning | Tripo Rigging 1.0, UniRig, Blender | Run if mesh is good but skeleton/weights are weak. | Usable humanoid/canine-compatible skeleton, skinning, pose/deformation proof. |
| Face controls | ARKit/Oculus/VRM transfer, Faceit/Blender | Major unknown; required for runtime lip sync. | Oculus visemes or ARKit/VRM expression coverage, mouth/eye deformation proof. |
| Texture/material cleanup | Gemini/Imagen, Blender bake, Real-ESRGAN | Run only when shape is good but finish is weak. | No white/background contamination, stable color/materials in Blender and browser. |
| Video fallback / hero clips | Kling V3 I2V Pro, Veo 3.1, LTX, LivePortrait/Hallo | Useful for demos or fallback, but not the main GLB runtime. | MP4 quality benchmark only. |

## Parallel test matrix

Run these in parallel once inputs and credentials are ready:

| Track | Model/tool | Execution mode | Purpose |
| --- | --- | --- | --- |
| A | AniGen | Self-hosted GPU worker | Open-source skinned GLB baseline. |
| B | Hunyuan 3D 3.1 Pro | Scenario hosted | Paid quality baseline. |
| C | Tripo 3.1 or Tripo P1 | Scenario hosted | Game/realtime mesh baseline. |
| D | Gemini 3.1 Flash + RMBG/BiRefNet | Hosted/local | Better source/reference inputs for all tracks. |

Do not wait for every possible fallback before making the first quality
decision. Start with A-D, then branch based on validator/render output.

## Worker/image readiness

| Worker | Needed now? | Notes |
| --- | --- | --- |
| `avatar-anigen-worker` | Yes | Required for self-hosted AniGen. Publish to a reachable registry or run through a hosted GPU notebook/Space equivalent. |
| `avatar-mesh-worker` | Soon, not first | Only needed if we self-host Hunyuan3D/TRELLIS/Rodin instead of using Scenario/Hugging Face. |
| `avatar-rig-postprocess-worker` | Soon, not first | Blender/UniRig/retopo/blendshape worker for repair stages after a good mesh exists. |

## Provider stance

| Provider | Use now? | Role |
| --- | --- | --- |
| Scenario | Yes | Fast paid tests for Hunyuan 3D 3.1 Pro, Tripo, rigging/retopo, video fallback. |
| Google / Vertex | Yes | Gemini source prep, texture/reference generation, possible Veo fallback. |
| Hugging Face | Yes | Open-source model hosting/spaces and self-host references. |
| DigitalOcean | Maybe | Ephemeral GPU VM runner for self-hosted workers; not true serverless GPU. |
| RunPod / Modal | Maybe | Better fit for true short-lived/serverless GPU workers if DigitalOcean is too VM-like. |
| Alibaba | Research next | Check Wan/Qwen/Tongyi/3D/video offerings if credits are confirmed. |
| Fireworks | Research next | Likely useful for LLM/prompt/image-adjacent hosted models, no clear 3D avatar model yet. |

Detailed model and provider catalogs are intentionally kept outside this active
plan directory:

- `.avatar-docs/available-models.md`
- `.avatar-docs/provider-credit-map.md`

## Immediate next steps

1. Resolve Scenario model schemas for Hunyuan 3D 3.1 Pro, Tripo 3.1/P1, Tripo Rigging, Tripo Retopology, Gemini 3.1 Flash, and Kling V3 I2V Pro.
2. Upload the canonical Morgan source/ref assets to Scenario or prepare equivalent hosted inputs.
3. Run the source cleanup/reference-sheet step first.
4. Run AniGen, Hunyuan 3D 3.1 Pro, and Tripo baseline tests in parallel where possible.
5. Validate every GLB candidate with `scripts/2026-04/validate-avatar-glb.py` and `scripts/render-avatar-glb.py`.
6. Pick the best head/mesh candidate before spending time on torso/runtime polish.

## Guardrails

- Do not keep GPUs running while planning or waiting.
- Do not treat video lip-sync models as a replacement for the GLB runtime path.
- Do not optimize for open-source cost until we have one high-quality working asset path.
- Do not manually touch up assets until the DAG has tried source cleanup, alternate generation, rigging, retopo, and material repair nodes.
