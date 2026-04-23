# Morgan Anthro-Canine Rig — Blendshape Spec (WS-B2)

This document is the **sourcing & authoring contract** for the Morgan canine
avatar GLB that replaces the Ready Player Me stand-in landed in WS-B1
(PR #4795). It is **not** a modeling task — the rig itself will be
commissioned externally. The goal of this doc is to pin down exactly what
morph targets (blendshapes) the GLB must ship with so that the existing
TalkingHead + HeadAudio runtime continues to drive lip-sync and facial
animation unchanged.

Sources of truth already in the repo:

- HeadAudio viseme enum: [`avatar/web/types/headaudio.d.ts`](../types/headaudio.d.ts)
- Runtime driving `head.mtAvatar[...].newvalue`: [`avatar/web/components/TalkingHeadView.tsx`](../components/TalkingHeadView.tsx)
- Shipped worklet (no npm install required to inspect):
  [`avatar/web/public/headaudio/headworklet.min.mjs`](../public/headaudio/headworklet.min.mjs)
- RPM stand-in contract: [`avatar/web/config/morgan.ts`](../config/morgan.ts)
- Upstream ARKit 52 reference: <https://arkit-face-blendshapes.com/>
- Upstream TalkingHead docs: <https://github.com/met4citizen/TalkingHead>

---

## 1. Required blendshape sets

The GLB **must** expose the union of the three sets below as glTF morph
targets on the head mesh (or meshes — TalkingHead merges by name). Missing
any viseme silently breaks lip-sync for that phoneme class; missing ARKit
breaks eye gaze, blinks, and brow/jaw expressivity.

### 1.1 HeadAudio Oculus visemes (hard requirement)

Taken directly from `avatar/web/types/headaudio.d.ts` — these names are what
the AudioWorklet writes into `head.mtAvatar[key].newvalue` every audio
frame. Names are case-sensitive and must match exactly:

| Morph target   | Phoneme class              |
| -------------- | -------------------------- |
| `viseme_sil`   | silence / rest             |
| `viseme_aa`    | open vowels (father, bat)  |
| `viseme_E`     | mid-front vowels (bed)     |
| `viseme_I`     | close-front vowels (sit)   |
| `viseme_O`     | mid-back vowels (go)       |
| `viseme_U`     | close-back vowels (boot)   |
| `viseme_PP`    | bilabial stops (p, b, m)   |
| `viseme_FF`    | labiodentals (f, v)        |
| `viseme_TH`    | dentals (th)               |
| `viseme_DD`    | alveolar stops (t, d, n)   |
| `viseme_kk`    | velar stops (k, g, ng)     |
| `viseme_CH`    | postalveolar affricates    |
| `viseme_SS`    | alveolar fricatives (s, z) |
| `viseme_nn`    | nasal resonance (n, m, ng) |
| `viseme_RR`    | rhotic (r)                 |

These are the Oculus LipSync / Ready Player Me `Oculus Visemes` morph set.
Most RPM, VRoid, and MetaHuman-exported rigs already carry them under these
exact names; custom sculpts frequently rename them and **must be renamed on
export**.

### 1.2 ARKit 52 (hard requirement for non-viseme animation)

TalkingHead's idle loop, gaze, blinks, and emotive states drive the full
Apple ARKit 52 blendshape set. Canonical list (see
<https://arkit-face-blendshapes.com/>):

```
browDownLeft, browDownRight, browInnerUp, browOuterUpLeft, browOuterUpRight,
cheekPuff, cheekSquintLeft, cheekSquintRight,
eyeBlinkLeft, eyeBlinkRight,
eyeLookDownLeft, eyeLookDownRight,
eyeLookInLeft, eyeLookInRight,
eyeLookOutLeft, eyeLookOutRight,
eyeLookUpLeft, eyeLookUpRight,
eyeSquintLeft, eyeSquintRight,
eyeWideLeft, eyeWideRight,
jawForward, jawLeft, jawOpen, jawRight,
mouthClose, mouthDimpleLeft, mouthDimpleRight,
mouthFrownLeft, mouthFrownRight,
mouthFunnel, mouthLeft,
mouthLowerDownLeft, mouthLowerDownRight,
mouthPressLeft, mouthPressRight,
mouthPucker, mouthRight, mouthRollLower, mouthRollUpper,
mouthShrugLower, mouthShrugUpper,
mouthSmileLeft, mouthSmileRight,
mouthStretchLeft, mouthStretchRight,
mouthUpperUpLeft, mouthUpperUpRight,
noseSneerLeft, noseSneerRight,
tongueOut
```

Names must be camelCase exactly as above (matches RPM's `morphTargets=ARKit`
export and TalkingHead's internal mapping tables).

### 1.3 Canine-species add-ons (required for Morgan)

Morgan is an anthro-canine; the ARKit set assumes a humanoid face and has
no way to articulate a snout, jowls, or independently-rotating ears.
These are authored as additional glTF morph targets on the head mesh
(ear rotations may be bone-driven instead if the rig prefers — see §3).

| Morph target         | Rationale                                                                 |
| -------------------- | ------------------------------------------------------------------------- |
| `snout_open`         | Jaw hinge extends along the snout axis; ARKit `jawOpen` alone looks flat. |
| `snout_wrinkle`      | Dorsal muzzle wrinkle for snarl / sniff / disgust expressions.            |
| `jowl_flap_left`     | Upper-lip pendulous jowl, left. Catches sibilants visually.               |
| `jowl_flap_right`    | Upper-lip pendulous jowl, right. Symmetric pair with above.               |
| `tongue_loll`        | Relaxed open-mouth tongue hang (panting idle). Extends `tongueOut`.       |
| `ear_left_rotate`    | Pinna forward/back pitch, left ear. Signals attention / alert state.      |
| `ear_right_rotate`   | Pinna forward/back pitch, right ear.                                      |
| `ear_left_droop`     | Pinna droop, left (submissive / sad idle).                                |
| `ear_right_droop`    | Pinna droop, right.                                                       |
| `nose_twitch`        | Small micro-motion for idle liveliness.                                   |

These are additive — TalkingHead ignores unknown morph targets, so the
canine names will not break the existing runtime. Future PRs can drive
them from emotional state or a secondary worklet.

---

## 2. Authoring pipeline (recommended)

1. **Model & sculpt** the canine head in Blender 4.x (or Maya / ZBrush).
   Keep topology quad-dominant around the muzzle, eyes, and jowls; each
   blendshape target must share vertex count + order with the neutral mesh.
2. **Author blendshapes as Shape Keys** in Blender. Name them exactly as
   listed in §1.1–§1.3 (case-sensitive). Use the
   [ARKit Blendshapes for Blender](https://github.com/JimWest/MeFaMo) add-on
   or a MetaHuman transfer as the ARKit 52 starting point, then hand-sculpt
   the viseme + canine deltas.
3. **Skeleton**: humanoid-compatible (Mixamo naming) for TalkingHead body
   IK. Ears can be extra bones driven by the `ear_*_rotate/droop` morphs
   via Blender drivers if bone animation is preferred over pure morphs.
4. **Export as glTF 2.0 binary (`.glb`)** with:
   - `Morph Targets` ✓
   - `Morph Normals` ✓ (RPM parity — improves silhouette on snout_open)
   - `Apply Modifiers` ✓
   - Draco compression **off** (TalkingHead's loader expects raw morph
     attributes; Draco is fine for positions but strips morph deltas on
     some pipelines).
5. **Optional VRM 0.x export** if the asset needs to be reused in VRM hosts
   — VRM 0.x preserves named morph targets; VRM 1.0 remaps them and is
   **not** recommended for this pipeline.
6. **Validate** with the glTF Validator
   (<https://github.khronos.org/glTF-Validator/>) and with a quick script
   that lists `primitive.extras.targetNames` for every required name in
   §1. The CI gate for this contract is tracked in the follow-up issue
   (see §5).

---

## 3. Hosting & URL convention

Existing hosting pattern in the repo points the RPM stand-in at
`https://models.readyplayer.me/<id>.glb?morphTargets=...`. For custom
assets we host under the 5dlabs CDN (see `infra/` and the `cdn.5dlabs.ai`
entries in infra values). The canine asset will ship as:

```
https://cdn.5dlabs.ai/avatars/morgan-canine-v1.glb
```

Versioning rules:

- Bump the `-vN` suffix on any change that adds or renames morph targets,
  changes skeleton bone names, or changes mesh topology. Clients cache by
  URL, so reusing the same URL after a morph-target rename will silently
  desync lip-sync on warm browsers.
- Keep `morgan-canine-vN.glb` immutable once referenced from `main`.
- Environment override remains `NEXT_PUBLIC_AVATAR_GLB_URL` (WS-B1
  contract) — no new env var is introduced in WS-B2.

---

## 4. Config wiring (WS-B2 stub)

`avatar/web/config/morgan.ts` is updated in this PR to:

- Keep `MORGAN_DEFAULT_GLB_URL` pointing at the RPM stand-in (unchanged).
- Add a commented-out `MORGAN_CANINE_GLB_URL` constant pointing at the
  hosted URL above.
- Add a `// TODO(ws-b2): swap to canine rig when hosted` marker next to
  the default.

The swap itself — changing the exported default — happens in a follow-up
PR once the asset is live behind the URL and has passed the morph-target
validation script.

---

## 5. Follow-up work

Tracked in the GitHub issue linked from this PR's description. Summary:

1. Commission or author the GLB per §2.
2. Upload to `https://cdn.5dlabs.ai/avatars/morgan-canine-v1.glb`.
3. Land a validation script (e.g. `scripts/validate-avatar-glb.mjs`) that
   fails CI if any name in §1.1 or §1.2 is missing from the GLB's morph
   target list.
4. Flip `MORGAN_DEFAULT_GLB_URL` to `MORGAN_CANINE_GLB_URL` and remove
   the TODO marker.
