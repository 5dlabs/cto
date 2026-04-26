# Archive / Deprioritize Notes

This does not delete cloned repos. It means "do not spend first-pass implementation time here unless a later search changes the evidence."

## Archive for first-pass runtime

| Candidate | Why not first-pass |
| --- | --- |
| `VAST-AI-Research/.github` | Organization metadata, not avatar pipeline code. |
| `VAST-AI-Research/VAST-AI-Research.github.io` | Website/org docs, not core runtime/provisioning. |
| `harlanhong/awesome-talking-head-generation` | Link farm; useful for research only. |
| `Kedreamix/Awesome-Talking-Head-Synthesis` | Link farm; useful for research only. |
| `gradio-app/gradio` | UI/demo framework, not avatar runtime or model. Useful only if building model demos. |
| `initml/social-files-size` | Output sizing utility, not avatar generation/runtime. |
| `initml/wasmedge-nodejs-starter` | Generic WASM starter, not avatar-specific. |
| `initml/smolagents-tests` | Agent library snapshot, not avatar-specific. |

## Deprioritize unless Plan B video is needed

These are useful, but they solve a different problem: generate or modify video on a server/GPU. They do not solve the low-cost client-side runtime.

| Candidate | Keep for |
| --- | --- |
| `Fantasy-AMAP/fantasy-talking` | high-quality hero clips / Plan B |
| `fudan-generative-vision/hallo2` | modern audio-driven portrait video |
| `TMElyralab/MuseTalk` | realtime-ish server video fallback |
| `KwaiVGI/LivePortrait` | portrait animation/motion transfer clips |
| `ByteDance/LatentSync` | lipsync refinement on existing video |
| `OpenTalker/SadTalker` | older baseline / fallback |
| `Rudrabha/Wav2Lip` | older baseline / fallback |
| `OpenTalker/video-retalking` | redub/retalk existing video |
| `HumanAIGC/EMO`, `ali-vilab/dreamtalk`, `tencent-ailab/V-Express` | research references and comparison points |

## Situational / later

| Candidate | When to revisit |
| --- | --- |
| `VAST-AI-Research/SeqTex`, `DetailGen3D` | after base mesh/rig exists and textures need improvement |
| `VAST-AI-Research/HoloPart`, `MIDI-3D`, `TriplaneGaussian` | if part segmentation, scene/object composition, or alternate 3D representations become relevant |
| `hongfz16/AvatarCLIP`, `dimgerogiannis/Arc2Avatar`, `DRiVEAvatar/DRiVEAvatar.github.io` | if we need academic references for text/motion/avatar transfer |
| `ValyrianTech/AgentRig` | if it contains concrete agent-driven rigging workflows after deeper inspection |
| `fagenorn/handcrafted-persona-engine` | behavior/persona state, not rendering |

## Do not overfit on mesh preview

Bad-looking preview mesh is a warning, but not the only metric. The runtime quality depends on:

- final topology after retopo/wrapping
- texture/material pass
- rig and skin weights
- facial morph targets/visemes
- lighting/camera/background
- animation polish

If the **identity and proportions** are wrong, rerun/tune generation. If the **base form is close** but deformation/materials are weak, post-process.
