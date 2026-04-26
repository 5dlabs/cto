# Morgan Avatar Pipeline Shortlist

This is the current working plan after peer review: use GPUs for **provisioning** avatar assets, then run Morgan as a **client-side 3D avatar** over the existing LiveKit loop. Keep server-side talking-head video generation as Plan B for hero clips, demos, and fallback.

## Core answer

The user mental model is mostly correct: a game-style avatar is a mesh plus a controllable rig, and modern models can automate parts of the asset-creation flow. The important correction is that **skeleton is necessary but not enough**.

| Layer | What it means | Why it matters |
| --- | --- | --- |
| Mesh | Visible vertices, faces, UVs, normals, textures/materials | Determines the surface we see and whether deformation can look good. |
| Skeleton | Bone/joint hierarchy in bind pose | Gives body pose and animation controls. |
| Skin weights | Per-vertex weights mapping mesh to bones | Determines whether shoulders, neck, jaw, arms, etc. deform cleanly. |
| Rig | Practical runtime contract: skeleton + skinning + naming + controls + animation conventions | GLB stores skeleton/skinning, but control rigs/IK are usually runtime logic or baked clips. |
| Blendshapes / morph targets | Per-vertex facial deltas driven by scalar weights | Usually the key layer for lip sync, facial expressions, and visemes. |
| Animations | Clips or procedural runtime controls driving bones and morph weights | Gives idle, listen, speak, gesture, gaze, and emotion behavior. |

The target artifact is therefore not just "a GLB with a skeleton." It is a **runtime-ready GLB/VRM-style avatar**:

- skinned body mesh with humanoid/Mixamo-compatible skeleton
- head/face mesh with ARKit-style expression blendshapes and Oculus/viseme blendshapes
- clean enough topology around eyes/mouth/neck to deform well
- material/texture quality good enough for close-up framing
- idle/listen/speak/gesture clips or procedural controls
- validated in the browser runtime before we call provisioning complete

## Recommended architecture

```text
LiveKit audio/text/data
  -> browser runtime
  -> viseme/timing extraction from TTS or PCM
  -> TalkingHead / Three.js avatar driver
  -> GLB/VRM morph weights + body animation clips
  -> rendered Morgan avatar layer over background
```

GPUs should be online only for:

- Morgan reference cleanup
- 3D mesh generation
- retopology / texture generation
- rigging / skinning / blendshape transfer
- optional hero-video generation

At runtime, LiveKit remains the transport. It should not become the animation system. The animation system should be browser-side: visemes, expressions, gaze, head motion, idle movement, and gesture state.

## Head / torso / background split

The split is useful for **pipeline design**:

- **Head:** identity, facial expressions, visemes, eye/gaze, jaw, mouth, close-up quality.
- **Torso/body:** humanoid skeleton, posture, idle, gestures, shoulder/neck deformation.
- **Background:** static plate first; later HDRI, 3D set, reactive visual environment, or composited scene.

But the runtime should ideally load a **single coherent avatar asset** rather than separate head and torso meshes, unless there is a strong reason to composite. Separate runtime assets create neck seams, mismatched lighting, and retargeting problems.

## Why this beats always-on video generation

| Axis | Provisioned GLB runtime | Server-side talking portrait video |
| --- | --- | --- |
| Runtime GPU cost | Near zero; client renders locally | GPU seconds per user/session/utterance |
| Latency | Low; visemes can stream with audio | Often queue/cold-start/generation delay |
| Flexibility | Gaze, gestures, mood, interruption, camera are controllable | Behavior is baked into generated video |
| Quality ceiling | Stylized/3D unless we solve photoreal avatar authoring | Photoreal hero clips are easier |
| Main risk | Asset provisioning quality, especially facial blendshapes | GPU cost, drift, artifacts, concurrency |

Bottom line: this is likely the most cost-effective architecture for an always-on LiveKit co-host. Keep talking-head video models for Plan B and marketing clips, not the primary live runtime.

## Minimal proof before heavy provisioning

Do not start by trying to generate perfect Morgan. First prove the runtime contract:

1. Load a known-good Ready Player Me / VRM / Mixamo-compatible placeholder GLB.
2. Run it in-browser with `met4citizen/TalkingHead`.
3. Feed audio/text from the current LiveKit loop.
4. Drive visemes, idle, listen, speak, two gestures, gaze, and mood.
5. Confirm the UX feels acceptable with a placeholder.
6. Then spend GPU effort on custom Morgan asset provisioning.

If the placeholder runtime feels bad, a better mesh will not save the architecture.

## Evaluation harness for headless provisioning

If we run AniGen/Hunyuan/TRELLIS/etc. headless instead of via Hugging Face UI, generate these artifacts every run:

- front, side, 3/4, close-up PNG renders
- wireframe render
- skeleton overlay
- skin-weight/deformation pose sheet
- mouth viseme test clip
- idle + gesture clip
- GLB validator output
- material/texture contact sheet

Generation should fix identity, silhouette, proportions, and base geometry. Post-processing should fix production-readiness: retopo, UVs, materials, textures, rigging, skin weights, morph targets, compression, and packaging.
