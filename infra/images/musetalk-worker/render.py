"""MuseTalk render helpers.

Phase 5: wires upstream `musetalk-src/scripts/inference.py::main` into
`render_avatar()` so the worker produces real lip-synced mp4 output.
"""

import argparse
import hashlib
import json
import logging
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import torch
import yaml

log = logging.getLogger("render")

_model = None
_device = None


def get_cache_key(persona_id: str, audio_hash: str) -> str:
    """Deterministic cache key for rendered clips."""
    return hashlib.sha256(f"{persona_id}:{audio_hash}".encode()).hexdigest()[:16]


def _musetalk_repo_dir() -> str:
    cache_dir = os.environ.get("MODEL_CACHE_DIR", "/models")
    return os.path.join(cache_dir, "musetalk")


def ensure_model_repo() -> str:
    repo_dir = _musetalk_repo_dir()
    if not Path(repo_dir).exists():
        raise FileNotFoundError(
            f"MuseTalk repo not found at {repo_dir}. Run download_models.py first."
        )
    return repo_dir


def load_model():
    """Lazy-load MuseTalk modules to verify GPU bootstrap works."""
    global _model, _device

    if _model is not None:
        return _model

    repo_dir = ensure_model_repo()
    dtype_str = os.environ.get("MODEL_DTYPE", "float16")
    dtype = torch.float16 if dtype_str == "float16" else torch.float32

    _device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    log.info("Loading MuseTalk bootstrap on %s with dtype=%s", _device, dtype)

    import sys
    if repo_dir not in sys.path:
        sys.path.insert(0, repo_dir)

    # Upstream musetalk loads weights via relative paths like "models/musetalkV15/musetalk.json".
    # download_models.py writes them under MODEL_CACHE_DIR (e.g. /models/musetalkV15/).
    # We chdir to / so that the relative path "models/musetalkV15/..." resolves to
    # /models/musetalkV15/... correctly.
    os.chdir("/")
    log.info("Changed working directory to / for model resolution")

    # Importing and initializing the upstream model stack is enough for Phase 4
    # to prove the container can bootstrap on a GPU node.
    from musetalk.utils.utils import load_all_model

    _model = {
        "models": load_all_model(),
        "dtype": dtype,
        "device": _device,
        "repo_dir": repo_dir,
    }

    if torch.cuda.is_available():
        vram_mb = torch.cuda.memory_allocated() / 1024 / 1024
        log.info("MuseTalk bootstrap loaded on GPU (VRAM: %.1f MB used)", vram_mb)
    else:
        log.warning("MuseTalk bootstrap loaded without CUDA")
    return _model


def _probe_duration(path: str) -> float:
    try:
        out = subprocess.check_output(
            [
                "ffprobe", "-v", "error", "-show_entries", "format=duration",
                "-of", "json", path,
            ],
            stderr=subprocess.STDOUT,
            timeout=30,
        )
        return float(json.loads(out.decode())["format"]["duration"])
    except Exception as e:
        log.warning("ffprobe failed for %s: %s", path, e)
        return 0.0


