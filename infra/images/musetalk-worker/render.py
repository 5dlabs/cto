"""MuseTalk v1.5 realtime Avatar rendering.

Ported from upstream /models/musetalk-src/scripts/realtime_inference.py
(mirrored in ./_ref_realtime_inference.py) into a library-friendly shape:

    - No input() prompts; exposes force_recreate flag
    - No sys.exit(); raises RuntimeError instead
    - Absolute base_path /models/avatars/{version}/{persona_id}/
    - Module-level model globals (vae, unet, pe, audio_processor, whisper, fp)
    - Avatar cache keyed by (persona_id, version)
    - shutil.move final mp4 to caller-specified output_path
    - Optional chunking for long audio (>MUSETALK_CHUNK_THRESHOLD_S)

Caller contract matches worker.py: render_avatar(ref_video, audio, output_path, fps)
returns a dict containing output_path and metadata.
"""

from __future__ import annotations

import copy
import glob
import hashlib
import json
import logging
import os
import pickle
import queue
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

import cv2
import numpy as np
import torch
from tqdm import tqdm

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Module-level globals populated by load_model() on first call.
# ---------------------------------------------------------------------------
_model: dict | None = None
_device: str | None = None

vae = None
unet = None
pe = None
timesteps = None
audio_processor = None
whisper = None
fp = None
weight_dtype = None

VERSION = "v15"
EXTRA_MARGIN = 10
PARSING_MODE = "jaw"
AUDIO_PAD_LEFT = 2
AUDIO_PAD_RIGHT = 2
LEFT_CHEEK_WIDTH = 90
RIGHT_CHEEK_WIDTH = 90
DEFAULT_BATCH_SIZE = int(os.environ.get("MUSETALK_BATCH_SIZE", "8"))
SKIP_SAVE_IMAGES = False

CHUNK_THRESHOLD_S = float(os.environ.get("MUSETALK_CHUNK_THRESHOLD_S", "60"))
CHUNK_SIZE_S = float(os.environ.get("MUSETALK_CHUNK_SIZE_S", "30"))

AVATAR_ROOT = Path("/models/avatars")

_avatar_cache: dict[str, "Avatar"] = {}


# ---------------------------------------------------------------------------
# Model bootstrap
# ---------------------------------------------------------------------------
def load_model() -> dict:
    """Load MuseTalk v1.5 models once, cache for reuse."""
    global _model, _device
    global vae, unet, pe, timesteps, audio_processor, whisper, fp, weight_dtype

    if _model is not None:
        return _model

    t0 = time.time()

    # dwpose expects /models/musetalk/models to resolve to /models
    dwpose_link = "/models/musetalk/models"
    if not os.path.exists(dwpose_link):
        try:
            os.symlink("/models", dwpose_link)
        except FileExistsError:
            pass

    # musetalk imports are rooted under /models/musetalk-src (on PYTHONPATH)
    # but internal utils do relative file lookups against cwd /models/musetalk
    prev_cwd = os.getcwd()
    os.chdir("/models/musetalk")

    try:
        from musetalk.utils.utils import load_all_model
        from musetalk.utils.face_parsing import FaceParsing
        from musetalk.utils.audio_processor import AudioProcessor
        from transformers import WhisperModel
    finally:
        # stay in /models/musetalk for downstream imports that also do rel lookups
        pass

    device = (
        "cuda"
        if torch.cuda.is_available()
        else ("mps" if torch.backends.mps.is_available() else "cpu")
    )

    vae_l, unet_l, pe_l = load_all_model(
        unet_model_path="/models/musetalkV15/unet.pth",
        vae_type="sd-vae",
        unet_config="/models/musetalkV15/musetalk.json",
        device=device,
    )
    ts = torch.tensor([0], device=device)

    pe_l = pe_l.half().to(device)
    vae_l.vae = vae_l.vae.half().to(device)
    unet_l.model = unet_l.model.half().to(device)

    dtype = unet_l.model.dtype
    ap = AudioProcessor(feature_extractor_path="/models/whisper")
    whisper_l = (
        WhisperModel.from_pretrained("/models/whisper")
        .to(device=device, dtype=dtype)
        .eval()
    )
    whisper_l.requires_grad_(False)

    fp_l = FaceParsing(
        left_cheek_width=LEFT_CHEEK_WIDTH,
        right_cheek_width=RIGHT_CHEEK_WIDTH,
    )

    vae = vae_l
    unet = unet_l
    pe = pe_l
    timesteps = ts
    audio_processor = ap
    whisper = whisper_l
    weight_dtype = dtype
    fp = fp_l
    _device = device

    _model = {
        "vae": vae,
        "unet": unet,
        "pe": pe,
        "timesteps": timesteps,
        "audio_processor": audio_processor,
        "whisper": whisper,
        "fp": fp,
        "weight_dtype": weight_dtype,
        "device": device,
        "version": VERSION,
        "load_time_s": round(time.time() - t0, 2),
    }
    logger.info("MuseTalk models loaded device=%s dtype=%s in %.2fs",
                device, dtype, _model["load_time_s"])
    return _model


