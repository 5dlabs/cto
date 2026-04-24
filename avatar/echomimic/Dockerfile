# EchoMimicV3 FastAPI container for OVH AI Deploy
# Base: CUDA 12.1 (EchoMimicV3 upstream tested with torch 2.3+cu121)
# Target flavor: ai1-1-gpu on V100S, region GRA
#
# Notes:
#   * V100S supports sm_70. Torch 2.3 cu121 wheels include sm_70 binaries.
#   * EchoMimicV3 (BadToBest) does audio-driven portrait animation end-to-end,
#     so the input surface is: source image + audio → mp4 (vs LivePortrait's
#     source + driving video).

FROM nvidia/cuda:12.1.1-cudnn8-runtime-ubuntu22.04

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    HF_HOME=/workspace/.cache/huggingface \
    TORCH_HOME=/workspace/.cache/torch \
    HOME=/workspace

WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
      python3.10 python3.10-venv python3.10-dev python3-pip \
      git git-lfs ffmpeg \
      build-essential cmake pkg-config \
      libgl1 libglib2.0-0 libsm6 libxext6 libxrender1 \
      curl ca-certificates \
      cuda-nvrtc-12-1 \
 && rm -rf /var/lib/apt/lists/* \
 && git lfs install \
 && ln -sf /usr/bin/python3.10 /usr/local/bin/python \
 && ln -sf /usr/bin/python3.10 /usr/local/bin/python3 \
 && python -m pip install --upgrade pip setuptools wheel

# Clone EchoMimicV3 at a pinned commit (update SHA after first successful build)
ARG ECHOMIMIC_REPO=https://github.com/antgroup/echomimic_v3.git
# Pinned 2026-04 head; update after confirming infer_flash.py API is stable.
ARG ECHOMIMIC_SHA=7e89489ca51c0d008fc1963ec6c03fc5bd0b9397
RUN git clone ${ECHOMIMIC_REPO} /workspace/EchoMimicV3 \
 && cd /workspace/EchoMimicV3 \
 && git checkout ${ECHOMIMIC_SHA}

WORKDIR /workspace/EchoMimicV3

# Torch cu121 first, then upstream requirements
RUN pip install --index-url https://download.pytorch.org/whl/cu121 \
      torch==2.3.1 torchvision==0.18.1 torchaudio==2.3.1 \
 && if [ -f requirements.txt ]; then pip install -r requirements.txt; fi \
 && pip install "huggingface_hub[cli]==0.27.1" \
      fastapi==0.115.0 \
      "uvicorn[standard]==0.30.6" \
      python-multipart==0.0.9 \
      soundfile==0.12.1

# EchoMimicV3-Flash needs 3 HF repos (see upstream README):
#   1. BadToBest/EchoMimicV3          — flash pro weights + transformer
#   2. alibaba-pai/Wan2.1-Fun-V1.1-1.3B-InP — base video diffusion model
#   3. TencentGameMate/chinese-wav2vec2-base — audio encoder
# Combined ~10GB; doesn't fit on standard GHA runners. We download at
# container start (bootstrap.sh) instead — OVH AI Deploy has the disk.
ENV PRETRAINED_DIR=/workspace/EchoMimicV3/pretrained_weights
RUN mkdir -p ${PRETRAINED_DIR}

COPY server.py /workspace/server.py
COPY bootstrap.sh /workspace/bootstrap.sh
RUN chmod +x /workspace/bootstrap.sh

ENV PORT=8000 \
    ECHOMIMIC_DIR=/workspace/EchoMimicV3 \
    ECHOMIMIC_PRETRAINED_DIR=/workspace/EchoMimicV3/pretrained_weights

EXPOSE 8000

RUN chmod -R 777 /workspace

HEALTHCHECK --interval=30s --timeout=10s --start-period=600s --retries=3 \
  CMD curl -fsS http://127.0.0.1:${PORT}/health || exit 1

CMD ["/workspace/bootstrap.sh"]
