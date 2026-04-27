# Morgan avatar model DAG plan

## Strategy

Get a working high-quality avatar pipeline first, then optimize cost. Scenario
Pro is now the primary hosted execution and asset-management path for source
prep, image-to-3D, rigging, retopo, and preview assets. Use open-source and OVH
self-hosted workers only after Scenario either validates the path or fails a
specific gate that justifies custom infrastructure.

The pipeline should be a gated DAG, not a single fixed model chain. Each node
produces artifacts and scores; downstream nodes run only when they repair a
specific failed gate or improve an accepted candidate.

## Current decision

| Decision | Current stance |
| --- | --- |
| Runtime goal | Browser-side GLB/VRM-style avatar over LiveKit/WebRTC, not continuous server-rendered video. |
| Execution goal | Scenario-hosted jobs and Scenario asset IDs first; no long-running GPU apps for validation. |
| Cost stance | Use Scenario credits to remove provisioning delay; optimize to open-source/self-hosted after quality is proven. |
| Model stance | Keep open-source and paid candidates cataloged, but run hosted Scenario candidates first. |
| Current blocker | Need the Scenario secret in the Avatar/OpenClaw runtime plus a source-reference upload/run manifest. |

## DAG

```text
Morgan source image
  -> Scenario upload / asset registration
  -> Scenario source cleanup / multiview reference sheet
  -> Scenario 3D head/asset candidates
  -> validate + render
  -> branch:
       good mesh, weak rig      -> Scenario rigging / retopo pass
       good head, weak face     -> face-control pass outside Scenario if needed
       weak geometry/materials  -> alternate Scenario 3D generator
       good candidate           -> torso/unified runtime asset
  -> final GLB/VRM package
  -> browser lab + LiveKit runtime proof
```

## Pipeline stages

| Stage | First tests | Why | Output gate |
| --- | --- | --- | --- |
| Source upload / asset registry | Scenario asset upload + tags/collections | Scenario becomes the run ledger for source refs, generated GLBs, and previews. | Stable Scenario asset IDs, provenance, and project-scoped metadata. |
| Source cleanup / references | Scenario Gemini 3.1 Flash, background removal/edit tools, multiview prompts | Better inputs improve every generator and reduce manual cleanup. | Head/bust/full refs with clean edges, no white halo, stable Morgan identity. |
| Primary hosted asset | Scenario Hunyuan 3D 3.1 Pro (`model_hunyuan-3d-pro-3-1-i23d`, multiview where refs exist) | Scenario docs position Hunyuan 3D 3.1 Pro as high-fidelity image-to-3D with PBR and multiview support. | Recognizable Morgan head, clean material path, usable mesh, Scenario asset output ready for validation. |
| Game-ready hosted route | Scenario Tripo P1 / Tripo 3.1 (`model_tripo-p1-image-to-3d`, `model_tripo-p1-multiview-to-3d`, `model_tripo-v3-1-image-to-3d`) | Tripo P1/3.1 expose face limits, PBR, HD texture, quad/low-poly and multiview controls for realtime assets. | Usable topology/materials and compatible export under our validator. |
| Retopo / optimization | Scenario Hunyuan Polygen 1.5 (`model_tencent-smarttopology`), Tripo Retopology, Blender automation | Run only after a visually good mesh exists. | Realtime-acceptable triangle count and preserved texture/material quality. |
| Rig / skinning | Scenario Tripo Rigging (`model_tripo-rigging-v1`, `model_tripo-rigging-v2-5`), UniRig/Blender fallback | Run if mesh is good but skeleton/weights are weak. | Usable humanoid/canine-compatible skeleton, skinning, pose/deformation proof. |
| Face controls | ARKit/Oculus/VRM transfer, Faceit/Blender | Major unknown; required for runtime lip sync. | Oculus visemes or ARKit/VRM expression coverage, mouth/eye deformation proof. |
| Texture/material cleanup | Scenario Tencent Texture Edit, Gemini/Imagen, Blender bake | Run only when shape is good but finish is weak. | No white/background contamination, stable color/materials in Blender and browser. |
| Video fallback / hero clips | Scenario lip-sync/I2V models, Kling V3 I2V Pro, Veo 3.1, LTX, LivePortrait/Hallo | Useful for demos or fallback, but not the main GLB runtime. | MP4 quality benchmark only. |

