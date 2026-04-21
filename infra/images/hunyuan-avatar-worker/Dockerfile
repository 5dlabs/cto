# Hunyuan Avatar Worker - Optimized for CI builds
# Uses pytorch base with CUDA 12.1, pre-installed deps
FROM pytorch/pytorch:2.4.0-cuda12.1-cudnn9-runtime

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

# Install system deps
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

# Install Python deps
COPY requirements.txt /tmp/requirements.txt
RUN pip install --upgrade pip && \
    pip install -r /tmp/requirements.txt && \
    pip install aiohttp==3.10.10 prometheus-client==0.21.0 nats-py boto3

# flash_attn prebuilt wheel for torch 2.4 + cu12.1 + cp311.
# hymm_sp imports flash_attn unconditionally at module load; pure-diffusers
# fallback is not viable. Wheel is forward-compatible cu123->cu121 runtime.
# Verified empirically in pod 2026-04-21.
RUN pip install --no-deps \
    https://github.com/Dao-AILab/flash-attention/releases/download/v2.6.3/flash_attn-2.6.3+cu123torch2.4cxx11abiFALSE-cp311-cp311-linux_x86_64.whl

# Copy application code
COPY download_models.py render.py worker.py ./

RUN mkdir -p /models/hunyuan-avatar /models/.cache/huggingface /tmp/renders/hunyuan-avatar

EXPOSE 8080 9090

CMD ["python", "/app/worker.py"]
