# Avatar Asset Specification (Phase 3)

This document defines the **import validation checklist** and **rig/blendshape requirements** for all avatar assets in the CTO platform.

Every new avatar (human or humanoid-animal) must pass this checklist before it ships.

## Rig Families

We ship **rig families**, not a single universal skeleton:

| Family | Use case | Base skeleton |
|---|---|---|
| `biped-human` | Standard human, humanoid | VRM 1.0 / Mixamo |
| `biped-anthro` | Furry, kemono, anthropomorphic animal | VRM 1.0 with `aux_*` extension bones |
| `quadruped-mammal` | Dogs, cats, wolves, horses | Custom quad rig retargeted to biped for animation clips |
| `quadruped-avian` | Birds, gryphons | Custom quad rig with wing extension bones |

All families share a **semantic interface** so the runtime can treat them uniformly.

## Mandatory Checklist

### 1. File format and structure

- [ ] Single `.glb` file (glTF 2.0 binary)
- [ ] Draco mesh compression enabled (`KHR_draco_mesh_compression`)
- [ ] KTX2 texture compression (`KHR_texture_basisu`)
- [ ] File size ≤ 25 MB post-compression
- [ ] Passes `gltf-validator` with zero errors
- [ ] Contains `KHR_materials_pbrSpecularGlossiness` or `KHR_materials_specular` for PBR

### 2. Skeleton / bones

- [ ] Total bone count ≤ 80 (≤ 60 preferred for mobile)
- [ ] All bones use `lowerCamelCase` naming
- [ ] Hierarchy root: `Hips` → `Spine` → `Spine1` → `Spine2` → `Neck` → `Head`
- [ ] Arms: `LeftShoulder` → `LeftArm` → `LeftForeArm` → `LeftHand` (same for right)
- [ ] Legs: `LeftUpLeg` → `LeftLeg` → `LeftFoot` → `LeftToeBase` (same for right)
- [ ] Eye bones present: `LeftEye`, `RightEye` (not blendshape-only)
- [ ] Jaw bone present: `Jaw` (separate from `jawOpen` blendshape)
- [ ] Extension bones (if animal): prefixed with `aux_` (e.g. `aux_Tail1`, `aux_EarL`)

### 3. Blendshapes (morph targets)

#### ARKit 52-shape set (mandatory for biped-human and biped-anthro)

All 52 ARKit blendshapes must be present with exact names:

- **Eye:** `eyeBlinkLeft`, `eyeLookDownLeft`, `eyeLookInLeft`, `eyeLookOutLeft`, `eyeLookUpLeft`, `eyeSquintLeft`, `eyeWideLeft` (× 2 for right)
- **Eyebrow:** `browDownLeft`, `browInnerUp`, `browOuterUpLeft` (× 2 for right)
- **Mouth:** `mouthClose`, `mouthDimpleLeft`, `mouthDimpleRight`, `mouthFunneler`, `mouthLeft`, `mouthLowerDownLeft`, `mouthLowerDownRight`, `mouthPressLeft`, `mouthPressRight`, `mouthPucker`, `mouthRollLower`, `mouthRollUpper`, `mouthShrugLower`, `mouthShrugUpper`, `mouthSmileLeft`, `mouthSmileRight`, `mouthStretchLeft`, `mouthStretchRight`, `mouthUpperUpLeft`, `mouthUpperUpRight`
- **Nose:** `noseSneerLeft`, `noseSneerRight`
- **Cheek:** `cheekPuff`, `cheekSquintLeft`, `cheekSquintRight`
- **Tongue:** `tongueOut`, `tongueUp`, `tongueCurl` (minimum 3; add more if available)

#### Oculus visemes (15-shape mapping)

For lip-sync compatibility, map to these 15 visemes:

| Viseme | Phonetic description | ARKit primary |
|---|---|---|
| `sil` | Silence | — |
| `PP` | Bilabial plosive | `mouthClose`, `mouthPress*` |
| `FF` | Labiodental fricative | `mouthPress*`, `cheek*` |
| `TH` | Dental fricative | `tongue*`, `jawOpen` |
| `DD` | Alveolar plosive | `jawOpen`, `mouthDimple*` |
| `kk` | Velar plosive | `mouthShrugLower`, `jawOpen` |
| `CH` | Postalveolar affricate | `mouthFunneler`, `tongue*` |
| `SS` | Alveolar fricative | `mouthSmile*`, `teethShow` |
| `nn` | Alveolar nasal | `cheekSquint*`, `noseSneer*` |
| `RR` | Alveolar approximant | `mouthRoll*`, `tongueUp` |
| `aa` | Open front vowel | `jawOpen`, `mouthWide` |
| `E` | Mid front vowel | `mouthSmile*`, `jawOpen` |
| `I` | Close front vowel | `mouthStretch*`, `lipTightener` |
| `O` | Close-mid back vowel | `mouthPucker`, `jawOpen` |
| `U` | Close back vowel | `mouthPucker`, `mouthClose` |

- [ ] Morph target count ≤ 60 active per draw call (WebGL2 limit)
- [ ] All blendshape weights range 0.0–1.0
- [ ] No negative weights or out-of-range values