def render_avatar(
    reference_image_path: str,
    audio_path: str,
    output_path: str,
    fps: int = 25,
) -> dict:
    """Run MuseTalk v1.5 inference and produce an mp4 at `output_path`."""
    t0 = time.time()
    model = load_model()

    # Ensure upstream scripts dir is importable and working dir resolves the
    # relative ./models/... paths used by upstream defaults.
    scripts_dir = "/models/musetalk-src/scripts"
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)

    # musetalk/utils/preprocessing.py does init_model() at module import with two
    # relative paths:
    #   ./musetalk/utils/dwpose/rtmpose-l_8xb32-270e_coco-ubody-wholebody-384x288.py
    #   ./models/dwpose/dw-ll_ucoco_384.pth
    # The config lives under /models/musetalk/musetalk/..., but the checkpoint is
    # at /models/dwpose/... (top-level, sibling of musetalk). No single cwd
    # satisfies both. Symlink /models/musetalk/models -> /models so both resolve
    # when cwd=/models/musetalk.
    musetalk_root = "/models/musetalk"
    models_link = os.path.join(musetalk_root, "models")
    if not os.path.islink(models_link) and not os.path.exists(models_link):
        try:
            os.symlink("/models", models_link)
            log.info("Created symlink %s -> /models", models_link)
        except FileExistsError:
            pass
    os.chdir(musetalk_root)
    log.info("Changed working directory to %s for upstream inference import", musetalk_root)

    from inference import main as musetalk_main  # noqa: WPS433

    cache_dir = os.environ.get("MODEL_CACHE_DIR", "/models")

    with tempfile.TemporaryDirectory(prefix="musetalk_run_", dir="/tmp") as work_dir:
        # Write per-request inference YAML
        cfg_path = os.path.join(work_dir, "inference.yaml")
        task_cfg = {
            "task_0": {
                "video_path": reference_image_path,
                "audio_path": audio_path,
            }
        }
        with open(cfg_path, "w") as f:
            yaml.safe_dump(task_cfg, f)

        # Upstream writes to {result_dir}/{version}/{output_vid_name}
        result_dir = os.path.join(work_dir, "results")
        os.makedirs(result_dir, exist_ok=True)
        output_basename = os.path.basename(output_path) or "output.mp4"
        # Strip extension for output_vid_name (upstream appends .mp4)
        output_name_noext = os.path.splitext(output_basename)[0]

        args = argparse.Namespace(
            ffmpeg_path="/usr/bin",
            gpu_id=0,
            vae_type="sd-vae",
            unet_config=os.path.join(cache_dir, "musetalk/musetalk.json"),
            unet_model_path=os.path.join(cache_dir, "musetalkV15/unet.pth"),
            whisper_dir=os.path.join(cache_dir, "whisper"),
            inference_config=cfg_path,
            bbox_shift=0,
            extra_margin=10,
            fps=fps,
            audio_padding_length_left=2,
            audio_padding_length_right=2,
            batch_size=int(os.environ.get("MUSETALK_BATCH_SIZE", "8")),
            result_dir=result_dir,
            output_vid_name=output_name_noext,
            use_saved_coord=False,
            saved_coord=False,
            use_float16=str(model["dtype"]) == "torch.float16",
            parsing_mode="jaw",
            left_cheek_width=90,
            right_cheek_width=90,
            version="v15",
        )

        log.info("Invoking MuseTalk main(args) with %s", vars(args))
        musetalk_main(args)

        produced = os.path.join(result_dir, "v15", f"{output_name_noext}.mp4")
        if not os.path.exists(produced):
            # Upstream fallback layout (when result_name is absent)
            input_basename = os.path.basename(reference_image_path).split(".")[0]
            audio_basename = os.path.basename(audio_path).split(".")[0]
            alt = os.path.join(result_dir, "v15", f"{input_basename}_{audio_basename}.mp4")
            if os.path.exists(alt):
                produced = alt
            else:
                # Last-ditch: glob any mp4 anywhere under result_dir
                import glob as _glob
                candidates = sorted(_glob.glob(os.path.join(result_dir, "**", "*.mp4"), recursive=True))
                if candidates:
                    produced = candidates[0]
                    log.warning("Using glob-discovered output: %s", produced)
                else:
                    raise RuntimeError(
                        f"MuseTalk did not produce an output video. "
                        f"Checked {produced} and {alt}; result_dir={result_dir}."
                    )

        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        shutil.move(produced, output_path)

    elapsed = time.time() - t0
    duration = _probe_duration(output_path)
    size = os.path.getsize(output_path) if os.path.exists(output_path) else 0
    log.info(
        "Rendered %s (%.2fs duration, %d bytes) in %.2fs",
        output_path, duration, size, elapsed,
    )

    return {
        "output_path": output_path,
        "duration_seconds": round(duration, 2),
        "size_bytes": size,
        "fps": fps,
        "render_time_s": round(elapsed, 2),
        "gpu": str(model["device"]),
        "dtype": str(model["dtype"]),
    }
