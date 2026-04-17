"""Download MuseTalk model weights into the local model cache.

The upstream repo is a GitHub repository, not a HuggingFace model repo.
If HF_MODEL_REPO points at a HuggingFace repo, use snapshot_download.
Otherwise clone the git repo directly into the cache.
"""

import logging
import os
import shutil
import subprocess
from pathlib import Path

from huggingface_hub import snapshot_download

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger("download_models")


def clone_git_repo(repo: str, dest: Path) -> None:
    if dest.exists() and any(dest.iterdir()):
        log.info("Git repo already present at %s, skipping clone", dest)
        return
    if dest.exists():
        shutil.rmtree(dest)
    dest.parent.mkdir(parents=True, exist_ok=True)
    log.info("Cloning git repo %s -> %s", repo, dest)
    subprocess.run([
        "git", "clone", "--depth", "1", repo, str(dest)
    ], check=True)


def download_hf_repo(repo: str, dest: Path, token: str | None) -> None:
    log.info("Downloading HuggingFace repo %s -> %s", repo, dest)
    snapshot_download(
        repo_id=repo,
        local_dir=str(dest),
        token=token,
        ignore_patterns=["*.md", "*.txt", "LICENSE*", ".gitattributes"],
    )


def main():
    repo = os.environ.get("HF_MODEL_REPO", "https://github.com/TMElyralab/MuseTalk.git")
    cache_dir = Path(os.environ.get("MODEL_CACHE_DIR", "/models"))
    token = os.environ.get("HF_TOKEN")
    target = cache_dir / "musetalk"

    if repo.startswith("http://") or repo.startswith("https://") or repo.endswith(".git"):
        clone_git_repo(repo, target)
    else:
        download_hf_repo(repo, target, token)

    log.info("Model download complete")


if __name__ == "__main__":
    main()
