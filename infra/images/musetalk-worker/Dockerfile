# MuseTalk Avatar Rendering Worker
<<<<<<< HEAD
# Base: Ubuntu 22.04 with Python 3.11 and CUDA 11.8 (V100-compatible)
# Target: V100S 32GB via PCI passthrough on OVH Public Cloud

FROM ubuntu:22.04 AS base
=======
# Base: Ubuntu 22.04 with CUDA 11.8 runtime
# Target: V100S 32GB via PCI passthrough on OVH Public Cloud

FROM ubuntu:22.04
>>>>>>> e25fbb68 (fix(musetalk): use ubuntu:22.04 base with manual CUDA install for clean build)

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    MODEL_CACHE_DIR=/models \
    CUDA_VERSION=11.8.0

WORKDIR /app

<<<<<<< HEAD
# Install Python 3.11 and system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
    python3.11 \
    python3.11-dev \
    python3.11-venv \
    python3-pip \
    ffmpeg \
    git \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set Python 3.11 as default
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.11 1 \
    && update-alternatives --install /usr/bin/python python /usr/bin/python3.11 1

# Install CUDA 11.8
RUN wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb \
    && dpkg -i cuda-keyring_1.0-1_all.deb \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
    cuda-toolkit-11-8 \
    && rm -rf /var/lib/apt/lists/* cuda-keyring_1.0-1_all.deb

# Set CUDA environment
ENV PATH=/usr/local/cuda-11.8/bin:${PATH}
ENV LD_LIBRARY_PATH=/usr/local/cuda-11.8/lib64:${LD_LIBRARY_PATH}
ENV CUDA_HOME=/usr/local/cuda-11.8
# Trigger workflow build

# System dependencies for video processing and model bootstrap
=======
# Install Python 3.10 and system dependencies
>>>>>>> e25fbb68 (fix(musetalk): use ubuntu:22.04 base with manual CUDA install for clean build)
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-dev \
    python3-pip \
    python3-venv \
    git \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

<<<<<<< HEAD
# Install PyTorch with CUDA 11.8 support
RUN python3 -m pip install --upgrade pip setuptools wheel && \
    python3 -m pip install torch==2.1.0 torchvision==0.16.0 --index-url https://download.pytorch.org/whl/cu118

# Python dependencies — pinned for CUDA 11.8 / V100S bootstrap
COPY requirements.txt .
RUN python3 -m pip install -r requirements.txt
=======
# Download static ffmpeg build
RUN wget -q https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz \
    && tar -xf ffmpeg-master-latest-linux64-gpl.tar.xz \
    && mv ffmpeg-master-latest-linux64-gpl/bin/ffmpeg /usr/local/bin/ \
    && mv ffmpeg-master-latest-linux64-gpl/bin/ffprobe /usr/local/bin/ \
    && rm -rf ffmpeg-master-latest-linux64-gpl*

# Install CUDA 11.8 runtime libraries
RUN wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb \
    && dpkg -i cuda-keyring_1.0-1_all.deb \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
    cuda-cudart-11-8 \
    cuda-nvrtc-11-8 \
    libcudnn8 \
    && rm -rf /var/lib/apt/lists/* cuda-keyring_1.0-1_all.deb

# Set CUDA environment
ENV PATH=/usr/local/cuda-11.8/bin:${PATH}
ENV LD_LIBRARY_PATH=/usr/local/cuda-11.8/lib64:${LD_LIBRARY_PATH}
ENV CUDA_HOME=/usr/local/cuda-11.8

# Install PyTorch with CUDA 11.8
RUN python3 -m pip install --upgrade pip && \
    python3 -m pip install torch==2.1.0 torchvision==0.16.0 --index-url https://download.pytorch.org/whl/cu118

# Python dependencies - use older livekit-rtc for Python 3.10
COPY requirements.txt .
RUN sed -i 's/livekit-rtc==0.17.5/livekit-rtc==0.16.0/' requirements.txt && \
    python3 -m pip install -r requirements.txt
>>>>>>> e25fbb68 (fix(musetalk): use ubuntu:22.04 base with manual CUDA install for clean build)

# Application code
COPY download_models.py .
COPY worker.py .
COPY render.py .

RUN mkdir -p /models /tmp/renders

# Health check server port
EXPOSE 8080
# Metrics port
EXPOSE 9090

# Default: run the NATS worker
CMD ["python3", "/app/worker.py"]