# ---------------------------------------------------------------------------
# Utility helpers
# ---------------------------------------------------------------------------
def _probe_duration(path: str) -> float:
    try:
        out = subprocess.check_output(
            [
                "ffprobe", "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=nw=1:nk=1",
                path,
            ],
            stderr=subprocess.STDOUT,
            timeout=30,
        )
        return float(out.decode().strip())
    except Exception:
        return 0.0


def _persona_id(ref_video_path: str, override: str | None) -> str:
    if override:
        return override
    try:
        st = os.stat(ref_video_path)
        key = f"{ref_video_path}:{st.st_size}:{int(st.st_mtime)}"
    except OSError:
        key = ref_video_path
    return hashlib.sha256(key.encode()).hexdigest()[:16]


def _video2imgs(video_path: str, save_path: str, ext: str = ".png", cut_frame: int = 10_000_000) -> None:
    os.makedirs(save_path, exist_ok=True)
    cap = cv2.VideoCapture(video_path)
    count = 0
    while True:
        if count > cut_frame:
            break
        ret, frame = cap.read()
        if not ret:
            break
        cv2.imwrite(f"{save_path}/{count:08d}{ext}", frame)
        count += 1
    cap.release()


def _osmakedirs(paths: list[str]) -> None:
    for p in paths:
        os.makedirs(p, exist_ok=True)


