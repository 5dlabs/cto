"""Download MuseTalk code and model artifacts into the local model cache.

Upstream uses two sources:
- GitHub repo for source code
- Hugging Face repos for model artifacts

The MuseTalk 1.5 runtime specifically requires these files to exist:
- /models/musetalkV15/musetalk.json
- /models/musetalkV15/unet.pth
"""

import logging
import os
import shutil
import subprocess
from pathlib import Path

from huggingface_hub import hf_hub_download, snapshot_download

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger("download_models")


MUSE_GIT_REPO = "https://github.com/TMElyralab/MuseTalk.git"
MUSE_HF_REPO = "TMElyralab/MuseTalk"

# repo_id, destination relative to cache_dir, include patterns
REQUIRED_HF_REPOS = [
    (MUSE_HF_REPO, "", ["musetalk/musetalk.json", "musetalk/pytorch_model.bin"]),
    (MUSE_HF_REPO, "", ["musetalkV15/musetalk.json", "musetalkV15/unet.pth"]),
    ("stabilityai/sd-vae-ft-mse", "sd-vae", ["config.json", "diffusion_pytorch_model.bin"]),
    ("openai/whisper-tiny", "whisper", ["config.json", "pytorch_model.bin", "preprocessor_config.json"]),
    ("yzd-v/DWPose", "dwpose", ["dw-ll_ucoco_384.pth"]),
    ("ByteDance/LatentSync", "syncnet", ["latentsync_syncnet.pt"]),
]

REQUIRED_DIRECT_FILES = [
    (
        "https://download.pytorch.org/models/resnet18-5c106cde.pth",
        "face-parse-bisent/resnet18-5c106cde.pth",
    ),
]


def clone_git_repo(repo: str, dest: Path) -> None:
    if dest.exists() and any(dest.iterdir()):
        log.info("Git repo already present at %s, skipping clone", dest)
        return
    if dest.exists() or dest.is_symlink():
        if dest.is_symlink() or dest.is_file():
            dest.unlink()
        else:
            shutil.rmtree(dest)
    dest.parent.mkdir(parents=True, exist_ok=True)
    log.info("Cloning git repo %s -> %s", repo, dest)
    subprocess.run(["git", "clone", "--depth", "1", repo, str(dest)], check=True)


def download_hf_patterns(repo_id: str, dest: Path, token: str | None, allow_patterns: list[str]) -> None:
    log.info("Downloading HF repo %s patterns %s -> %s", repo_id, allow_patterns, dest)
    dest.mkdir(parents=True, exist_ok=True)
    snapshot_download(
        repo_id=repo_id,
        local_dir=str(dest),
        token=token,
        allow_patterns=allow_patterns,
        ignore_patterns=["*.md", "*.txt", "LICENSE*", ".gitattributes"],
    )


def ensure_hf_artifacts(cache_dir: Path, token: str | None) -> None:
    for repo_id, relative_dest, patterns in REQUIRED_HF_REPOS:
        dest = cache_dir / relative_dest if relative_dest else cache_dir
        missing = [pattern for pattern in patterns if not (dest / pattern).exists()]
        if not missing:
            log.info("HF artifacts already present for %s at %s", repo_id, dest)
            continue
        log.info("Missing artifacts for %s: %s", repo_id, missing)
        download_hf_patterns(repo_id, dest, token, patterns)


def ensure_direct_file(cache_dir: Path, url: str, relative_path: str) -> None:
    dest = cache_dir / relative_path
    if dest.exists():
        log.info("Direct file already present at %s", dest)
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    log.info("Downloading direct file %s -> %s", url, dest)
    subprocess.run(["curl", "-L", url, "-o", str(dest)], check=True)


def ensure_layout(cache_dir: Path) -> None:
    required_dirs = [
        "musetalk",
        "musetalkV15",
        "sd-vae",
        "whisper",
        "dwpose",
        "syncnet",
        "face-parse-bisent",
    ]
    for relative in required_dirs:
        (cache_dir / relative).mkdir(parents=True, exist_ok=True)

    critical_files = [
        cache_dir / "musetalkV15" / "musetalk.json",
        cache_dir / "musetalkV15" / "unet.pth",
    ]
    for path in critical_files:
        if not path.exists():
            raise FileNotFoundError(f"Critical MuseTalk artifact missing after bootstrap: {path}")


def main():
    repo = os.environ.get("MUSE_CODE_REPO", MUSE_GIT_REPO)
    cache_dir = Path(os.environ.get("MODEL_CACHE_DIR", "/models"))
    token = os.environ.get("HF_TOKEN")
    source_target = cache_dir / "musetalk-src"

    clone_git_repo(repo, source_target)
    ensure_hf_artifacts(cache_dir, token)

    for url, relative_path in REQUIRED_DIRECT_FILES:
        ensure_direct_file(cache_dir, url, relative_path)

    ensure_layout(cache_dir)
    log.info("Model download complete")


if __name__ == "__main__":
    main()