## Parallel test matrix

Run these in parallel once inputs and credentials are ready:

| Track | Model/tool | Execution mode | Purpose |
| --- | --- | --- | --- |
| A | Scenario upload + asset tagging | Scenario MCP/API | Register canonical Morgan source, refs, current GLB, run manifests. |
| B | Scenario Hunyuan 3D 3.1 Pro single/multiview | Scenario hosted | Highest-quality geometry/material baseline. |
| C | Scenario Tripo P1 / Tripo 3.1 single/multiview | Scenario hosted | Game/realtime topology baseline. |
| D | Scenario source cleanup/reference generation | Scenario hosted | Better front/side/back refs for all 3D tracks. |
| E | Validator/render harness | Local/OpenClaw CPU Blender + browser lab | Gate all Scenario outputs; Scenario metadata is advisory, not authoritative. |

Do not wait for every possible fallback before making the first quality
decision. Start with A-D, then branch based on validator/render output.

## Worker/image readiness

| Worker | Needed now? | Notes |
| --- | --- | --- |
| Scenario MCP/API lane | Yes | Primary path for uploads, model runs, asset display/download, and job state. |
| `avatar-anigen-worker` | Deferred | Build only if Scenario output fails or cost/quality later justifies self-hosting. |
| `avatar-mesh-worker` | Deferred | Only needed if we self-host Hunyuan3D/TRELLIS/Rodin after hosted validation. |
| `avatar-rig-postprocess-worker` | Deferred but likely | Blender/UniRig/retopo/blendshape worker for repair stages after a good mesh exists or Scenario rigging is insufficient. |

## Provider stance

| Provider | Use now? | Role |
| --- | --- | --- |
| Scenario | Yes, primary | Hosted model execution, asset upload/list/display/download, job state, Hunyuan/Tripo/Rodin/Meshy/Trellis/Tencent utility models, video fallback. |
| Google / Vertex | Yes | Gemini source prep, texture/reference generation, possible Veo fallback. |
| Hugging Face | Later | Open-source model hosting/spaces and self-host references after Scenario validates or fails. |
| OVH / DigitalOcean | Deferred | Ephemeral GPU/self-hosted workers after hosted validation, not tonight's default path. |
| RunPod / Modal | Deferred | True short-lived/serverless GPU workers if we later need custom open-source scale-to-zero. |
| Alibaba | Research next | Check Wan/Qwen/Tongyi/3D/video offerings if credits are confirmed. |
| Fireworks | Research next | Likely useful for LLM/prompt/image-adjacent hosted models, no clear 3D avatar model yet. |

Detailed model and provider catalogs are intentionally kept outside this active
plan directory:

- `.avatar-docs/available-models.md`
- `.avatar-docs/provider-credit-map.md`

## Immediate next steps

1. Inject Scenario credentials into the Morgan Avatar/OpenClaw runtime without committing secret values.
2. Upload/register canonical Morgan source refs in Scenario with run tags and provenance.
3. Use Scenario MCP/API schemas for Hunyuan 3D 3.1 Pro multiview, Tripo P1/3.1, Tripo Rigging, Tencent topology/UV/texture utilities before each run.
4. Run Hunyuan 3D 3.1 Pro and Tripo P1/3.1 Scenario candidates in parallel where credits allow.
5. Validate every GLB candidate with the runtime validator and Blender/browser renders; do not trust Scenario metadata alone for skeleton, morph, or runtime-readiness.
6. Pick the best head/mesh candidate before spending time on torso/runtime polish or self-hosted worker images.

## Guardrails

- Do not keep GPUs running while planning or waiting.
- Do not build or provision OVH/self-hosted model images until a Scenario-hosted validation result proves the gap.
- Do not treat video lip-sync models as a replacement for the GLB runtime path.
- Do not optimize for open-source cost until we have one high-quality working asset path.
- Do not manually touch up assets until the DAG has tried source cleanup, alternate generation, rigging, retopo, and material repair nodes.
- Do not treat Scenario asset metadata as a promotion gate. It is useful triage data, but promotion requires our GLB validator and browser/Blender evidence.
