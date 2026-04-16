"""Download MuseTalk model weights from HuggingFace to local cache."""

import os
import logging
from huggingface_hub import snapshot_download

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger("download_models")


def main():
    repo = os.environ.get("HF_MODEL_REPO", "TMElyralab/MuseTalk")
    cache_dir = os.environ.get("MODEL_CACHE_DIR", "/models")
    token = os.environ.get("HF_TOKEN")

    log.info("Downloading %s → %s", repo, cache_dir)

    snapshot_download(
        repo_id=repo,
        local_dir=os.path.join(cache_dir, "musetalk"),
        token=token,
        ignore_patterns=["*.md", "*.txt", "LICENSE*", ".gitattributes"],
    )

    log.info("Model download complete")


if __name__ == "__main__":
    main()
