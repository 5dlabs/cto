#!/usr/bin/env bash
# Runtime weight bootstrap for EchoMimicV3-Flash.
#
# The three required HF repos (~10GB) don't fit on a standard GHA runner's
# build disk, so we download them on container start instead. OVH AI Deploy
# instances have ample ephemeral storage and persistent caching per job, so
# first-boot cost (~3-5 min on a warm HF CDN) is paid once per cold start.
#
# Idempotent: each repo is skipped if its sentinel already exists.

set -euo pipefail

# HF Hub: default 10s read timeout is too aggressive for the ~10GB of weights
# on the xethub CDN. Bump to 5 min; resume is automatic via huggingface-cli.
export HF_HUB_DOWNLOAD_TIMEOUT="${HF_HUB_DOWNLOAD_TIMEOUT:-300}"
export HF_XET_DOWNLOAD_TIMEOUT="${HF_XET_DOWNLOAD_TIMEOUT:-300}"

PRETRAINED_DIR="${ECHOMIMIC_PRETRAINED_DIR:-/workspace/EchoMimicV3/pretrained_weights}"
mkdir -p "$PRETRAINED_DIR"

download() {
  local repo="$1"
  local dest="$2"
  local sentinel="$3"
  if [ -e "$dest/$sentinel" ]; then
    echo "[bootstrap] $repo already present at $dest"
    return 0
  fi
  echo "[bootstrap] downloading $repo -> $dest"
  local attempt=1
  local max_attempts=5
  while [ $attempt -le $max_attempts ]; do
    if HF_HUB_ENABLE_HF_TRANSFER=0 huggingface-cli download \
        "$repo" \
        --local-dir "$dest" \
        --exclude "*.git*" "README.md" "docs/*"; then
      echo "[bootstrap] $repo done"
      return 0
    fi
    echo "[bootstrap][warn] $repo attempt $attempt/$max_attempts failed; retrying in 10s"
    sleep 10
    attempt=$((attempt+1))
  done
  echo "[bootstrap][error] $repo download failed after $max_attempts attempts"
  return 1
}

# EchoMimicV3 flash transformer (~3.4GB)
download "BadToBest/EchoMimicV3" \
         "$PRETRAINED_DIR/EchoMimicV3" \
         "transformer/diffusion_pytorch_model.safetensors" || true

# Wan2.1-Fun-V1.1-1.3B-InP base video diffusion (~3GB)
download "alibaba-pai/Wan2.1-Fun-V1.1-1.3B-InP" \
         "$PRETRAINED_DIR/Wan2.1-Fun-V1.1-1.3B-InP" \
         "config.json" || true

# chinese-wav2vec2-base audio encoder (~400MB)
download "TencentGameMate/chinese-wav2vec2-base" \
         "$PRETRAINED_DIR/chinese-wav2vec2-base" \
         "config.json" || true

echo "[bootstrap] layout:"
ls -la "$PRETRAINED_DIR" || true

exec uvicorn server:app --app-dir /workspace --host 0.0.0.0 --port "${PORT:-8000}" --workers 1