# ---------------------------------------------------------------------------
# Avatar (ported from upstream Avatar class, no input()/sys.exit())
# ---------------------------------------------------------------------------
class Avatar:
    @torch.no_grad()
    def __init__(
        self,
        avatar_id: str,
        video_path: str,
        bbox_shift: int = 0,
        batch_size: int = DEFAULT_BATCH_SIZE,
        force_recreate: bool = False,
    ) -> None:
        from musetalk.utils.blending import get_image_prepare_material

        self.avatar_id = avatar_id
        self.video_path = video_path
        self.bbox_shift = bbox_shift
        self.batch_size = batch_size
        self.version = VERSION
        self.base_path = str(AVATAR_ROOT / self.version / avatar_id)
        self.avatar_path = self.base_path
        self.full_imgs_path = f"{self.avatar_path}/full_imgs"
        self.coords_path = f"{self.avatar_path}/coords.pkl"
        self.latents_out_path = f"{self.avatar_path}/latents.pt"
        self.video_out_path = f"{self.avatar_path}/vid_output"
        self.mask_out_path = f"{self.avatar_path}/mask"
        self.mask_coords_path = f"{self.avatar_path}/mask_coords.pkl"
        self.avatar_info_path = f"{self.avatar_path}/avator_info.json"  # sic (upstream typo)
        self.avatar_info = {
            "avatar_id": avatar_id,
            "video_path": video_path,
            "bbox_shift": bbox_shift,
            "version": self.version,
        }

        self._init(force_recreate=force_recreate)

    # ------------------------------------------------------------------
    def _init(self, force_recreate: bool) -> None:
        if os.path.exists(self.avatar_path):
            need_recreate = force_recreate
            if not need_recreate and os.path.exists(self.avatar_info_path):
                try:
                    with open(self.avatar_info_path) as f:
                        info = json.load(f)
                    if info.get("bbox_shift") != self.avatar_info["bbox_shift"]:
                        logger.info(
                            "bbox_shift changed (%s -> %s); recreating avatar %s",
                            info.get("bbox_shift"), self.avatar_info["bbox_shift"],
                            self.avatar_id,
                        )
                        need_recreate = True
                except Exception as e:
                    logger.warning("Could not read avatar info %s: %s; recreating",
                                   self.avatar_info_path, e)
                    need_recreate = True

            if need_recreate:
                shutil.rmtree(self.avatar_path)
                self._create_dirs_and_prepare()
            else:
                self._load_cached()
        else:
            self._create_dirs_and_prepare()

    def _create_dirs_and_prepare(self) -> None:
        _osmakedirs([
            self.avatar_path,
            self.full_imgs_path,
            self.video_out_path,
            self.mask_out_path,
        ])
        self._prepare_material()

    def _load_cached(self) -> None:
        self.input_latent_list_cycle = torch.load(self.latents_out_path, weights_only=False)
        with open(self.coords_path, "rb") as f:
            self.coord_list_cycle = pickle.load(f)
        input_img_list = sorted(
            glob.glob(os.path.join(self.full_imgs_path, "*.[jpJP][pnPN]*[gG]")),
            key=lambda x: int(os.path.splitext(os.path.basename(x))[0]),
        )
        from musetalk.utils.preprocessing import read_imgs
        self.frame_list_cycle = read_imgs(input_img_list)
        with open(self.mask_coords_path, "rb") as f:
            self.mask_coords_list_cycle = pickle.load(f)
        input_mask_list = sorted(
            glob.glob(os.path.join(self.mask_out_path, "*.[jpJP][pnPN]*[gG]")),
            key=lambda x: int(os.path.splitext(os.path.basename(x))[0]),
        )
        self.mask_list_cycle = read_imgs(input_mask_list)

    # ------------------------------------------------------------------
    @torch.no_grad()
    def _prepare_material(self) -> None:
        from musetalk.utils.preprocessing import get_landmark_and_bbox, read_imgs
        from musetalk.utils.blending import get_image_prepare_material

        logger.info("preparing avatar %s from %s", self.avatar_id, self.video_path)
        with open(self.avatar_info_path, "w") as f:
            json.dump(self.avatar_info, f)

        if os.path.isfile(self.video_path):
            _video2imgs(self.video_path, self.full_imgs_path, ext=".png")
        else:
            logger.info("copying frames from %s", self.video_path)
            files = sorted(
                os.listdir(self.video_path),
                key=lambda x: int(os.path.splitext(x)[0]),
            )
            for name in files:
                shutil.copyfile(
                    os.path.join(self.video_path, name),
                    os.path.join(self.full_imgs_path, name),
                )

        input_img_list = sorted(
            glob.glob(os.path.join(self.full_imgs_path, "*.[jpJP][pnPN]*[gG]")),
            key=lambda x: int(os.path.splitext(os.path.basename(x))[0]),
        )
        if not input_img_list:
            raise RuntimeError(f"no frames extracted from {self.video_path}")

        coord_list, frame_list = get_landmark_and_bbox(input_img_list, self.bbox_shift)
        input_latent_list: list = []
        idx = -1
        coord_placeholder = (0.0, 0.0, 0.0, 0.0)
        for bbox, frame in zip(coord_list, frame_list):
            idx += 1
            if bbox == coord_placeholder:
                continue
            x1, y1, x2, y2 = bbox
            y2 = min(y2 + EXTRA_MARGIN, frame.shape[0])
            crop_frame = frame[y1:y2, x1:x2]
            resized = cv2.resize(crop_frame, (256, 256), interpolation=cv2.INTER_LANCZOS4)
            latents = vae.get_latents_for_unet(resized)
            input_latent_list.append(latents)

        if not input_latent_list:
            raise RuntimeError("no usable face bboxes found in reference video")

        self.frame_list_cycle = frame_list + frame_list[::-1]
        self.coord_list_cycle = coord_list + coord_list[::-1]
        self.input_latent_list_cycle = input_latent_list + input_latent_list[::-1]

        self.mask_coords_list_cycle = []
        self.mask_list_cycle = []
        for i, frame in enumerate(tqdm(self.frame_list_cycle, desc="mask prep")):
            cv2.imwrite(f"{self.full_imgs_path}/{i:08d}.png", frame)
            bbox = self.coord_list_cycle[i]
            x1, y1, x2, y2 = bbox
            y2 = min(y2 + EXTRA_MARGIN, frame.shape[0])
            crop_box = (x1, y1, x2, y2)
            mask, crop_box = get_image_prepare_material(frame, crop_box, fp=fp, mode=PARSING_MODE)
            cv2.imwrite(f"{self.mask_out_path}/{i:08d}.png", mask)
            self.mask_coords_list_cycle.append(crop_box)
            self.mask_list_cycle.append(mask)

        with open(self.mask_coords_path, "wb") as f:
            pickle.dump(self.mask_coords_list_cycle, f)
        with open(self.coords_path, "wb") as f:
            pickle.dump(self.coord_list_cycle, f)
        torch.save(self.input_latent_list_cycle, self.latents_out_path)

    # ------------------------------------------------------------------
    def _process_frames(self, res_frame_queue: "queue.Queue", video_len: int, tmp_dir: str) -> None:
        from musetalk.utils.blending import get_image_blending

        idx = 0
        pbar = tqdm(total=video_len, desc="blend")
        while idx < video_len:
            try:
                res_frame = res_frame_queue.get(block=True, timeout=1)
            except queue.Empty:
                continue
            bbox = self.coord_list_cycle[idx % len(self.coord_list_cycle)]
            ori_frame = copy.deepcopy(self.frame_list_cycle[idx % len(self.frame_list_cycle)])
            x1, y1, x2, y2 = bbox
            try:
                res_frame = cv2.resize(res_frame.astype(np.uint8), (x2 - x1, y2 - y1))
            except Exception:
                idx += 1
                pbar.update(1)
                continue
            mask = self.mask_list_cycle[idx % len(self.mask_list_cycle)]
            mask_crop_box = self.mask_coords_list_cycle[idx % len(self.mask_coords_list_cycle)]
            combined = get_image_blending(ori_frame, res_frame, bbox, mask, mask_crop_box)
            cv2.imwrite(f"{tmp_dir}/{idx:08d}.png", combined)
            idx += 1
            pbar.update(1)
        pbar.close()

    # ------------------------------------------------------------------
    @torch.no_grad()
    def inference(
        self,
        audio_path: str,
        output_path: str,
        fps: int,
        out_vid_name: str = "out",
    ) -> str:
        from musetalk.utils.utils import datagen

        tmp_dir = f"{self.avatar_path}/tmp"
        if os.path.exists(tmp_dir):
            shutil.rmtree(tmp_dir)
        os.makedirs(tmp_dir, exist_ok=True)

        t0 = time.time()
        whisper_features, librosa_length = audio_processor.get_audio_feature(
            audio_path, weight_dtype=weight_dtype
        )
        whisper_chunks = audio_processor.get_whisper_chunk(
            whisper_features,
            _device,
            weight_dtype,
            whisper,
            librosa_length,
            fps=fps,
            audio_padding_length_left=AUDIO_PAD_LEFT,
            audio_padding_length_right=AUDIO_PAD_RIGHT,
        )

        video_num = len(whisper_chunks)
        res_frame_queue: queue.Queue = queue.Queue()
        blend_thread = threading.Thread(
            target=self._process_frames,
            args=(res_frame_queue, video_num, tmp_dir),
        )
        blend_thread.start()

        gen = datagen(whisper_chunks, self.input_latent_list_cycle, self.batch_size)
        total_batches = int(np.ceil(float(video_num) / self.batch_size))
        for _, (whisper_batch, latent_batch) in enumerate(
            tqdm(gen, total=total_batches, desc="infer"),
        ):
            audio_feature_batch = pe(whisper_batch.to(_device))
            latent_batch = latent_batch.to(device=_device, dtype=weight_dtype)
            pred_latents = unet.model(
                latent_batch,
                timesteps,
                encoder_hidden_states=audio_feature_batch,
            ).sample
            pred_latents = pred_latents.to(device=_device, dtype=weight_dtype)
            recon = vae.decode_latents(pred_latents)
            for frame in recon:
                res_frame_queue.put(frame)

        blend_thread.join()

        # ffmpeg: images -> silent mp4 -> mux audio
        silent = f"{self.avatar_path}/{out_vid_name}_silent.mp4"
        muxed = f"{self.avatar_path}/{out_vid_name}.mp4"
        for p in (silent, muxed):
            if os.path.exists(p):
                os.remove(p)

        subprocess.check_call([
            "ffmpeg", "-y", "-v", "warning",
            "-r", str(fps),
            "-f", "image2",
            "-i", f"{tmp_dir}/%08d.png",
            "-vcodec", "libx264",
            "-vf", "format=yuv420p",
            "-crf", "18",
            silent,
        ])
        subprocess.check_call([
            "ffmpeg", "-y", "-v", "warning",
            "-i", audio_path,
            "-i", silent,
            "-c:v", "copy",
            muxed,
        ])

        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
        shutil.move(muxed, output_path)
        try:
            os.remove(silent)
        except OSError:
            pass
        shutil.rmtree(tmp_dir, ignore_errors=True)

        logger.info("inference done in %.2fs -> %s", time.time() - t0, output_path)
        return output_path


