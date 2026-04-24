"""
FastAPI wrapper around EchoMimicV3-Flash inference.

Exposes:
  GET  /health   -> liveness probe (OVH AI Deploy requires 200 on /health)
  GET  /         -> basic info + resolved model paths
  POST /animate  -> multipart:
                     source   (image, required)   -> --image_path
                     audio    (wav/mp3, required) -> --audio_path
                     prompt   (str, optional)
                   -> video/mp4

Invokes upstream `infer_flash.py` per antgroup/echomimic_v3 @ 7e89489c's run_flash.sh,
pointing at pretrained weights baked into the image under ECHOMIMIC_PRETRAINED_DIR.
"""

from __future__ import annotations

import asyncio
import logging
import os
import shutil
import tempfile
import time
import uuid
from pathlib import Path

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.responses import FileResponse, JSONResponse

LOG = logging.getLogger("echomimic-server")
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(name)s %(message)s")

ECHOMIMIC_DIR = Path(os.environ.get("ECHOMIMIC_DIR", "/workspace/EchoMimicV3"))
PRETRAINED_DIR = Path(os.environ.get("ECHOMIMIC_PRETRAINED_DIR", str(ECHOMIMIC_DIR / "pretrained_weights")))

# Resolved at import time — fail fast in logs if layout is wrong.
INFER_SCRIPT = ECHOMIMIC_DIR / "infer_flash.py"
CONFIG_PATH = ECHOMIMIC_DIR / "config" / "config.yaml"
WAN_MODEL_DIR = PRETRAINED_DIR / "Wan2.1-Fun-V1.1-1.3B-InP"
TRANSFORMER_PATH = PRETRAINED_DIR / "EchoMimicV3" / "transformer" / "diffusion_pytorch_model.safetensors"
WAV2VEC_DIR = PRETRAINED_DIR / "chinese-wav2vec2-base"

DEFAULT_PROMPT = "A person is speaking."


def _env_str(name: str, default: str) -> str:
    value = os.environ.get(name)
    if value is None:
        return default
    value = value.strip()
    if not value:
        raise ValueError(f"{name} must not be empty")
    return value


def _env_int_str(name: str, default: int, *, min_value: int = 1) -> str:
    value = _env_str(name, str(default))
    try:
        parsed = int(value)
    except ValueError as exc:
        raise ValueError(f"{name} must be an integer, got {value!r}") from exc
    if parsed < min_value:
        raise ValueError(f"{name} must be >= {min_value}, got {parsed}")
    return str(parsed)


def _env_choice(name: str, default: str, choices: set[str]) -> str:
    value = _env_str(name, default)
    if value not in choices:
        raise ValueError(f"{name} must be one of {sorted(choices)}, got {value!r}")
    return value


SAMPLE_SIZE = (
    _env_int_str("ECHOMIMIC_SAMPLE_HEIGHT", 768, min_value=16),
    _env_int_str("ECHOMIMIC_SAMPLE_WIDTH", 768, min_value=16),
)


# Upstream run_flash.sh defaults, with V100-safe memory/dtype overrides.
FLASH_DEFAULTS = {
    "num_inference_steps": "8",
    "ckpt_idx": "50000",
    "sampler_name": "Flow_Unipc",
    "video_length": _env_int_str("ECHOMIMIC_VIDEO_LENGTH", 65),
    "guidance_scale": "6.0",
    "audio_guidance_scale": "3.0",
    "audio_scale": "1.0",
    "neg_scale": "1.0",
    "neg_steps": "0",
    "teacache_threshold": "0.1",
    "num_skip_start_steps": "5",
    "riflex_k": "6",
    "ulysses_degree": "1",
    "ring_degree": "1",
    "weight_dtype": _env_choice("ECHOMIMIC_WEIGHT_DTYPE", "float16", {"float16", "bfloat16"}),
    "fps": "25",
    "shift": "5.0",
}

_GPU_LOCK = asyncio.Lock()
app = FastAPI(title="EchoMimicV3-Flash", version="0.2.0")


def _layout_status() -> dict:
    return {
        "infer_script": {"path": str(INFER_SCRIPT), "exists": INFER_SCRIPT.exists()},
        "config": {"path": str(CONFIG_PATH), "exists": CONFIG_PATH.exists()},
        "wan_base": {"path": str(WAN_MODEL_DIR), "exists": WAN_MODEL_DIR.exists()},
        "transformer": {"path": str(TRANSFORMER_PATH), "exists": TRANSFORMER_PATH.exists()},
        "wav2vec": {"path": str(WAV2VEC_DIR), "exists": WAV2VEC_DIR.exists()},
    }


