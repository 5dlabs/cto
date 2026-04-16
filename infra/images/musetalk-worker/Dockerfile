# MuseTalk Avatar Rendering Worker
# Base: NVIDIA PyTorch container with CUDA 11.8 (V100-compatible)
# Target: V100S 32GB via PCI passthrough on OVH Public Cloud

FROM nvcr.io/nvidia/pytorch:23.10-py3 AS base

WORKDIR /app

# System dependencies for video processing
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    && rm -rf /var/lib/apt/lists/*

# Python dependencies — pinned for V100 compatibility
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Application code
COPY download_models.py .
COPY worker.py .
COPY render.py .

# Health check server port
EXPOSE 8080
# Metrics port
EXPOSE 9090

# Default: run the NATS worker
CMD ["python3", "/app/worker.py"]