# ---------------------------------------------------------------------------
# Public entry point (called by worker.py)
# ---------------------------------------------------------------------------
def render_avatar(
    ref_video_path: str,
    audio_path: str,
    output_path: str,
    fps: int = 25,
    *,
    persona_id: str | None = None,
    bbox_shift: int = 0,
    batch_size: int | None = None,
    force_recreate: bool = False,
) -> dict:
    """Render a lip-synced mp4 with MuseTalk v1.5 realtime Avatar."""
    load_model()

    pid = _persona_id(ref_video_path, persona_id)
    cache_key = f"{pid}:{VERSION}"
    avatar = _avatar_cache.get(cache_key)
    if avatar is None:
        avatar = Avatar(
            avatar_id=pid,
            video_path=ref_video_path,
            bbox_shift=bbox_shift,
            batch_size=batch_size or DEFAULT_BATCH_SIZE,
            force_recreate=force_recreate,
        )
        _avatar_cache[cache_key] = avatar

    t0 = time.time()
    audio_duration = _probe_duration(audio_path)

    if audio_duration and audio_duration > CHUNK_THRESHOLD_S:
        output_path = _chunked_render(avatar, audio_path, output_path, fps, audio_duration)
    else:
        avatar.inference(audio_path, output_path, fps=fps)

    render_time = time.time() - t0
    out_duration = _probe_duration(output_path)
    size_bytes = os.path.getsize(output_path) if os.path.exists(output_path) else 0

    return {
        "output_path": output_path,
        "duration_seconds": out_duration,
        "size_bytes": size_bytes,
        "fps": fps,
        "render_time_s": round(render_time, 2),
        "persona_id": pid,
        "version": VERSION,
        "gpu": torch.cuda.get_device_name(0) if torch.cuda.is_available() else "cpu",
        "dtype": str(weight_dtype),
        "batch_size": avatar.batch_size,
    }


