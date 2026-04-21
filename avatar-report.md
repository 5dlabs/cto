---

# Open-Source Talking-Head Avatar Pipeline Research Report

> **See also**: [`docs/avatar-architecture.md`](./docs/avatar-architecture.md) — covers the LemonSlice/OpenClaw integration path, the `openclaw-avatar` plugin architecture, and multi-provider plugin design deltas. This report remains the canonical survey of self-hosted model candidates; the architecture doc covers how those candidates plug into OpenClaw + LiveKit.

**Target hardware:** NVIDIA V100S 32 GB · Volta compute 7.0 · Self-hosted K8s
**Constraints:** Zero budget · NO bf16 / bfloat16 · NO flash-attn v2 · fp16 only
**Inputs:** Still PNG + ElevenLabs WAV/MP3 · Dog moderator + 2 furries + human

---

## Executive Summary

Of the 17 candidates investigated, only **three** explicitly document V100/Volta compatibility: V-Express (tested, ~8 GB VRAM on V100), MuseTalk (advertises 30fps+ real-time on V100), and EchoMimic V2 (lists V100 16 GB as a tested GPU). For **animal/furry avatars**, only **LivePortrait Animals** and **JoyVASA** provide native non-human face support — all other candidates are human-only with face detectors (InsightFace, MediaPipe) that crash or produce garbage on non-human faces, confirming your SadTalker/MuseTalk observations. The recommended deployment strategy is a two-stage chained pipeline: LivePortrait or JoyVASA for gesture/body animation from still PNG, then MuseTalk or V-Express for lip-sync refinement, with separate paths for human vs. animal avatars.

---

## Candidate Table

