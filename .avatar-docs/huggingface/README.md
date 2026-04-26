# Hugging Face Avatar Model Candidates

Recent Hugging Face model/Space catalog for a multi-stage Morgan avatar pipeline. This is intentionally broader than the repo clone list: it includes direct generation, lip-sync, 3D mesh, rigging/skeleton, motion, and cleanup/touch-up candidates.

- Source: Hugging Face public Hub via MCP-guided terms plus public HF API normalization
- Recent filter: createdAt or lastModified >= 2025-09-01 when available; semantic MCP seed Spaces kept if highly relevant
- Candidates: **98**
- Candidates with obvious GitHub links in public model cards: **14**

## Suggested pipeline buckets

| Stage | Candidate families | Role |
| --- | --- | --- |
| 1. Prep / cleanup | RMBG, BiRefNet, CodeFormer, GFPGAN, Real-ESRGAN | crop/mask/restoration/upscale before generation |
| 2. Talking portrait Plan B | LivePortrait, OmniAvatar, Skyreels A1, Hallo2, SadTalker, Wav2Lip, MuseTalk, LatentSync | image/audio or driving-video to talking head output |
| 3. 3D base asset | Hunyuan3D, TRELLIS, TripoSR, AniGen, FaceLift, Sapiens pointmap | Morgan mesh / pointmap / textured 3D starting point |
| 4. Rigging/control | UniRig, SkinTokens, SMPL/SMPL-X variants, VRM/avatar skeleton rows | skeleton, skin weights, controls, retargeting |
| 5. Motion/animation | AniGen, HY-Motion, LTX/LongCat avatar/video rows | motion layer after static asset exists |

