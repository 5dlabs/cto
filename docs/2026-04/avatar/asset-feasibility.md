# Morgan Avatar Asset Feasibility Gate

This gate answers the dependency question before runtime polish: can the ML
pipeline produce a Morgan avatar file that is good enough to drive in a
browser-side GLB/VRM runtime?

The target is not a pretty static mesh. The target is a runtime-ready asset with
acceptable head quality, clean materials, usable rigging, and enough facial
control for speech.

## Source-image conditioning matrix

Use `avatar/morgan.jpg` as the initial canonical source unless a better original
is selected. Generate comparable inputs before running expensive GPU jobs:

| Track | Input condition | Variants | Purpose |
| --- | --- | --- | --- |
| A | Original reference | full frame, bust, head + neck, head crop | Preserve identity and expose baseline model behavior. |
| B | Subject isolated / transparent | full frame, bust, head + neck, head crop | Remove background geometry and texture leakage. |
| C | Subject isolated on neutral blank background | full frame, bust, head + neck, head crop | Give image-to-3D models a stable non-alpha input. |

Record the exact crop, prompt, negative prompt, seed, and model settings for
each generated candidate.

## Acceptance labels

| Label | Meaning | Next action |
| --- | --- | --- |
| `runtime-ready` | Loads as GLB/VRM, has usable mesh/materials, skeleton/skinning, and required facial controls. | Wire into the browser runtime benchmark. |
| `needs-face-authoring` | Head/mesh is promising, but ARKit/Oculus/VRM facial controls are missing or incomplete. | Try blendshape transfer, Faceit/Blender, or a template-head workflow. |
| `needs-rigging` | Static mesh/materials are promising, but skeleton/skinning are missing or weak. | Try AniGen full output, SkinTokens, UniRig, or a template torso. |
| `mesh-only` | Mostly a static mesh with no usable rig or face controls. | Keep only if visual quality is exceptional. |
| `archive` | Identity, topology, materials, or deformation are not salvageable. | Do not spend more runtime work on this candidate. |

## Mechanical checks

Run `scripts/2026-04/validate-avatar-glb.py` on every GLB candidate. The validator emits
mesh/material/rig/morph coverage and supports three profiles:

- `talkinghead`: Oculus visemes + ARKit blendshapes
- `morgan-canine`: `talkinghead` plus Morgan/canine add-ons
- `vrm`: VRM expression vowels/emotion presets

Minimum mechanical criteria for the first TalkingHead-oriented pass:

| Check | Pass condition |
| --- | --- |
| GLB load | GLB 2.0 JSON chunk parses without error. |
| Mesh | At least one mesh and non-zero triangle count. |
| Materials | Materials/textures present; white-like materials are explainable. |
| Skeleton | At least one skin, skinned node, and meaningful bone count for body assets. |
| Facial morphs | Oculus visemes and ARKit blendshape names present, or clearly categorized as `needs-face-authoring`. |
| Scale/bounds | Bounding box is plausible and centered/normalizable in the browser lab. |

For a head-only early candidate, missing body skinning is not an automatic
archive decision; classify it as a head candidate and decide whether it can be
merged into a torso/body rig later.

Baseline command pattern:

```bash
python3 scripts/2026-04/validate-avatar-glb.py --profile talkinghead --no-fail \
  --output output/avatar-provisioning/baseline/validation/morgan-anigen-preview.talkinghead.json \
  avatar/web/public/avatar-lab/morgan-anigen-preview.glb
```

## Visual checks

Every serious candidate should have proof artifacts:

- front, side, and three-quarter renders
- head close-up
- neutral grey background render
- dark background render
- checker/transparent-edge render for white halo diagnosis
- wireframe or topology render around face, mouth, eyes, ears, and neck
- material/texture contact sheet
- skeleton overlay if bones exist
- viseme sheet if morph targets exist
- neck/shoulder pose sheet if skinning exists

The white/background issue is a first-class defect. It may come from source
background leakage, alpha handling, generated texture bleed, white material base
color, vertex colors, normals/lighting, or browser viewer exposure. Compare the
same GLB under multiple backgrounds before blaming the model.

Use the fast profile for very large GLBs. It samples a lightweight preview mesh
for basic views and skips expensive wireframe/skeleton overlays; full profile
keeps those overlays for smaller candidates and auto-skips them over the overlay
triangle budget.

```bash
blender --background --python scripts/render-avatar-glb.py -- \
  avatar/web/public/avatar-lab/morgan-anigen-preview.glb \
  --out-dir output/avatar-provisioning/baseline/renders/morgan-anigen-preview \
  --resolution 256 \
  --profile fast

blender --background --python scripts/render-avatar-glb.py -- \
  avatar/web/public/avatar-lab/morgan-procedural.glb \
  --out-dir output/avatar-provisioning/baseline/renders/morgan-procedural \
  --resolution 256
```

