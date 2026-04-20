"""Download HunyuanVideo-Avatar weights into the local model cache."""

from __future__ import annotations

import logging
import os
from pathlib import Path

from huggingface_hub import snapshot_download

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger("download_models")

MODEL_REPO = "tencent/HunyuanVideo-Avatar"
MODEL_DIR = Path(os.environ.get("MODEL_CACHE_DIR", "/models/hunyuan-avatar"))
READY_SENTINEL = MODEL_DIR / ".ready"


def main() -> None:
    if READY_SENTINEL.exists():
        log.info("Sentinel exists at %s, skipping download", READY_SENTINEL)
        return

    MODEL_DIR.mkdir(parents=True, exist_ok=True)
    token = os.environ.get("HF_TOKEN")

    snapshot_download(
        repo_id=MODEL_REPO,
        local_dir=str(MODEL_DIR),
        local_dir_use_symlinks=False,
        token=token,
        resume_download=True,
    )

    READY_SENTINEL.write_text("ready\n", encoding="utf-8")
    log.info("Model download complete: %s", MODEL_DIR)


if __name__ == "__main__":
    main()