#### Species-specific corrective deltas (for muzzled animals)

For biped-anthro and quadruped families:

- [ ] `arkit_*` blendshapes present (semantic, retargetable)
- [ ] `species_*` blendshapes present (corrective deltas for muzzle/snout geometry)
- [ ] `aux_*` blendshapes present (tail/ear/wing secondary motion)
- [ ] Mapping table from viseme → species blendshapes in `avatar.json`

### 4. Materials and textures

- [ ] Albedo/base color: ≤ 2K resolution
- [ ] Normal map: ≤ 1K resolution
- [ ] ORM (occlusion/roughness/metallic): ≤ 1K, packed in single texture
- [ ] Total texture count per region: ≤ 5 (head, body, hair, eyes, accessories)
- [ ] No uncompressed PNG/JPG textures in final `.glb`
- [ ] Material count ≤ 12 (merge materials where possible)

### 5. Animation clips

- [ ] Reference animation set: `idle`, `listen`, `speak`, `think`, `acknowledge`
- [ ] Each clip ≤ 3 seconds (loopable)
- [ ] Clips authored on reference rig, retargeted to species rig
- [ ] No baked IK — runtime IK only (TwoBoneIK)
- [ ] Animation file size ≤ 500 KB total

### 6. Manifest file (`avatar.json`)

Every avatar ships with a manifest:

```json
{
  "version": "1.0.0",
  "name": "morgan-cyberpunk",
  "family": "biped-human",
  "rig": {
    "boneCount": 67,
    "blendshapeCount": 55,
    "hasEyes": true,
    "hasJaw": true,
    "hasTongue": true,
    "extensionBones": []
  },
  "texture": {
    "maxResolution": "2048x2048",
    "format": "ktx2",
    "totalSizeBytes": 3145728
  },
  "animation": {
    "clips": ["idle", "listen", "speak", "think", "acknowledge"],
    "totalSizeBytes": 245760
  },
  "asset": {
    "filename": "morgan-cyberpunk.glb",
    "sizeBytes": 18350080,
    "sha256": "abc123...",
    "lodMap": {
      "lod0": "morgan-cyberpunk-lod0.glb",
      "lod1": "morgan-cyberpunk-lod1.glb",
      "lod2": "morgan-cyberpunk-lod2.glb"
    }
  },
  "visemeMapping": "arkit52-to-ovrlipsync.json",
  "license": "internal-use-only"
}
```

- [ ] Manifest present and valid JSON
- [ ] SHA-256 hash matches actual `.glb` file
- [ ] LOD map entries exist and validate
- [ ] Viseme mapping file present for the rig family

## CI Validation Gate

Add to `.github/workflows/avatar-lint.yml`:

```yaml
name: Avatar Asset Lint

on:
  pull_request:
    paths:
      - 'avatar/design/samples/**/*.glb'
      - 'avatar/design/samples/**/avatar.json'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install gltf-validator
        run: npm install -g gltf-validator-cli
      - name: Validate .glb files
        run: |
          for f in $(find avatar/design/samples -name '*.glb'); do
            gltf-validator "$f" --fail-on-errors
          done
      - name: Validate manifests
        run: |
          for f in $(find avatar/design/samples -name 'avatar.json'); do
            python -m json.tool "$f" > /dev/null
          done
      - name: Check file sizes
        run: |
          find avatar/design/samples -name '*.glb' -size +25M -exec echo "FAIL: {} exceeds 25MB" \; -exec false \;
      - name: Check bone count
        run: |
          # Use gltf-transform or custom script to validate bone count
          echo "TODO: bone count validation"
```

## Import Workflow

1. Author asset in Blender (or equivalent DCC)
2. Export to `.glb` with Draco + KTX2
3. Run `gltf-validator` locally
4. Create `avatar.json` manifest
5. Place in `avatar/design/samples/v{N}/{slug}/`
6. Add entry to `avatar/design/samples/v{N}/{slug}/_selection.md`
7. Open PR — CI runs validation gate
8. On merge, copy selected asset to `avatar/web/public/avatars/`
9. Update `_selection.md` with `selected: true` and production path

## Compatibility Matrix

| Feature | biped-human | biped-anthro | quadruped-mammal | quadruped-avian |
|---|---|---|---|---|
| ARKit 52 | ✅ required | ✅ required | ⚠️ partial | ⚠️ partial |
| Oculus 15 | ✅ required | ✅ required | ✅ required | ✅ required |
| Eye bones | ✅ required | ✅ required | ✅ required | ⚠️ optional |
| Jaw bone | ✅ required | ✅ required | ⚠️ adaptive | ⚠️ adaptive |
| Tongue blendshapes | ✅ required | ✅ required | ❌ N/A | ❌ N/A |
| Tail/ears | ❌ N/A | ✅ aux_* | ✅ aux_* | ✅ aux_* |
| Runtime IK | ✅ TwoBone | ✅ TwoBone | ⚠️ raycast | ⚠️ raycast |
| LOD levels | ✅ 3 | ✅ 3 | ✅ 3 | ✅ 3 |