def _chunked_render(
    avatar: Avatar,
    audio_path: str,
    output_path: str,
    fps: int,
    audio_duration: float,
) -> str:
    """Split long audio into CHUNK_SIZE_S chunks, render each, concat."""
    work_dir = f"{avatar.avatar_path}/chunks"
    if os.path.exists(work_dir):
        shutil.rmtree(work_dir)
    os.makedirs(work_dir, exist_ok=True)

    n = int(np.ceil(audio_duration / CHUNK_SIZE_S))
    logger.info("chunking audio: duration=%.1fs chunks=%d size=%.1fs",
                audio_duration, n, CHUNK_SIZE_S)
    chunk_outputs: list[str] = []
    for i in range(n):
        start = i * CHUNK_SIZE_S
        chunk_audio = f"{work_dir}/audio_{i:04d}.wav"
        chunk_out = f"{work_dir}/video_{i:04d}.mp4"
        subprocess.check_call([
            "ffmpeg", "-y", "-v", "warning",
            "-ss", str(start),
            "-t", str(CHUNK_SIZE_S),
            "-i", audio_path,
            "-c:a", "pcm_s16le",
            chunk_audio,
        ])
        avatar.inference(chunk_audio, chunk_out, fps=fps, out_vid_name=f"chunk_{i}")
        chunk_outputs.append(chunk_out)

    list_file = f"{work_dir}/concat.txt"
    with open(list_file, "w") as f:
        for p in chunk_outputs:
            f.write(f"file '{p}'\n")

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    if os.path.exists(output_path):
        os.remove(output_path)
    subprocess.check_call([
        "ffmpeg", "-y", "-v", "warning",
        "-f", "concat", "-safe", "0",
        "-i", list_file,
        "-c", "copy",
        output_path,
    ])
    shutil.rmtree(work_dir, ignore_errors=True)
    return output_path


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(name)s %(message)s")
    if len(sys.argv) < 4:
        print("usage: render.py <ref_video> <audio> <output_mp4> [fps]")
        sys.exit(1)
    ref, aud, out = sys.argv[1], sys.argv[2], sys.argv[3]
    f = int(sys.argv[4]) if len(sys.argv) > 4 else 25
    result = render_avatar(ref, aud, out, fps=f)
    print(json.dumps(result, indent=2))