@app.get("/")
async def root() -> dict:
    return {
        "service": "echomimic-v3-flash",
        "version": "0.2.0",
        "layout": _layout_status(),
        "defaults": {**FLASH_DEFAULTS, "sample_size": list(SAMPLE_SIZE)},
        "endpoints": ["/health", "/animate"],
    }


@app.get("/health")
async def health() -> dict:
    if not INFER_SCRIPT.exists():
        raise HTTPException(status_code=503, detail=f"missing infer_flash.py at {INFER_SCRIPT}")
    return {"status": "ok"}


def _ext_for(upload: UploadFile, default: str) -> str:
    if upload.filename and "." in upload.filename:
        ext = upload.filename.rsplit(".", 1)[-1].lower()
        if ext and len(ext) <= 5:
            return ext
    return default


async def _save_upload(upload: UploadFile, dest: Path) -> None:
    with dest.open("wb") as f:
        while True:
            chunk = await upload.read(1 << 20)
            if not chunk:
                break
            f.write(chunk)


def _build_cmd(image_path: Path, audio_path: Path, save_path: Path, prompt: str, seed: int) -> list[str]:
    cmd = [
        "python", str(INFER_SCRIPT),
        "--image_path", str(image_path),
        "--audio_path", str(audio_path),
        "--prompt", prompt,
        "--config_path", str(CONFIG_PATH),
        "--model_name", str(WAN_MODEL_DIR),
        "--transformer_path", str(TRANSFORMER_PATH),
        "--wav2vec_model_dir", str(WAV2VEC_DIR),
        "--save_path", str(save_path),
        "--seed", str(seed),
        "--enable_teacache",
        # sample_size requires two ints (h w)
        "--sample_size", *SAMPLE_SIZE,
    ]
    for flag, val in FLASH_DEFAULTS.items():
        cmd.extend([f"--{flag}", val])
    return cmd


@app.post("/animate")
async def animate(
    source: UploadFile = File(..., description="source portrait image (jpg/png)"),
    audio: UploadFile = File(..., description="driving audio (wav/mp3)"),
    prompt: str = Form(DEFAULT_PROMPT),
    seed: int = Form(43),
) -> FileResponse:
    if not INFER_SCRIPT.exists():
        raise HTTPException(status_code=503, detail="infer_flash.py not present")

    job_id = uuid.uuid4().hex[:12]
    work = Path(tempfile.mkdtemp(prefix=f"em-{job_id}-"))
    try:
        src_path = work / f"source.{_ext_for(source, 'jpg')}"
        aud_path = work / f"audio.{_ext_for(audio, 'wav')}"
        out_dir = work / "out"
        out_dir.mkdir()

        await _save_upload(source, src_path)
        await _save_upload(audio, aud_path)

        cmd = _build_cmd(src_path, aud_path, out_dir, prompt, seed)
        LOG.info("job=%s cmd=%s", job_id, " ".join(cmd))

        async with _GPU_LOCK:
            t0 = time.monotonic()
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                cwd=str(ECHOMIMIC_DIR),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
            )
            stdout_bytes, _ = await proc.communicate()
            elapsed = time.monotonic() - t0

        log_tail = (stdout_bytes or b"").decode("utf-8", errors="replace")[-4000:]
        if proc.returncode != 0:
            LOG.error("job=%s rc=%s elapsed=%.1fs log_tail=%s", job_id, proc.returncode, elapsed, log_tail)
            raise HTTPException(
                status_code=500,
                detail={"error": "echomimic failed", "rc": proc.returncode, "log_tail": log_tail},
            )

        # infer_flash.py writes mp4s under --save_path (may nest a subdir).
        candidates = sorted(out_dir.rglob("*.mp4"), key=lambda p: p.stat().st_mtime, reverse=True)
        if not candidates:
            raise HTTPException(
                status_code=500,
                detail={"error": "no output mp4 produced", "log_tail": log_tail},
            )
        primary = candidates[0]
        staged = Path(tempfile.gettempdir()) / f"em-out-{job_id}.mp4"
        shutil.copy2(primary, staged)

        LOG.info("job=%s done elapsed=%.1fs out=%s size=%d",
                 job_id, elapsed, staged.name, staged.stat().st_size)
        return FileResponse(
            path=str(staged),
            media_type="video/mp4",
            filename=f"echomimic-{job_id}.mp4",
            headers={"X-EchoMimic-Elapsed-S": f"{elapsed:.2f}", "X-EchoMimic-Job-Id": job_id},
        )
    finally:
        shutil.rmtree(work, ignore_errors=True)


@app.exception_handler(Exception)
async def unhandled_exc(_request, exc: Exception):  # noqa: ANN001
    LOG.exception("unhandled error: %s", exc)
    return JSONResponse(status_code=500, content={"error": str(exc)})
