"""Single-GPU HunyuanVideo-Avatar render helpers."""

from __future__ import annotations

import csv
import logging
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

import torch

log = logging.getLogger("render")

MODEL_DIR = Path(os.environ.get("MODEL_CACHE_DIR", "/models/hunyuan-avatar"))
REPO_DIR = Path(os.environ.get("HUNYUAN_REPO_DIR", str(MODEL_DIR / "hymm_sp")))
OUTPUT_ROOT = Path(os.environ.get("OUTPUT_DIR", "/tmp/renders/hunyuan-avatar"))
CHECKPOINT_PATH = Path(
    os.environ.get(
        "HUNYUAN_CHECKPOINT",
        str(MODEL_DIR / "ckpts/hunyuan-video-t2v-720p/transformers/mp_rank_00_model_states_fp8.pt"),
    )
)
_SAMPLE_GPU_POOR = REPO_DIR / "sample_gpu_poor.py"
_PIPELINE_READY = False


def _require_path(path: Path, description: str) -> Path:
    if not path.exists():
        raise FileNotFoundError(f"{description} not found: {path}")
    return path


def _pipeline_env() -> dict[str, str]:
    env = os.environ.copy()
    env.setdefault("PYTHONPATH", str(REPO_DIR))
    env["PYTHONPATH"] = f"{REPO_DIR}:{env['PYTHONPATH']}" if env.get("PYTHONPATH") else str(REPO_DIR)
    env.setdefault("MODEL_BASE", str(MODEL_DIR / "weights"))
    env.setdefault("CUDA_VISIBLE_DEVICES", "0")
    env.setdefault("DISABLE_SP", "1")
    env.setdefault("CPU_OFFLOAD", "0")
    env.setdefault("USE_DEEPCACHE", "1")
    env.setdefault("USE_FP8", "1")
    return env


def load_model() -> dict[str, str]:
    global _PIPELINE_READY

    _require_path(REPO_DIR, "HunyuanVideo-Avatar repo")
    _require_path(_SAMPLE_GPU_POOR, "sample_gpu_poor.py")
    _require_path(CHECKPOINT_PATH, "FP8 checkpoint")

    device = "cuda" if torch.cuda.is_available() else "cpu"
    if device != "cuda":
        raise RuntimeError("HunyuanVideo-Avatar requires CUDA for inference")

    if not _PIPELINE_READY:
        log.info("HunyuanVideo-Avatar pipeline ready on %s using checkpoint %s", device, CHECKPOINT_PATH)
        _PIPELINE_READY = True

    return {
        "device": device,
        "checkpoint": str(CHECKPOINT_PATH),
        "repo_dir": str(REPO_DIR),
    }


def render_avatar(ref_image_path: str, audio_path: str, out_path: str) -> None:
    load_model()

    ref_image = _require_path(Path(ref_image_path), "Reference image")
    audio = _require_path(Path(audio_path), "Audio file")
    output_path = Path(out_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(dir=str(OUTPUT_ROOT)) as tmpdir:
        tmpdir_path = Path(tmpdir)
        jobs_csv = tmpdir_path / "job.csv"
        save_path = tmpdir_path / "results"
        save_path.mkdir(parents=True, exist_ok=True)

        with jobs_csv.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.writer(handle)
            writer.writerow(["image_path", "audio_path"])
            writer.writerow([str(ref_image.resolve()), str(audio.resolve())])

        cmd = [
            "python3",
            str(_SAMPLE_GPU_POOR),
            "--input",
            str(jobs_csv),
            "--ckpt",
            str(CHECKPOINT_PATH),
            "--sample-n-frames",
            "129",
            "--seed",
            os.environ.get("HUNYUAN_SEED", "128"),
            "--image-size",
            "704",
            "--cfg-scale",
            "7.5",
            "--infer-steps",
            "50",
            "--use-deepcache",
            "1",
            "--flow-shift-eval-video",
            os.environ.get("HUNYUAN_FLOW_SHIFT", "5.0"),
            "--save-path",
            str(save_path),
            "--use-fp8",
            "--infer-min",
        ]

        if os.environ.get("CPU_OFFLOAD", "0") == "1":
            cmd.append("--cpu-offload")

        subprocess.run(cmd, cwd=REPO_DIR, env=_pipeline_env(), check=True)

        candidates = sorted(save_path.rglob("*.mp4"))
        if not candidates:
            raise FileNotFoundError(f"No MP4 output produced in {save_path}")

        shutil.copyfile(candidates[0], output_path)
        log.info("Rendered avatar video to %s", output_path)