| Name | Stage | VRAM (inf) | V100 compat | Animal support | License | URL |
|---|---|---|---|---|---|---|
| **EchoMimic V2** | Gesture/Body | ~16 GB (V100 tested) | ✅ Explicitly listed | ❌ Human only | Apache 2.0 | [antgroup/echomimic_v2](https://github.com/antgroup/echomimic_v2) |
| **LivePortrait** | Gesture/Body | ~4–8 GB | ⚠️ Likely OK (no xformers/flash-attn) | ✅ **Native Animals model** (dogs, cats) | MIT (InsightFace models: **non-commercial**) | [KwaiVGI/LivePortrait](https://github.com/KwaiVGI/LivePortrait) |
| **AnimateAnyone** (Moore) | Gesture/Body | ≥16 GB | ✅ torch 2.0.1 + xformers 0.0.22 | ❌ Human only | Apache 2.0 | [MooreThreads/Moore-AnimateAnyone](https://github.com/MooreThreads/Moore-AnimateAnyone) |
| **UniAnimate** | Gesture/Body | ~12 GB (512p) / ~21 GB (768p) | ⚠️ Likely OK (xformers 0.0.20, torch 2.0.1) | ❌ Human only | **No license file** | [ali-vilab/UniAnimate](https://github.com/ali-vilab/UniAnimate) |
| **MimicMotion** | Gesture/Body | 16 GB (VAE), 8 GB (U-Net) | ⚠️ Probably OK (torch 2.0.1, no xformers) | ❌ Human only | Apache 2.0 | [Tencent/MimicMotion](https://github.com/Tencent/MimicMotion) |
| **Champ** | Gesture/Body | Unknown (A100/3090 tested) | ⚠️ Risky — torch 2.2.2 + xformers 0.0.25.post1 | ❌ Human only | MIT | [fudan-generative-vision/champ](https://github.com/fudan-generative-vision/champ) |
| **Hallo** | Gesture/Body | Unknown (A100 tested) | ⚠️ Risky — torch 2.2.2+cu121, xformers 0.0.25.post1 | ❌ Human only | MIT | [fudan-generative-vision/hallo](https://github.com/fudan-generative-vision/hallo) |
| **Hallo2** | Gesture/Body | ~20 GB (250 frames) | ⚠️ Risky — xformers 0.0.25.post1 | ❌ Human only | MIT (CodeFormer: **S-Lab 1.0**) | [fudan-generative-vision/hallo2](https://github.com/fudan-generative-vision/hallo2) |
| **JoyVASA** | Gesture/Body | ~8 GB (RTX 4060 tested) | ⚠️ Risky — torch 2.2.2, xformers 0.0.25.post1 | ✅ **Native animal mode** (`--animation_mode animal`) | MIT | [jdh-algo/JoyVASA](https://github.com/jdh-algo/JoyVASA) |
| **V-Express** | Gesture/Body | ~8 GB (V100 tested, `--save_gpu_memory`) | ✅ **V100 explicitly tested** (7956 MiB) | ❌ Human only | **No license file** | [tencent-ailab/V-Express](https://github.com/tencent-ailab/V-Express) |
| **AniPortrait** | Gesture/Body | Unknown | ✅ torch 2.0.1 + xformers 0.0.22 | ❌ Human only | Apache 2.0 | [Zejun-Yang/AniPortrait](https://github.com/Zejun-Yang/AniPortrait) |
| **SadTalker-Video-Lip-Sync** | Lip-sync | Unknown ("high" for DAIN) | ✅ torch 1.12.1 (ancient, V100-safe) | ❌ Human only (crashes on animal faces) | **No license file** | [Zz-ww/SadTalker-Video-Lip-Sync](https://github.com/Zz-ww/SadTalker-Video-Lip-Sync) |
| **LatentSync 1.5** | Lip-sync | 8 GB | ⚠️ Uncertain — torch 2.5.1+cu121, no xformers | ❌ Human (+ anime) | Apache 2.0 | [bytedance/LatentSync](https://github.com/bytedance/LatentSync) |
| **LatentSync 1.6** | Lip-sync | 18 GB | ⚠️ Uncertain — same stack as 1.5 | ❌ Human (+ anime) | Apache 2.0 | [bytedance/LatentSync](https://github.com/bytedance/LatentSync) |
| **MuseTalk** | Lip-sync | ~4 GB (fp16) | ✅ **V100 real-time 30fps+** | ❌ Human only (gray blobs on animals) | MIT | [TMElyralab/MuseTalk](https://github.com/TMElyralab/MuseTalk) |
| **Wav2Lip** | Lip-sync | <4 GB (96×96 face) | ✅ torch 1.1.0 (V100-safe, ancient) | ❌ Human only | **Non-commercial / research only** | [Rudrabha/Wav2Lip](https://github.com/Rudrabha/Wav2Lip) |
| **VideoRetalking** | Lip-sync | Unknown | ✅ torch 1.9.0 (V100-safe) | ❌ Human only | Apache 2.0 | [OpenTalker/video-retalking](https://github.com/OpenTalker/video-retalking) |
| **Diff2Lip** | Lip-sync | Unknown | ✅ CUDA 11.3, no xformers | ❌ Human only | **CC BY-NC 4.0** (non-commercial) | [soumik-kanad/diff2lip](https://github.com/soumik-kanad/diff2lip) |

### xformers Version Compatibility Matrix (critical for Volta sm_70)

| xformers version | torch version | Volta/sm_70 support | Used by |
|---|---|---|---|
| **0.0.20** | torch 2.0.1 | ✅ Built with sm_70 | UniAnimate |
| **0.0.22** | torch 2.0.1 | ✅ Built with sm_70 | V-Express, AniPortrait, Moore-AnimateAnyone |
| **0.0.25.post1** | torch 2.2.2 | ⚠️ **sm_70 dropped** in some builds; must compile from source | Champ, Hallo, Hallo2, JoyVASA |
| **0.0.28.post3** | torch 2.5.1 | ⚠️ **sm_70 dropped** in official wheels; source build required | EchoMimic V2 |
| None | — | ✅ N/A | MuseTalk, LatentSync, Wav2Lip, VideoRetalking, Diff2Lip, MimicMotion |

**Key insight:** xformers ≥ 0.0.24 dropped Volta (sm_70) from prebuilt PyPI wheels ([xformers release notes](https://github.com/facebookresearch/xformers/releases)). Projects pinning 0.0.25.post1 (Hallo, Hallo2, JoyVASA, Champ) or 0.0.28.post3 (EchoMimic V2) will require **building xformers from source with `TORCH_CUDA_ARCH_LIST="7.0"`** or patching to use vanilla PyTorch attention. Projects using 0.0.22 or 0.0.20 (V-Express, AniPortrait, Moore-AnimateAnyone, UniAnimate) ship wheels with sm_70 support.

---

## Pipeline Chaining (Question 1)

### Feasibility

A two-stage pipeline — Stage 1 (gesture/body animation from still image + audio) → Stage 2 (lip-sync refinement) — is **feasible and is already the de facto architecture** in production workflows. Several projects explicitly acknowledge this pattern:

- **LatentSync** borrows code from MuseTalk, StyleSync, SyncNet, and Wav2Lip ([LatentSync README acknowledgments](https://github.com/bytedance/LatentSync#-acknowledgement)), indicating the community treats lip-sync as a composable post-process.
- **Hallo2** chains a base portrait animation with CodeFormer super-resolution as a built-in post-process stage ([Hallo2 README](https://github.com/fudan-generative-vision/hallo2)).
- **SadTalker-Video-Lip-Sync** explicitly chains SadTalker → Wav2Lip → DAIN interpolation → GFPGAN enhancement ([SadTalker-Video-Lip-Sync README](https://github.com/Zz-ww/SadTalker-Video-Lip-Sync)).

### Known Projects Doing This

1. **SadTalker-Video-Lip-Sync**: SadTalker (face motion) + Wav2Lip (lip sync) + DAIN (frame interpolation) + GFPGAN (face enhancement). Source: [Zz-ww/SadTalker-Video-Lip-Sync](https://github.com/Zz-ww/SadTalker-Video-Lip-Sync).
2. **Hallo2**: Base animation + CodeFormer + RealESRGAN post-process. Source: [fudan-generative-vision/hallo2](https://github.com/fudan-generative-vision/hallo2).
3. **Community ComfyUI workflows**: Chain LivePortrait → MuseTalk or Wav2Lip for lip-sync on the generated video.

### Quality Gotchas

- **Double encoding loss:** Each stage decodes to pixel space and re-encodes. For diffusion-based pipelines (LatentSync, Diff2Lip), this means VAE decode → re-encode → VAE decode, with cumulative quality loss. **Mitigation:** Use lossless intermediate formats (PNG sequences or FFV1 codec), never intermediate lossy MP4.
- **Color shift / gamma drift:** Different models normalize face crops differently. LivePortrait uses affine-transformed crops; MuseTalk uses its own face-parse-bisenet ROI. Compositing back introduces color seam artifacts at face boundaries. **Mitigation:** Match color space (sRGB throughout), apply Poisson blending or alpha-feathered compositing at the mask boundary.
- **Identity drift at seams:** The lip-sync stage may subtly alter identity features (jaw shape, skin texture) compared to the gesture stage output. Over long sequences this compounds. **Mitigation:** Keep the lip-sync ROI as small as possible (lower-face only), use the gesture stage's face as the identity anchor, and apply a soft blend mask.
- **Temporal jitter at stage boundaries:** If the two stages use different frame rates or temporal smoothing, cuts between segments produce visible jitter. **Mitigation:** Enforce 25fps throughout; apply temporal smoothing (Kalman filter as LivePortrait does with `pykalman`).

---

## Path A: Humans (Top 2 Ranked)

### #1: MuseTalk (lip-sync) + EchoMimic V2 or V-Express (gesture)

**For the gesture stage — EchoMimic V2:**

- **Pros:**
  - ✅ **V100 (16 GB) explicitly listed as tested GPU** ([README](https://github.com/antgroup/echomimic_v2))
  - Audio-driven semi-body animation — generates gesture + head motion + lip movement from audio alone
  - Apache 2.0 license — fully commercial-safe
  - Accelerated inference: ~50s/120 frames on A100 (9× speedup); V100 will be slower but functional
  - Active development (last updated 2024)

- **Cons:**
  - xformers 0.0.28.post3 pinned — **must build from source for sm_70** or patch to use `torch.nn.functional.scaled_dot_product_attention` (available in torch 2.5.1)
  - `torchao` nightly dependency (used for acceleration) — may have Volta compatibility issues; can be skipped for basic inference
  - VRAM: ~16 GB on V100 — fits in 32 GB with headroom but not enough for concurrent models
  - Human-only (no animal support)

- **VRAM:** ~16 GB inference on V100 (documented)
- **License:** Apache 2.0
- **URL:** [github.com/antgroup/echomimic_v2](https://github.com/antgroup/echomimic_v2)
- **Setup notes for V100:**
  1. Install torch 2.5.1+cu121 (or downgrade to 2.0.1+cu118 and adjust diffusers)
  2. Build xformers from source: `TORCH_CUDA_ARCH_LIST="7.0" pip install xformers --no-build-isolation`
  3. Skip `torchao` install (acceleration optional); inference works without it
  4. Set `PYTORCH_CUDA_ALLOC_CONF=max_split_size_mb:256` to prevent fragmentation
  5. Weights: ~2–4 GB total from HuggingFace `BadToBest/EchoMimicV2`

**For the lip-sync stage — MuseTalk:**

- **Pros:**
  - ✅ **V100 real-time at 30fps+** — explicitly documented ([README](https://github.com/TMElyralab/MuseTalk))
  - Extremely low VRAM: ~4 GB in fp16 mode — can co-reside with gesture model
  - Not a diffusion model — single-step latent inpainting, vastly faster than diffusion-based lip-sync
  - MIT license — fully commercial-safe
  - Accepts both video and image+audio input
  - v1.5 (March 2025) with improved quality

- **Cons:**
  - ❌ **Does not work on animal faces** — your experience confirms gray blob output on non-human inputs
  - Requires MMLab stack (mmcv, mmdet, mmpose) — heavyweight but well-documented install
  - Face region limited to 256×256 — lower res than LatentSync 1.6 (512×512)
  - Human face detector (DWPose + face-parse-bisenet) fails on non-standard face geometry

- **VRAM:** ~4 GB (fp16)
- **License:** MIT
- **URL:** [github.com/TMElyralab/MuseTalk](https://github.com/TMElyralab/MuseTalk)
- **Setup notes for V100:**
  1. PyTorch 2.0.1+cu118 (as documented)
  2. No xformers required — MuseTalk doesn't use it
  3. `--use_float16` flag for fp16 inference
  4. Install mmcv==2.0.1, mmdet==3.1.0, mmpose==1.1.0
  5. Weights: `musetalkV15/unet.pth`, `sd-vae-ft-mse`, `whisper-tiny`, DWPose, SyncNet (~3 GB total)

### #2: V-Express (gesture) + VideoRetalking (lip-sync)

**V-Express (gesture):**

- **Pros:**
  - ✅ **V100 explicitly tested** — peak 7956 MiB with `--save_gpu_memory` flag ([README](https://github.com/tencent-ailab/V-Express))
  - Lowest VRAM of any gesture candidate: ~8 GB
  - torch 2.0.1 + xformers 0.0.22 — **both V100/Volta-safe out of the box**, no source builds needed
  - Tunable reference/audio attention weights for quality control

- **Cons:**
  - **No license file in repo** — legal risk; cannot confirm commercial usage rights
  - Slow: 2617 seconds (43+ min) for 31-second audio on V100 — not practical for real-time
  - Human-only
  - Less active development than alternatives

- **VRAM:** ~8 GB with `--save_gpu_memory`
- **License:** ⚠️ **None declared** — contact Tencent AI Lab before production use
- **URL:** [github.com/tencent-ailab/V-Express](https://github.com/tencent-ailab/V-Express)
- **Setup notes for V100:**
  1. Direct `pip install -r requirements.txt` works — all deps are V100-safe
  2. torch 2.0.1, xformers 0.0.22 — prebuilt wheels include sm_70
  3. Use `--save_gpu_memory` flag always on V100
  4. Budget 40+ minutes per 30-second clip

**VideoRetalking (lip-sync):**

- **Pros:**
  - torch 1.9.0 — ancient, guaranteed V100-safe, no xformers/flash-attn
  - 3-stage pipeline (expression → lip-sync → face enhancement) — built-in quality refinement
  - Apache 2.0 license
  - Handles diverse head poses (though not extreme angles)

- **Cons:**
  - VRAM undocumented (likely 8–12 GB based on architecture)
  - Weights only on Google Drive (no HuggingFace) — harder to automate K8s download
  - Not actively maintained (last significant update 2023)
  - Human-only

- **VRAM:** Unknown (estimated 8–12 GB)
- **License:** Apache 2.0
- **URL:** [github.com/OpenTalker/video-retalking](https://github.com/OpenTalker/video-retalking)
- **Setup notes for V100:** Direct install works. torch 1.9.0+cu111. No special GPU flags needed.

---

## Path B: Animals/Furries (Top 2 Ranked)

### #1: LivePortrait Animals

- **Pros:**
  - ✅ **Dedicated Animals model** with separate inference script `inference_animals.py` and Gradio app `app_animals.py` ([README](https://github.com/KwaiVGI/LivePortrait))
  - Explicit support for dogs 🐶 and cats 🐱 — updated January 2025 with more training data
  - **No xformers, no flash-attn in requirements** — pure ONNX runtime + PyTorch standard ops
  - Very lightweight VRAM: ~4–8 GB (ONNX-based inference, not diffusion)
  - MIT license on code
  - Extremely well-maintained (1000+ GitHub stars, active development through 2025)
  - Works from a single still image — matches your PNG input constraint
  - Built-in Kalman smoothing for temporal stability

- **Cons:**
  - ⚠️ **InsightFace models bundled are non-commercial research-only** — must be replaced for commercial use, or accept research-only constraint
  - **X-Pose dependency required for animal mode** — Linux/Windows NVIDIA GPU only, no macOS. X-Pose may need specific build steps for V100
  - Not audio-driven natively — drives from a reference video's motion, not from audio. You'd need a separate audio-to-motion step or chain with a lip-sync stage
  - Animal model is for natural animal faces (cats, dogs) — **untested on anthropomorphic/furry art styles**. Furry characters with human-like facial proportions but animal textures are a gray area

- **VRAM:** ~4–8 GB
- **License:** MIT (code); InsightFace: **non-commercial research only**
- **URL:** [github.com/KwaiVGI/LivePortrait](https://github.com/KwaiVGI/LivePortrait)
- **Setup notes for V100:**
  1. `pip install -r requirements.txt` — no xformers, no flash-attn; deps are V100-safe
  2. Install X-Pose for animal mode: follow repo instructions for Linux NVIDIA
  3. Download animal model weights from HuggingFace (LivePortrait animals checkpoint)
  4. ONNX runtime GPU 1.18.0 — prebuilt for CUDA 11.x/12.x, works on V100
  5. Test with a natural dog/cat photo first, then try furry art to gauge quality

### #2: JoyVASA

- **Pros:**
  - ✅ **Native animal face animation mode** — paper title explicitly: *"Portrait and Animal Image Animation"* ([README](https://github.com/jdh-algo/JoyVASA))
  - **Audio-driven** — uses wav2vec2/HuBERT for audio → facial dynamics, which is exactly what you need for ElevenLabs WAV input
  - Runs on 8 GB VRAM (tested on RTX 4060 Laptop) — fits in V100 32 GB easily
  - MIT license — commercial-safe
  - Leverages LivePortrait's animal model internally (downloads `liveportrait_animals` weights)
  - Combines gesture + lip-sync in one pipeline — no chaining needed for animals

- **Cons:**
  - ⚠️ **xformers 0.0.25.post1 pinned** — must build from source for V100 sm_70, or patch to remove xformers dependency
  - torch 2.2.2 — relatively modern; may need downgrade to 2.0.1 + matching xformers 0.0.22 for guaranteed V100 compatibility
  - **MultiScaleDeformableAttention** custom CUDA op required for animal mode — must compile from source, and sm_70 compatibility is **untested**
  - Relatively new project (2024) — less battle-tested than LivePortrait
  - Animal results quality depends on LivePortrait's animal model — same untested-on-furries limitation
  - A100 is the listed "tested" GPU — V100 not explicitly confirmed

- **VRAM:** ~8 GB
- **License:** MIT
- **URL:** [github.com/jdh-algo/JoyVASA](https://github.com/jdh-algo/JoyVASA)
- **Setup notes for V100:**
  1. Downgrade to torch 2.0.1+cu118 + xformers 0.0.22 (V100-safe wheel exists)
  2. Rebuild `MultiScaleDeformableAttention` from source with `TORCH_CUDA_ARCH_LIST="7.0"`
  3. Verify `bitsandbytes==0.43.1` works on V100 (usually does — it has sm_70 fallback)
  4. Download weights: LivePortrait animals + wav2vec2/HuBERT from HuggingFace
  5. Test with `--animation_mode animal` on a natural dog photo before furry art

---

## Horizontal Scaling Notes

### Per-model single-GPU vs multi-GPU

| Model | Single-GPU inference? | Multi-GPU required? | Model weight size (est.) | Cold-start time (est.) |
|---|---|---|---|---|
| **EchoMimic V2** | ✅ Yes | No | ~4 GB (UNet + Ref UNet + VAE + Whisper) | 30–60s |
| **LivePortrait** | ✅ Yes | No | ~1–2 GB (ONNX models) | 10–20s |
| **JoyVASA** | ✅ Yes | No | ~3 GB (LivePortrait animals + wav2vec2) | 20–40s |
| **V-Express** | ✅ Yes | No | ~3 GB (UNet + VAE + audio encoder) | 20–30s |
| **MuseTalk** | ✅ Yes | No | ~2 GB (UNet + VAE + Whisper + DWPose) | 15–25s |
| **LatentSync 1.5** | ✅ Yes | No | ~2 GB (UNet + Whisper) | 15–25s |
| **LatentSync 1.6** | ✅ Yes | No | ~3 GB (512×512 UNet + Whisper) | 20–30s |
| **Wav2Lip** | ✅ Yes | No | ~0.5 GB (face det + Wav2Lip GAN) | 5–10s |
| **VideoRetalking** | ✅ Yes | No | ~2 GB (3-stage pipeline) | 20–30s |
| **Diff2Lip** | ✅ Single or Multi | Optional (MPI) | ~1 GB (guided-diffusion) | 15–25s |
| **UniAnimate** | ✅ Yes | Optional (A100 parallel denoise) | ~5 GB (CLIP + VAE + UNet) | 40–60s |
| **AniPortrait** | ✅ Yes | No | ~3 GB | 20–30s |
| **Moore-AnimateAnyone** | ✅ Yes | No | ~3 GB | 20–30s |
| **Champ** | ✅ Yes | No | ~3 GB | 20–30s |
| **Hallo / Hallo2** | ✅ Yes | No | ~4–5 GB | 30–50s |
| **MimicMotion** | ✅ Yes | No | ~4 GB (SVD + pose models) | 30–50s |

**All candidates support single-GPU inference.** No model _requires_ multi-GPU. For K8s horizontal scaling, each avatar job can run on a single V100S pod. Parallelism is achieved by running multiple pods, not multi-GPU within a pod.

**Cold-start optimization:** Pre-pull model weights into a PersistentVolume or init container. The ONNX-based models (LivePortrait) have the fastest cold start (~10s). Diffusion-based models (EchoMimic V2, LatentSync) are slower due to UNet + VAE loading.

---

## Show-Stoppers / Risks

### License Traps (Non-Commercial / Research-Only)

| Model | License | Commercial use? | Citation |
|---|---|---|---|
| **Wav2Lip** | Custom non-commercial | ❌ **BLOCKED** — "personal/research/non-commercial purposes" only, commercial = sync.so paid product | [README](https://github.com/Rudrabha/Wav2Lip#license-and-citation) |
| **Diff2Lip** | CC BY-NC 4.0 | ❌ **BLOCKED** — non-commercial only | [README](https://github.com/soumik-kanad/diff2lip) |
| **LivePortrait** (InsightFace models) | InsightFace: non-commercial research | ⚠️ Code MIT, but **bundled face detection models are research-only**. Must replace InsightFace for commercial use. | [InsightFace license](https://github.com/deepinsight/insightface/blob/master/LICENSE) |
| **V-Express** | **No license file** | ⚠️ **Legally ambiguous** — no explicit grant of rights | [Repo](https://github.com/tencent-ailab/V-Express) |
| **UniAnimate** | **No license file** | ⚠️ **Legally ambiguous** | [Repo](https://github.com/ali-vilab/UniAnimate) |
| **SadTalker-Video-Lip-Sync** | **No license file** | ⚠️ **Legally ambiguous** | [Repo](https://github.com/Zz-ww/SadTalker-Video-Lip-Sync) |
| **Hallo2** (CodeFormer component) | S-Lab License 1.0 | ⚠️ Redistribution constraint on the super-res component | [Hallo2 README](https://github.com/fudan-generative-vision/hallo2) |

### V100 Incompatibilities

1. **xformers ≥ 0.0.24 prebuilt wheels drop sm_70 (Volta)**
   - Affected: EchoMimic V2 (0.0.28.post3), Champ (0.0.25.post1), Hallo (0.0.25.post1), Hallo2 (0.0.25.post1), JoyVASA (0.0.25.post1)
   - Fix: Build xformers from source with `TORCH_CUDA_ARCH_LIST="7.0"` or replace with `torch.nn.functional.scaled_dot_product_attention` (torch ≥ 2.0)
   - Source: [xformers GitHub releases](https://github.com/facebookresearch/xformers/releases)

2. **flash-attn v2 is Ampere+ only (sm_80+)**
   - None of the investigated candidates explicitly require flash-attn in their requirements.txt. However, some underlying diffusers/transformers versions may auto-detect and try to import it.
   - Fix: Ensure `flash-attn` is never installed in the environment; PyTorch SDPA will fall back to math attention on Volta.

3. **bf16 / bfloat16 is not supported on Volta (compute 7.0)**
   - No candidate explicitly requires bf16 in their codebase. All use fp16.
   - Risk: Some diffusers versions default to bf16 on Linux. Set `torch_dtype=torch.float16` explicitly in all pipeline configs.

4. **torch 2.5.1+cu121 (LatentSync, EchoMimic V2)**
   - CUDA 12.1 toolkit works on V100, but these specific wheels are built for sm_80+.
   - Fix: Use `torch==2.0.1+cu118` or `torch==2.1.0+cu118` which include sm_70 binaries.

5. **MultiScaleDeformableAttention (JoyVASA animal mode)**
   - Custom CUDA kernel; must be compiled from source for sm_70.
   - Risk: May fail to compile if the op's CUDA code uses Ampere-specific intrinsics.

### Specific Broken Commits / Recent Regressions

- **LatentSync 1.6** (2025/06/11): v1.6 requires 18 GB VRAM — fits in V100S 32 GB but leaves less headroom than v1.5 (8 GB). Training stage2_512 requires 55 GB — impossible on V100S. Source: [LatentSync README](https://github.com/bytedance/LatentSync).
- **MuseTalk v1.5** (2025/03): Major update; older MuseTalk v1.0 workflows may break. Verify using v1.5-specific weights (`musetalkV15/unet.pth`). Source: [MuseTalk README](https://github.com/TMElyralab/MuseTalk).
- **EchoMimic V2** `torchao` dependency: Nightly build that may not have sm_70 support. Skip for inference (acceleration is optional). Source: [EchoMimic V2 install instructions](https://github.com/antgroup/echomimic_v2).
- **Hallo2** `--save_gpu_memory` not implemented — unlike V-Express, there's no memory optimization flag. The ~20 GB requirement is fixed for default settings. Source: [Hallo2 README](https://github.com/fudan-generative-vision/hallo2).

---

## Recommended Deployment Order

1. **MuseTalk v1.5** — Deploy first. Confirmed V100 real-time 30fps+, 4 GB VRAM, MIT license, no xformers. This is your **human lip-sync** baseline. Validates the full K8s pod pipeline (image+audio → video) with minimal risk. Source: [TMElyralab/MuseTalk](https://github.com/TMElyralab/MuseTalk).

2. **LivePortrait Animals** — Deploy second. **Only** production-grade animal face model. No xformers, no flash-attn, ~4–8 GB VRAM. Test with natural dog/cat photos, then with your furry art style to gauge quality. Provides the **gesture/motion** stage for animal avatars. Source: [KwaiVGI/LivePortrait](https://github.com/KwaiVGI/LivePortrait).

3. **V-Express** — Deploy third. **V100 explicitly tested** (7956 MiB), torch 2.0.1 + xformers 0.0.22 (V100-safe wheels). Provides audio-driven **human gesture** animation. Slow (43 min/30s clip) but reliable. Evaluate license risk with legal team. Source: [tencent-ailab/V-Express](https://github.com/tencent-ailab/V-Express).

4. **JoyVASA** — Deploy fourth. **Audio-driven animal animation** in one pipeline (no chaining needed). Requires xformers source build or downgrade to 0.0.22. Test MultiScaleDeformableAttention compilation on V100 first. If this works, it becomes the **single-pipeline solution for animal avatars**. Source: [jdh-algo/JoyVASA](https://github.com/jdh-algo/JoyVASA).

5. **EchoMimic V2** — Deploy fifth. Better quality than V-Express for human gesture, V100 explicitly tested. Requires xformers source build (0.0.28.post3 → sm_70). Skip `torchao`. Replaces V-Express as the human gesture stage if quality and speed are superior. Source: [antgroup/echomimic_v2](https://github.com/antgroup/echomimic_v2).

6. **LatentSync 1.5** — Deploy sixth (optional). Higher-quality lip-sync than MuseTalk (diffusion-based, 8 GB VRAM). Test on V100 — torch 2.5.1+cu121 may need downgrade. If it works, A/B test against MuseTalk for quality. The 1.6 version (18 GB) can be tested later. Source: [bytedance/LatentSync](https://github.com/bytedance/LatentSync).

7. **AniPortrait** — Deploy seventh (optional). torch 2.0.1 + xformers 0.0.22 — guaranteed V100-safe. Alternative human gesture pipeline with audio-to-mesh intermediate representation. Apache 2.0 license. Source: [Zejun-Yang/AniPortrait](https://github.com/Zejun-Yang/AniPortrait).

**Do NOT deploy** (license blockers): Wav2Lip (non-commercial), Diff2Lip (CC BY-NC 4.0).
**Defer** (legal ambiguity): V-Express and UniAnimate (no license files) — get legal clearance before production.
**Defer** (V100 risk without payoff): Hallo/Hallo2 (xformers 0.0.25.post1 + 20 GB VRAM + A100-only tested), Champ (same xformers issue + no audio driving), MimicMotion (16 GB VAE + no audio driving), UniAnimate (no license + 21 GB for 768p).

### Recommended K8s Pod Architecture

```
┌─────────────────────────────────┐
│ Human Avatar Pod (V100S 32GB)   │
│  Stage 1: EchoMimic V2 (~16GB) │
│  Stage 2: MuseTalk (~4GB)       │
│  Sequential, not concurrent     │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│ Animal Avatar Pod (V100S 32GB)  │
│  Option A: JoyVASA (~8GB)       │
│    (single pipeline, preferred) │
│  Option B: LivePortrait (~8GB)  │
│    + MuseTalk for lip-sync      │
│    (but MuseTalk → gray blobs!) │
└─────────────────────────────────┘
```

**For animal avatars:** JoyVASA is the only viable single-pipeline option since MuseTalk (your confirmed experience) and all other lip-sync tools produce garbage on non-human faces. If JoyVASA's lip-sync quality is insufficient, the fallback is LivePortrait for motion + **no lip-sync refinement** (accept LivePortrait's built-in mouth movement quality).