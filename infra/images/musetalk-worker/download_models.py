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


# Required HuggingFace models for MuseTalk
REQUIRED_HF_MODELS = [
    ("stabilityai/sd-vae-ft-mse", "models/sd-vae"),  # VAE for face encoding
    ("facebook/wav2vec2-base-960h", "models/wav2vec"),  # Audio feature extraction
]


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


def download_required_models(cache_dir: Path, token: str | None) -> None:
    """Download required HuggingFace models for MuseTalk."""
    for repo_id, local_path in REQUIRED_HF_MODELS:
        dest = cache_dir / local_path
        if dest.exists() and any(dest.iterdir()):
            log.info("Model %s already present at %s, skipping", repo_id, dest)
            continue
        
        log.info("Downloading model %s -> %s", repo_id, dest)
        dest.parent.mkdir(parents=True, exist_ok=True)
        snapshot_download(
            repo_id=repo_id,
            local_dir=str(dest),
            token=token,
            ignore_patterns=["*.md", "*.txt", "LICENSE*", ".gitattributes"],
        )


def main():
    repo = os.environ.get("HF_MODEL_REPO", "https://github.com/TMElyralab/MuseTalk.git")
    cache_dir = Path(os.environ.get("MODEL_CACHE_DIR", "/models"))
    token = os.environ.get("HF_TOKEN")
    target = cache_dir / "musetalk"

    # Clone the MuseTalk repo
    if repo.startswith("http://") or repo.startswith("https://") or repo.endswith(".git"):
        clone_git_repo(repo, target)
    else:
        download_hf_repo(repo, target, token)

    # Create symlink for musetalkV15 compatibility (upstream expects this path)
    musetalk_v15 = cache_dir / "musetalkV15"
    if not musetalk_v15.exists():
        log.info("Creating musetalkV15 symlink for upstream compatibility")
        musetalk_v15.symlink_to(target, target_is_directory=True)

    # Download required HuggingFace models
    download_required_models(cache_dir, token)

    log.info("Model download complete")


if __name__ == "__main__":
    main()
