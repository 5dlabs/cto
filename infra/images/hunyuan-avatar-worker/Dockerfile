FROM nvcr.io/nvidia/pytorch:24.10-py3 AS builder

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    CUDA_VISIBLE_DEVICES=0 \
    MODEL_CACHE_DIR=/models/hunyuan-avatar \
    HF_HOME=/models/.cache/huggingface \
    TORCH_HOME=/models/.cache/torch

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    git \
    curl \
    ca-certificates \
    build-essential \
    ninja-build \
    pkg-config \
    libgl1 \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender1 \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt /tmp/requirements.txt

RUN python -m pip install --upgrade pip setuptools wheel && \
    python -m pip install torch==2.4.0 torchvision==0.19.0 torchaudio==2.4.0 && \
    python -m pip install -r /tmp/requirements.txt && \
    python -m pip install flash-attn==2.6.3 --no-build-isolation && \
    python -m pip install aiohttp==3.10.10 prometheus-client==0.21.0

FROM nvcr.io/nvidia/pytorch:24.10-py3 AS runtime

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    CUDA_VISIBLE_DEVICES=0 \
    MODEL_CACHE_DIR=/models/hunyuan-avatar \
    HF_HOME=/models/.cache/huggingface \
    TORCH_HOME=/models/.cache/torch \
    DISABLE_SP=1 \
    USE_FP8=1 \
    USE_DEEPCACHE=1

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    git \
    curl \
    ca-certificates \
    libgl1 \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/lib/python3.10/dist-packages /usr/local/lib/python3.10/dist-packages
COPY --from=builder /usr/local/bin /usr/local/bin

COPY download_models.py render.py worker.py ./

RUN mkdir -p /models/hunyuan-avatar /models/.cache/huggingface /tmp/renders/hunyuan-avatar

EXPOSE 8080 9090

CMD ["python", "/app/worker.py"]