## 3D animation / motion (12)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [VAST-AI/AniGen](https://hf.co/spaces/VAST-AI/AniGen) | space | 2026-04-24 | 45 | https://github.com/VAST-AI-Research/AniGen.git, https://github.com/baegwangbin/DSINE, https://github.com/microsoft/TREL… | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [VAST-AI/SkinTokens](https://hf.co/VAST-AI/SkinTokens) | model | 2026-04-20 | 4 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [williamgoodman/HY-Motion-1.0](https://hf.co/spaces/williamgoodman/HY-Motion-1.0) | space | 2026-02-23 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [apozz/UniRig-safetensors](https://hf.co/apozz/UniRig-safetensors) | model | 2026-01-16 | 3 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [Heartsync/HY-Motion-1.0](https://hf.co/spaces/Heartsync/HY-Motion-1.0) | space | 2026-01-04 | 2 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [tencent/HY-Motion-1.0](https://hf.co/spaces/tencent/HY-Motion-1.0) | space | 2026-01-01 | 273 | https://github.com/black-forest-labs/flux, https://github.com/huggingface/diffusers, https://github.com/openai/CLIP, ht… | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [HechicerIA/HY-Motion-1.0](https://hf.co/spaces/HechicerIA/HY-Motion-1.0) | space | 2026-01-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [KJSlksj/HY-Motion-1.0](https://hf.co/spaces/KJSlksj/HY-Motion-1.0) | space | 2026-01-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [inoculatemedia/HY-Motion-1.0](https://hf.co/spaces/inoculatemedia/HY-Motion-1.0) | space | 2026-01-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [RayDay/HY-Motion-1.0](https://hf.co/spaces/RayDay/HY-Motion-1.0) | space | 2026-01-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [SpyC0der77/HY-Motion-1.0](https://hf.co/spaces/SpyC0der77/HY-Motion-1.0) | space | 2026-01-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |
| [clove1002/anigen](https://hf.co/spaces/clove1002/anigen) | space | 2025-09-01 | 0 | not found in card | Candidate for animatable 3D asset generation or motion layer after mesh creation. |

## adjacent / needs triage (12)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [facebook/sapiens2-pointmap](https://hf.co/spaces/facebook/sapiens2-pointmap) | space | 2026-04-24 | 3 | https://github.com/facebookresearch/sapiens2 | Potentially useful later; inspect before adoption. |
| [Shravani-Limited/VideoAvatar-UK-Voice-Engine](https://hf.co/Shravani-Limited/VideoAvatar-UK-Voice-Engine) | model | 2026-02-17 | 2 | not found in card | Potentially useful later; inspect before adoption. |
| [banao-tech/3D_AI_Avatar](https://hf.co/spaces/banao-tech/3D_AI_Avatar) | space | 2026-02-05 | 1 | not found in card | Potentially useful later; inspect before adoption. |
| [vantagewithai/LongCat-Video-Avatar-ComfyUI-GGUF](https://hf.co/vantagewithai/LongCat-Video-Avatar-ComfyUI-GGUF) | model | 2026-01-05 | 18 | https://github.com/meituan-longcat/LongCat-Video, https://github.com/meituan-longcat/LongCat-Flash-Chat, https://github… | Potentially useful later; inspect before adoption. |
| [fjkane/LongCat-Video-Avatar-bf16](https://hf.co/fjkane/LongCat-Video-Avatar-bf16) | model | 2025-12-19 | 2 | not found in card | Potentially useful later; inspect before adoption. |
| [meituan-longcat/LongCat-Video-Avatar](https://hf.co/meituan-longcat/LongCat-Video-Avatar) | model | 2025-12-17 | 235 | https://github.com/meituan-longcat/LongCat-Video, https://github.com/meituan-longcat/LongCat-Flash-Chat, https://github… | Potentially useful later; inspect before adoption. |
| [cpuai/LongCat-Video-Avatar](https://hf.co/spaces/cpuai/LongCat-Video-Avatar) | space | 2025-12-17 | 25 | not found in card | Potentially useful later; inspect before adoption. |
| [Tabsubject/LongCat-Video-Avatar](https://hf.co/spaces/Tabsubject/LongCat-Video-Avatar) | space | 2025-12-17 | 2 | not found in card | Potentially useful later; inspect before adoption. |
| [vvmarchuk/LongCat-Video-Avatar](https://hf.co/spaces/vvmarchuk/LongCat-Video-Avatar) | space | 2025-12-17 | 2 | not found in card | Potentially useful later; inspect before adoption. |
| [rwgertdthdht/LongCat-Video-Avatar](https://hf.co/spaces/rwgertdthdht/LongCat-Video-Avatar) | space | 2025-12-17 | 1 | not found in card | Potentially useful later; inspect before adoption. |
| [digital-avatar/ditto-talkinghead](https://hf.co/digital-avatar/ditto-talkinghead) | model | 2025-11-12 | 34 | https://github.com/antgroup/ditto-talkinghead, https://github.com/user-attachments/assets, https://github.com/thuhcsi/S… | Potentially useful later; inspect before adoption. |
| [Yashwant0806/avatartalk-3d-chatterbox](https://hf.co/spaces/Yashwant0806/avatartalk-3d-chatterbox) | space | 2025-10-10 | 3 | not found in card | Potentially useful later; inspect before adoption. |

## image/text-to-3D asset (18)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [VAST-AI/AniGen](https://hf.co/VAST-AI/AniGen) | model | 2026-04-13 | 10 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [oxide-lab/TripoSR](https://hf.co/oxide-lab/TripoSR) | model | 2026-04-10 | 2 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [Zhzzzzz/Hunyuan3D-2.1](https://hf.co/Zhzzzzz/Hunyuan3D-2.1) | model | 2026-04-03 | 1 | https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1, https://github.com/VAST-AI-Research/TripoSG, https://github.com/faceb… | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [AgenticVibes/hunyuan3d-2.1-mlx](https://hf.co/AgenticVibes/hunyuan3d-2.1-mlx) | model | 2026-04-03 | 1 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [moeqbadar/Hunyuan3D-2.1](https://hf.co/moeqbadar/Hunyuan3D-2.1) | model | 2026-04-01 | 1 | https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1, https://github.com/VAST-AI-Research/TripoSG, https://github.com/faceb… | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [prithivMLmods/TRELLIS.2-Text-to-3D](https://hf.co/spaces/prithivMLmods/TRELLIS.2-Text-to-3D) | space | 2026-03-23 | 35 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [choephix/TRELLIS.2](https://hf.co/spaces/choephix/TRELLIS.2) | space | 2026-03-09 | 2 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [jc-builds/triposr-ios](https://hf.co/jc-builds/triposr-ios) | model | 2026-03-08 | 3 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [hysts-mcp/TRELLIS](https://hf.co/spaces/hysts-mcp/TRELLIS) | space | 2026-02-26 | 17 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [opsiclear-admin/Trellis.2.multiview](https://hf.co/spaces/opsiclear-admin/Trellis.2.multiview) | space | 2026-02-24 | 10 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [tencent/HY-Motion-1.0](https://hf.co/tencent/HY-Motion-1.0) | model | 2025-12-31 | 402 | https://github.com/Tencent-Hunyuan/HY-Motion-1.0, https://github.com/Tencent-Hunyuan/HY-Motion-1.0.git, https://github.… | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [microsoft/TRELLIS.2](https://hf.co/spaces/microsoft/TRELLIS.2) | space | 2025-12-17 | 1448 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [frogleo/Image-to-3D](https://hf.co/spaces/frogleo/Image-to-3D) | space | 2025-10-24 | 188 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [togawa83/TripoSRSafetensors](https://hf.co/togawa83/TripoSRSafetensors) | model | 2025-09-21 | 0 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [casperbankerson/InstantMesh](https://hf.co/spaces/casperbankerson/InstantMesh) | space | 2025-09-08 | 0 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [trellis-community/TRELLIS](https://hf.co/spaces/trellis-community/TRELLIS) | space | 2025-06-25 | 592 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [tencent/Hunyuan3D-2](https://hf.co/spaces/tencent/Hunyuan3D-2) | space | 2025-06-02 | 3258 | https://github.com/Tencent/Hunyuan3D-2, https://github.com/facebookresearch/dinov2, https://github.com/Stability-AI/sta… | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |
| [8hsbot/InstantMesh](https://hf.co/spaces/8hsbot/InstantMesh) | space | 2025-03-19 | 0 | not found in card | Candidate for generating or improving Morgan 3D mesh/asset before rigging. |

## lip-sync / talking portrait (18)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [fffiloni/LatentSync](https://hf.co/spaces/fffiloni/LatentSync) | space | 2026-04-24 | 591 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [Adedoyinjames/SadTalker_free-API](https://hf.co/spaces/Adedoyinjames/SadTalker_free-API) | space | 2026-04-16 | 1 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [vinthony/SadTalker](https://hf.co/spaces/vinthony/SadTalker) | space | 2026-03-25 | 1434 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [Kirfol/LatentSync](https://hf.co/Kirfol/LatentSync) | model | 2026-03-01 | 1 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [Xuttt123/latentsync-pruned](https://hf.co/Xuttt123/latentsync-pruned) | model | 2026-02-01 | 1 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [nota-ai/compressed-wav2lip](https://hf.co/spaces/nota-ai/compressed-wav2lip) | space | 2026-01-20 | 82 | https://github.com/Nota-NetsPresso/nota-wav2lip.git, https://github.com/orgs/Nota-NetsPresso, https://github.com/Rudrab… | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [mortal9826/wav2lip-weights](https://hf.co/mortal9826/wav2lip-weights) | model | 2026-01-07 | 3 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [DevHimal/Wav2Lip-HD](https://hf.co/DevHimal/Wav2Lip-HD) | model | 2025-12-04 | 1 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [trymonolith/MuseTalk](https://hf.co/spaces/trymonolith/MuseTalk) | space | 2025-12-04 | 2 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [shibing624/ai-avatar-wav2lip](https://hf.co/shibing624/ai-avatar-wav2lip) | model | 2025-11-17 | 3 | https://github.com/shibing624/AIAvatar, https://github.com/lipku/LiveTalking, https://github.com/shibing624/AIAvatar.gi… | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [fabiolamp/wav2lip_fab_GPU](https://hf.co/spaces/fabiolamp/wav2lip_fab_GPU) | space | 2025-10-24 | 3 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [segestic/MusetalkLive](https://hf.co/spaces/segestic/MusetalkLive) | space | 2025-09-18 | 0 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [Parvej24/wav2lip-models](https://hf.co/Parvej24/wav2lip-models) | model | 2025-09-14 | 0 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [zackzhang76/sadtalker-weights](https://hf.co/zackzhang76/sadtalker-weights) | model | 2025-09-02 | 0 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [manavisrani07/gradio-lipsync-wav2lip](https://hf.co/spaces/manavisrani07/gradio-lipsync-wav2lip) | space | 2024-09-07 | 176 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [pragnakalp/Wav2lip-ZeroGPU](https://hf.co/spaces/pragnakalp/Wav2lip-ZeroGPU) | space | 2024-05-08 | 66 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [sjk4367/MuseTalk](https://hf.co/spaces/sjk4367/MuseTalk) | space | 2024-04-15 | 0 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |
| [kevinwang676/SadTalker](https://hf.co/spaces/kevinwang676/SadTalker) | space | 2023-07-19 | 39 | not found in card | Can synchronize Morgan speech to face video, especially after a portrait/video base exists. |

## rigging / skeleton / avatar controls (8)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [xwshi/Seamless-Avatar-Smplx-Model](https://hf.co/xwshi/Seamless-Avatar-Smplx-Model) | model | 2026-02-28 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [MajorDaniel/UniRig](https://hf.co/spaces/MajorDaniel/UniRig) | space | 2026-02-10 | 3 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [cynthiayetsko/UniRig](https://hf.co/spaces/cynthiayetsko/UniRig) | space | 2025-11-17 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [seungminkwak/UniRig-1](https://hf.co/spaces/seungminkwak/UniRig-1) | space | 2025-10-28 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [ivalenzuela/UniRigExtras](https://hf.co/spaces/ivalenzuela/UniRigExtras) | space | 2025-09-10 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [netw1z/UniRig](https://hf.co/spaces/netw1z/UniRig) | space | 2025-06-04 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [Emilio1955/UniRig](https://hf.co/spaces/Emilio1955/UniRig) | space | 2025-06-04 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |
| [jacobyomer/UniRig](https://hf.co/spaces/jacobyomer/UniRig) | space | 2025-06-04 | 0 | not found in card | Potential skeleton/skin/blendshape layer for making a Morgan mesh puppetable. |

## talking head / portrait animation (18)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [innoai/LivePortrait](https://hf.co/spaces/innoai/LivePortrait) | space | 2026-04-24 | 52 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [alexnasa/OmniAvatar](https://hf.co/spaces/alexnasa/OmniAvatar) | space | 2026-04-23 | 283 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [yjjuy7/LivePortrait](https://hf.co/spaces/yjjuy7/LivePortrait) | space | 2026-04-16 | 2 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [sohaibdevv/ai-talking-head](https://hf.co/spaces/sohaibdevv/ai-talking-head) | space | 2026-04-04 | 1 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [elix3r/LTX-2.3-22b-AV-LoRA-talking-head](https://hf.co/elix3r/LTX-2.3-22b-AV-LoRA-talking-head) | model | 2026-03-24 | 34 | https://github.com/ClownsharkBatwing/RES4LYF | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [ghostai1/halloween1.3b_poltergeist](https://hf.co/ghostai1/halloween1.3b_poltergeist) | model | 2026-02-03 | 1 | https://github.com/ggerganov/llama.cpp | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [multimodalart/ltx2-audio-to-video](https://hf.co/spaces/multimodalart/ltx2-audio-to-video) | space | 2026-01-30 | 51 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [fudan-generative-ai/hallo4](https://hf.co/fudan-generative-ai/hallo4) | model | 2025-12-01 | 2 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [MySafeCode/OmniAvatar](https://hf.co/spaces/MySafeCode/OmniAvatar) | space | 2025-11-28 | 1 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [alexnasa/OmniAvatar-Clay-Fast](https://hf.co/spaces/alexnasa/OmniAvatar-Clay-Fast) | space | 2025-11-20 | 48 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [smartdigitalnetworks/MoDA-fast-talking-head](https://hf.co/spaces/smartdigitalnetworks/MoDA-fast-talking-head) | space | 2025-08-18 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [Muhammad-Qasim/MoDA-fast-talking-head](https://hf.co/spaces/Muhammad-Qasim/MoDA-fast-talking-head) | space | 2025-08-18 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [jkh32499/Plex-v2-MoDA-fast-talking-head](https://hf.co/spaces/jkh32499/Plex-v2-MoDA-fast-talking-head) | space | 2025-08-18 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [AbuNadirMir/OmniAvatar](https://hf.co/spaces/AbuNadirMir/OmniAvatar) | space | 2025-08-01 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [Kyryll/OmniAvatar](https://hf.co/spaces/Kyryll/OmniAvatar) | space | 2025-08-01 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [CKCATIEKLINE/LivePortrait-animal](https://hf.co/spaces/CKCATIEKLINE/LivePortrait-animal) | space | 2025-04-17 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [Abhisheksao/tts-hallo-talking-portrait](https://hf.co/spaces/Abhisheksao/tts-hallo-talking-portrait) | space | 2025-02-08 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |
| [mihozzv/tts-hallo-talking-portrait](https://hf.co/spaces/mihozzv/tts-hallo-talking-portrait) | space | 2024-06-27 | 0 | not found in card | Can animate a cleaned Morgan portrait or transfer driving motion as a fast Plan B. |

## touch-up / restoration / preprocessing (12)

| HF repo | Type | Recent date | Likes | GitHub | Relevance |
| --- | --- | --- | ---: | --- | --- |
| [briaai/RMBG-2.0](https://hf.co/briaai/RMBG-2.0) | model | 2026-04-06 | 1158 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [briaai/Fibo-Edit-RMBG](https://hf.co/briaai/Fibo-Edit-RMBG) | model | 2026-03-17 | 42 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [briaai/Fibo-Edit-RMBG](https://hf.co/spaces/briaai/Fibo-Edit-RMBG) | space | 2026-03-17 | 26 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [ZhengPeng7/BiRefNet_demo](https://hf.co/spaces/ZhengPeng7/BiRefNet_demo) | space | 2026-02-21 | 308 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [AXERA-TECH/CodeFormer](https://hf.co/AXERA-TECH/CodeFormer) | model | 2026-01-25 | 3 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [Xlnk/rmbg](https://hf.co/spaces/Xlnk/rmbg) | space | 2026-01-24 | 5 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [maxorange/GFPGAN-5](https://hf.co/spaces/maxorange/GFPGAN-5) | space | 2025-12-28 | 16 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [sczhou/CodeFormer](https://hf.co/spaces/sczhou/CodeFormer) | space | 2025-12-19 | 2370 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [avans06/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN](https://hf.co/spaces/avans06/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN) | space | 2025-11-27 | 131 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [x0x7/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN](https://hf.co/spaces/x0x7/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN) | space | 2025-11-27 | 4 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [jhon19792025/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN](https://hf.co/spaces/jhon19792025/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN) | space | 2025-11-27 | 3 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |
| [dhanraj9543494/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN](https://hf.co/spaces/dhanraj9543494/Image_Face_Upscale_Restoration-GFPGAN-RestoreFormer-CodeFormer-GPEN) | space | 2025-11-27 | 3 | not found in card | Useful pre/post-processing: crop cleanup, restoration, background removal, upscaling, segmentation. |

## Search terms used

`LivePortrait`, `MuseTalk`, `Wav2Lip`, `SadTalker`, `Hallo2`, `Hallo`, `FantasyTalking`, `EchoMimic`, `LatentSync`, `OmniAvatar`, `Skyreels talking head`, `MoDA talking head`, `talking portrait`, `talking head`, `talking face`, `lip sync avatar`, `audio driven portrait`, `portrait animation`, `face animation`, `video avatar`, `Hunyuan3D`, `TripoSR`, `TRELLIS`, `AniGen`, `UniRig`, `SkinTokens`, `3D avatar`, `image to 3D character`, `image to 3D avatar`, `avatar rigging`, `avatar skeleton`, `mesh avatar`, `SMPL avatar`, `VRM avatar`, `HY-Motion`, `FaceLift`, `Sapiens pointmap`, `InstantMesh`, `Wonder3D`, `Rodin 3D`, `GFPGAN`, `CodeFormer`, `face restoration`, `portrait enhancement`, `RMBG`, `BiRefNet`, `background removal portrait`, `human segmentation`, `SAM2 human`, `Real-ESRGAN face`

## Notes

- GitHub links were extracted from public README/model cards when present; blank github_links means no obvious GitHub URL was found in the card during this pass.
- Canonical older repos already cloned locally may be omitted here because this pass focuses on recent HF artifacts.
- Full structured metadata lives in `../index/huggingface-models.json`.