## Head-first decision rule

Do not proceed to torso/runtime integration until one head candidate is:

1. visually recognizable as Morgan,
2. clean around mouth, eyes, ears, fur edges, and neck,
3. free of obvious white/background contamination,
4. mechanically inspectable as GLB,
5. either already face-controllable or one bounded face-authoring pass away.

If the head passes, process the torso separately but package a single final
runtime asset. Separate head and torso runtime meshes should be avoided unless
there is no other viable route.

## Current local test assets

| Path | Role |
| --- | --- |
| `avatar/web/public/avatar-lab/morgan-anigen-preview.glb` | Existing generated candidate to diagnose first. |
| `avatar/web/public/avatar-lab/morgan-procedural.glb` | Synthetic procedural viseme harness; useful validator/lab baseline. |
| `avatar/web/public/avatar-lab/morgan-sample.glb` | Lightweight comparison asset. |
| `output/morgan-crop-*.jpg` | Existing crop/background comparison sheets. |
| `avatar/design/source-conditioning/morgan/` | Reproducible local source-conditioning matrix, prompts, manifest, and QA composites for `avatar/morgan.jpg`. |

## Baseline classifications

TalkingHead validation output lives under
`output/avatar-provisioning/baseline/validation/`.

| Asset | TalkingHead classification | Notes |
| --- | --- | --- |
| `morgan-anigen-preview.glb` | `needs-face-authoring` | 1 skinned mesh, 19 bones, 1,163,370 triangles, no morph targets; white-like base material. |
| `morgan-procedural.glb` | `mesh-only` | Synthetic baseline with Oculus visemes present, partial ARKit coverage, no skinning. |
| `morgan-sample.glb` | `mesh-only` | Lightweight textured comparison mesh with no skinning or morph targets. |

## Non-destructive AniGen / OVH prep lane

Until the AniGen head benchmark is explicitly approved, stay in local prep and
read-only cloud checks only. Do **not** run `avatar/scripts/ai-deploy-launch.sh`
or `avatar/scripts/gate3-bucket-setup.sh`; those can create or update OVH
resources.

Use the local helper to assemble a run id, expected paths, and copy/paste-safe
preflight commands without contacting OVH, S3, or GHCR:

```sh
avatar/scripts/anigen-benchmark-prep.sh
```

Default artifact layout:

```text
output/avatar-provisioning/<RUN_ID>/source-conditioning/
output/avatar-provisioning/<RUN_ID>/anigen/
output/avatar-provisioning/<RUN_ID>/validation/
s3://cto-avatar-gra/gate-artifacts/<RUN_ID>/  # only after upload approval
```

Safe readiness checks for the orchestrator to run later:

```sh
# OVH AI flavor readiness: GET only, no AI Deploy app creation.
PROJECT_ID="$(op --account "${OP_ACCOUNT_OVH:-my.1password.com}" read 'op://Automation/OVH CA API/project_id')"
REGION="${REGION:-GRA}"
FLAVOR="${FLAVOR:-ai1-1-gpu}"
avatar/scripts/ovh-api.sh GET "/cloud/project/${PROJECT_ID}/ai/capabilities/region/${REGION}/flavor" \
  | jq -e --arg flavor "$FLAVOR" \
      '[.. | objects | select([.id?, .name?, .flavor?, .spec.name?] | index($flavor))] | length > 0' \
      >/dev/null \
  && echo "[ok] OVH AI flavor visible: ${REGION}/${FLAVOR}"

# OVH S3 bucket readiness: HEAD only, no create-bucket / lifecycle writes.
BUCKET="${BUCKET:-cto-avatar-gra}"
S3_REGION="${S3_REGION:-gra}"
S3_ENDPOINT="https://s3.${S3_REGION}.perf.cloud.ovh.net"
AWS_ACCESS_KEY_ID="$(op read 'op://Automation/OVH S3-GRA Access Key/credential')" \
AWS_SECRET_ACCESS_KEY="$(op read 'op://Automation/OVH S3-GRA Secret/credential')" \
AWS_DEFAULT_REGION="$S3_REGION" \
aws --endpoint-url "$S3_ENDPOINT" s3api head-bucket --bucket "$BUCKET" \
  && echo "[ok] OVH S3 bucket reachable: s3://${BUCKET}"

# Container image readiness: registry manifest read only.
IMAGE="${IMAGE:-ghcr.io/5dlabs/anigen:latest}"
docker manifest inspect "$IMAGE" >/dev/null \
  && echo "[ok] image manifest exists: ${IMAGE}"
```

Secrets must stay as `op://` lookups or one-command environment injection; do
not paste secret values into notes, logs, or benchmark artifacts.
