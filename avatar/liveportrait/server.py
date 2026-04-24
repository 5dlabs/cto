"""
FastAPI wrapper around LivePortrait's inference.py.

Exposes:
  GET  /health              -> liveness probe (OVH AI Deploy requires 200 on /health)
  GET  /                    -> basic info
  POST /animate             -> multipart:
                                 source   (image or video, required)
                                 driving  (video or .pkl template, required)
                                 flag_crop_driving_video (bool, optional)
                                 driving_option (str, optional)
                                 animation_region (str, optional)
                               -> video/mp4

Design notes:
  * LivePortrait's CLI (`python inference.py --source <p> --driving <p>`) is the most
    robust entry point. Reimplementing the pipeline in-process requires carrying tyro's
    ArgumentConfig parsing, which has extra Gradio/server fields we don't want to expose.
  * First-request latency on cold GPU is ~20-40s (model load). OVH AI Deploy scales to
    zero when idle; keep that in mind for /health timeouts on boot.
  * Concurrency: uvicorn --workers 1. LivePortrait is GPU-bound and shares a single
    CUDA context; serial requests are safer than parallel.
"""

from __future__ import annotations

import asyncio
import logging
import os
import shutil
import subprocess
import tempfile
import time
import uuid
from pathlib import Path

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.responses import FileResponse, JSONResponse

LOG = logging.getLogger("liveportrait-server")
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(name)s %(message)s")

LIVEPORTRAIT_DIR = Path(os.environ.get("LIVEPORTRAIT_DIR", "/workspace/LivePortrait"))
INFERENCE_SCRIPT = LIVEPORTRAIT_DIR / "inference.py"

# One lock ensures only one LivePortrait subprocess at a time (GPU memory).
_GPU_LOCK = asyncio.Lock()

app = FastAPI(title="LivePortrait", version="1.0.0")


@app.get("/")
async def root() -> dict:
    return {
        "service": "liveportrait",
        "version": "1.0.0",
        "liveportrait_dir": str(LIVEPORTRAIT_DIR),
        "endpoints": ["/health", "/animate"],
    }


@app.get("/health")
async def health() -> dict:
    ok = INFERENCE_SCRIPT.exists()
    if not ok:
        raise HTTPException(status_code=503, detail=f"inference.py not found at {INFERENCE_SCRIPT}")
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


@app.post("/animate")
async def animate(
    source: UploadFile = File(..., description="source portrait image (jpg/png) or video"),
    driving: UploadFile = File(..., description="driving video (mp4) or .pkl template"),
    flag_crop_driving_video: bool = Form(False),
    driving_option: str = Form("expression-friendly"),
    animation_region: str = Form("all"),
    flag_stitching: bool = Form(True),
    flag_relative_motion: bool = Form(True),
    flag_pasteback: bool = Form(True),
    flag_do_crop: bool = Form(True),
    driving_multiplier: float = Form(1.0),
) -> FileResponse:
    if driving_option not in ("expression-friendly", "pose-friendly"):
        raise HTTPException(status_code=400, detail="driving_option must be 'expression-friendly' or 'pose-friendly'")
    if animation_region not in ("exp", "pose", "lip", "eyes", "all"):
        raise HTTPException(status_code=400, detail="animation_region must be one of exp|pose|lip|eyes|all")

    job_id = uuid.uuid4().hex[:12]
    work = Path(tempfile.mkdtemp(prefix=f"lp-{job_id}-"))
    try:
        src_path = work / f"source.{_ext_for(source, 'jpg')}"
        drv_path = work / f"driving.{_ext_for(driving, 'mp4')}"
        out_dir = work / "out"
        out_dir.mkdir()

        await _save_upload(source, src_path)
        await _save_upload(driving, drv_path)

        cmd = [
            "python", str(INFERENCE_SCRIPT),
            "--source", str(src_path),
            "--driving", str(drv_path),
            "--output-dir", str(out_dir),
            "--driving-option", driving_option,
            "--animation-region", animation_region,
            "--driving-multiplier", str(driving_multiplier),
        ]
        # boolean flags: LivePortrait uses tyro, which accepts --flag / --no-flag
        cmd += ["--flag-crop-driving-video"] if flag_crop_driving_video else ["--no-flag-crop-driving-video"]
        cmd += ["--flag-stitching"] if flag_stitching else ["--no-flag-stitching"]
        cmd += ["--flag-relative-motion"] if flag_relative_motion else ["--no-flag-relative-motion"]
        cmd += ["--flag-pasteback"] if flag_pasteback else ["--no-flag-pasteback"]
        cmd += ["--flag-do-crop"] if flag_do_crop else ["--no-flag-do-crop"]

        LOG.info("job=%s cmd=%s", job_id, " ".join(cmd))

        async with _GPU_LOCK:
            t0 = time.monotonic()
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                cwd=str(LIVEPORTRAIT_DIR),
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
                detail={"error": "liveportrait failed", "rc": proc.returncode, "log_tail": log_tail},
            )

        # LivePortrait writes `<source>--<driving>.mp4` (and a _concat variant). Pick the
        # non-concat main output.
        candidates = sorted(out_dir.glob("*.mp4"))
        if not candidates:
            raise HTTPException(
                status_code=500,
                detail={"error": "no output mp4 produced", "log_tail": log_tail},
            )
        primary = next((c for c in candidates if "_concat" not in c.name), candidates[0])

        # Stage the output outside the tempdir so the cleanup doesn't race FileResponse.
        staged = Path(tempfile.gettempdir()) / f"lp-out-{job_id}.mp4"
        shutil.copy2(primary, staged)

        LOG.info("job=%s done elapsed=%.1fs out=%s size=%d", job_id, elapsed, staged.name, staged.stat().st_size)
        return FileResponse(
            path=str(staged),
            media_type="video/mp4",
            filename=f"liveportrait-{job_id}.mp4",
            headers={"X-LivePortrait-Elapsed-S": f"{elapsed:.2f}", "X-LivePortrait-Job-Id": job_id},
        )
    finally:
        shutil.rmtree(work, ignore_errors=True)


@app.exception_handler(Exception)
async def unhandled_exc(_request, exc: Exception):  # noqa: ANN001
    LOG.exception("unhandled error: %s", exc)
    return JSONResponse(status_code=500, content={"error": str(exc)})
