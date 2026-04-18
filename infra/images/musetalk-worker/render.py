"""MuseTalk render bootstrap helpers.

This module intentionally stops at model/bootstrap verification for Phase 4.
The PRD acceptance line is GPU provisioning, image build, deployment, and in-pod
CUDA/PyTorch validation. Full video generation lands in a later phase.
"""

import hashlib
import logging
import os
import time
from pathlib import Path

import torch

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

    # Upstream musetalk loads weights via relative paths like "models/sd-vae".
    # download_models.py writes them under MODEL_CACHE_DIR (e.g. /models/models/sd-vae),
    # so chdir into MODEL_CACHE_DIR's parent of those nested model dirs before loading.
    cache_dir = os.environ.get("MODEL_CACHE_DIR", "/models")
    if Path(cache_dir).is_dir():
        os.chdir(cache_dir)
        log.info("Changed working directory to %s for model resolution", cache_dir)

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


def render_avatar(
    reference_image_path: str,
    audio_path: str,
    output_path: str,
    fps: int = 25,
) -> dict:
    """Bootstrap-only placeholder for Phase 4.

    We validate that the worker can load MuseTalk dependencies and see CUDA.
    Full render output is intentionally not implemented in this phase.
    """
    del reference_image_path, audio_path, output_path, fps

    t0 = time.time()
    model = load_model()
    elapsed = time.time() - t0

    return {
        "output_path": None,
        "render_time_s": round(elapsed, 2),
        "gpu": str(model["device"]),
        "dtype": str(model["dtype"]),
        "bootstrap_only": True,
    }
