"""
FastAPI wrapper around EchoMimicV3 inference.

Exposes:
  GET  /health        -> liveness probe (OVH AI Deploy requires 200 on /health)
  GET  /              -> basic info
  POST /animate       -> multipart:
                           source  (image, required)
                           audio   (wav/mp3, required)
                         -> video/mp4

NOTE: EchoMimicV3's upstream entrypoint may shift; this wrapper shells out to
`python infer.py` (or `infer_echomimic.py`) and auto-detects the script. When
the upstream API stabilizes, swap subprocess for in-process calls.
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

LOG = logging.getLogger("echomimic-server")
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(name)s %(message)s")

ECHOMIMIC_DIR = Path(os.environ.get("ECHOMIMIC_DIR", "/workspace/EchoMimicV3"))

# Probe common entrypoint names; fall back to first infer*.py
def _find_inference_script() -> Path | None:
    for name in ("infer.py", "inference.py", "infer_echomimic.py", "run.py"):
        p = ECHOMIMIC_DIR / name
        if p.exists():
            return p
    for p in ECHOMIMIC_DIR.glob("infer*.py"):
        return p
    return None

_GPU_LOCK = asyncio.Lock()

app = FastAPI(title="EchoMimicV3", version="0.1.0")


@app.get("/")
async def root() -> dict:
    script = _find_inference_script()
    return {
        "service": "echomimic-v3",
        "version": "0.1.0",
        "echomimic_dir": str(ECHOMIMIC_DIR),
        "inference_script": str(script) if script else None,
        "endpoints": ["/health", "/animate"],
    }


@app.get("/health")
async def health() -> dict:
    if not ECHOMIMIC_DIR.exists():
        raise HTTPException(status_code=503, detail=f"ECHOMIMIC_DIR missing: {ECHOMIMIC_DIR}")
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
    source: UploadFile = File(..., description="source portrait image (jpg/png)"),
    audio: UploadFile = File(..., description="driving audio (wav/mp3)"),
    fps: int = Form(25),
    seed: int = Form(42),
) -> FileResponse:
    script = _find_inference_script()
    if script is None:
        raise HTTPException(status_code=503, detail="no EchoMimicV3 inference script found")

    job_id = uuid.uuid4().hex[:12]
    work = Path(tempfile.mkdtemp(prefix=f"em-{job_id}-"))
    try:
        src_path = work / f"source.{_ext_for(source, 'jpg')}"
        aud_path = work / f"audio.{_ext_for(audio, 'wav')}"
        out_dir = work / "out"
        out_dir.mkdir()

        await _save_upload(source, src_path)
        await _save_upload(audio, aud_path)

        # Upstream flag names are TBD until we pin a SHA; pass both common forms.
        cmd = [
            "python", str(script),
            "--source", str(src_path),
            "--audio", str(aud_path),
            "--output", str(out_dir),
            "--fps", str(fps),
            "--seed", str(seed),
        ]

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

        candidates = sorted(out_dir.glob("*.mp4"))
        if not candidates:
            raise HTTPException(
                status_code=500,
                detail={"error": "no output mp4 produced", "log_tail": log_tail},
            )
        primary = candidates[0]
        staged = Path(tempfile.gettempdir()) / f"em-out-{job_id}.mp4"
        shutil.copy2(primary, staged)

        LOG.info("job=%s done elapsed=%.1fs out=%s size=%d", job_id, elapsed, staged.name, staged.stat().st_size)
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
