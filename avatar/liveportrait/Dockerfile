# LivePortrait FastAPI container for OVH AI Deploy
# Base: CUDA 11.8 (V100S is Volta; CUDA 11.8 is the sweet spot — torch 2.3.0 wheels exist)
# Target flavor: ai1-1-gpu on V100S, region GRA

FROM nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    HF_HOME=/workspace/.cache/huggingface \
    TORCH_HOME=/workspace/.cache/torch \
    # OVH AI Deploy runs containers as arbitrary UID; make /workspace world-writable
    HOME=/workspace

WORKDIR /workspace

# System deps: python 3.10, ffmpeg, git, build tools for native wheels
# cuda-nvrtc-11-8 is required by cuDNN kernels at runtime (missing from base runtime image)
RUN apt-get update && apt-get install -y --no-install-recommends \
      python3.10 python3.10-venv python3.10-dev python3-pip \
      git git-lfs ffmpeg \
      build-essential cmake pkg-config \
      libgl1 libglib2.0-0 libsm6 libxext6 libxrender1 \
      curl ca-certificates \
      cuda-nvrtc-11-8 \
 && rm -rf /var/lib/apt/lists/* \
 && git lfs install \
 && ln -sf /usr/bin/python3.10 /usr/local/bin/python \
 && ln -sf /usr/bin/python3.10 /usr/local/bin/python3 \
 && python -m pip install --upgrade pip setuptools wheel

# Clone LivePortrait at a pinned commit
ARG LIVEPORTRAIT_SHA=49784e879821538ecda5c8e4ca0472f4cb6236cf
RUN git clone https://github.com/KlingAIResearch/LivePortrait.git /workspace/LivePortrait \
 && cd /workspace/LivePortrait \
 && git checkout ${LIVEPORTRAIT_SHA}

WORKDIR /workspace/LivePortrait

# Install torch first (CUDA 11.8 build), then LivePortrait requirements
RUN pip install --index-url https://download.pytorch.org/whl/cu118 \
      torch==2.3.0 torchvision==0.18.0 torchaudio==2.3.0 \
 && pip install -r requirements.txt \
 && pip install "huggingface_hub[cli]==0.24.0" \
      fastapi==0.115.0 \
      "uvicorn[standard]==0.30.6" \
      python-multipart==0.0.9

# Download pretrained weights (humans only; skip animals to save ~2GB)
# Using HF_HUB_ENABLE_HF_TRANSFER=1 for faster parallel download when available
RUN HF_HUB_ENABLE_HF_TRANSFER=0 huggingface-cli download \
      KwaiVGI/LivePortrait \
      --local-dir /workspace/LivePortrait/pretrained_weights \
      --exclude "*.git*" "README.md" "docs/*" "liveportrait_animals/*"

# FastAPI wrapper
COPY server.py /workspace/server.py

# OVH AI Deploy expects app on configurable port; default 8000
ENV PORT=8000 \
    LIVEPORTRAIT_DIR=/workspace/LivePortrait

EXPOSE 8000

# Make everything world-writable so OVH's random UID can execute + write tmp files
RUN chmod -R 777 /workspace

HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
  CMD curl -fsS http://127.0.0.1:${PORT}/health || exit 1

CMD ["sh", "-c", "uvicorn server:app --host 0.0.0.0 --port ${PORT} --workers 1"]
