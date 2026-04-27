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
| Cute animal stance | Solve cuteness in the 2D source/multiview references before image-to-3D. 3D conversion models mostly infer from pixels and will not reliably make a creepy source cute. |
| Current blocker | Need a source-reference upload/run manifest and first Scenario dog/Morgan candidate runs. |

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

## Cute dog / animal 2D-to-3D guidance

The talking-dog tests showed that the weak point is not lip sync; it is the
image-to-3D conversion drifting toward an uncanny humanoid or insufficiently
cute animal. Treat this as source conditioning plus conservative conversion:

1. Make the source image cute first with Scenario image editing, preferably
   `model_google-gemini-3-1-flash`. Prompt for a charming pet mascot or soft
   plush dog figurine while preserving dog muzzle, fur pattern, ears, and
   animal anatomy. Explicitly ban human lips, human teeth, human skin,
   humanoid face structure, and biped proportions.
2. If the UI can produce consistent front/side/back references, create clean
   multiviews before 3D. Single-image conversion invents the unseen shape and
   is more likely to humanize or distort the dog.
3. Run `model_hunyuan-3d-pro-3-1-i23d` first for quality conversion. Use
   `generateType: Normal`, `enablePbr: true`, and `faceCount` around
   `300000-500000` for the first serious candidate. Hunyuan 3D 3.1 Pro is
   mostly image-driven, so do not expect a text prompt at this stage to fix
   cuteness.
4. If Hunyuan preserves charm but the mesh is too dense or hard to rig, try
   Tripo 3.0 / P1 with detailed geometry/texture, PBR, `quad: true`, and a
   `faceLimit` around `50000-100000`.
5. If organic shape or surface quality is messy, try Trellis 2 with
   `resolution: 1536`, `textureSize: 4096`, `remesh: true`, `remeshBand: 2-4`,
   and `decimationTarget: 100000`.

Do not use terms like `talking portrait`, `avatar face`, `lip sync`, or
`human-like expression` in 3D conversion prompts. Those belong only in the
later video fallback lane, never in the GLB/VRM runtime conversion lane.

### Post-conversion likeness refinement

If a generated dog mesh is structurally good but loses cuteness, markings, or
personality, do not discard it immediately. Run an image-conditioned 3D refiner
before retopology:

| Refinement pass | Scenario model | Inputs | Suggested settings | Use when |
| --- | --- | --- | --- | --- |
| Geometry/surface likeness | `model_ultrashape-1-0` | original cute image + coarse GLB | `numInferenceSteps: 50`, `octreeResolution: 1024`, fixed `seed` for comparisons | The silhouette, muzzle, eyes, or soft dog personality drifted during 2D-to-3D. |
| Reference retexture | `model_trellis-2-retexture` | original cute image + refined GLB | `resolution: 1024`, `textureSize: 4096`, `texSlatGuidanceStrength: 1`, fixed `seed` | Shape is acceptable but fur, eyes, colors, or markings no longer match the source. |
| Texture-only edit | `model_tencent-texture-edit` | FBX under 100k faces + image or prompt | use the reference image, not a prompt, when preserving identity | Only after decimation/export to FBX; useful for texture cleanup, not shape. |
| Retopo after likeness | `model_tripo-retopology` | refined GLB | `quad: true`, `bake: true`, `faceLimit: 10000-20000` | The refiner preserves charm but the mesh is too dense for runtime. |
| Shape-preserving reduction | `model_tencent-smarttopology` / Hunyuan Polygen 1.5 | refined GLB or OBJ | `polygonType: quadrilateral`, `faceLevel: medium` or `high` | Need lower-poly topology while keeping the accepted shape. |

Recommended cute-dog chain:

```text
original cute image
  -> Hunyuan 3D 3.1 Pro or Tripo image-to-3D
  -> Ultrashape 1.0 with the same original image
  -> Trellis 2 Retexture with the same original image, if markings/eyes/fur drift
  -> Tripo Retopology or Hunyuan Polygen only after likeness is accepted
  -> Blender/browser validation gallery
```

Retopology and rigging must come after likeness refinement. If topology cleanup
runs too early, it can bake in an uncanny shape and make later image-conditioned
refinement less effective.

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
