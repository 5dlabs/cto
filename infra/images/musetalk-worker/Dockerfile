# MuseTalk Avatar Rendering Worker
# Base: Ubuntu 22.04 with Python 3.11 and CUDA 11.8 (V100-compatible)
# Target: V100S 32GB via PCI passthrough on OVH Public Cloud

ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}ubuntu:22.04 AS base

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    MODEL_CACHE_DIR=/models \
    CUDA_VERSION=11.8.0

WORKDIR /app

# Fix dpkg statoverride issue in Ubuntu 22.04 containers
RUN rm -f /var/lib/dpkg/info/*.postinst && \
    rm -f /var/lib/dpkg/statoverride && \
    touch /var/lib/dpkg/statoverride

# Install Python 3.11 and system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    software-properties-common \
    gnupg \
    ca-certificates \
    && add-apt-repository -y ppa:deadsnakes/ppa \
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
    cuda-libraries-11-8 \
    && rm -rf /var/lib/apt/lists/* cuda-keyring_1.0-1_all.deb

# Set CUDA environment
ENV PATH=/usr/local/cuda-11.8/bin:${PATH}
ENV LD_LIBRARY_PATH=/usr/local/cuda-11.8/lib64:${LD_LIBRARY_PATH}
ENV CUDA_HOME=/usr/local/cuda-11.8

# Install PyTorch with CUDA 11.8 support
RUN python3 -m pip install --upgrade pip setuptools wheel && \
    python3 -m pip install torch==2.1.0 torchvision==0.16.0 --index-url https://download.pytorch.org/whl/cu118

# Python dependencies — pinned for CUDA 11.8 / V100S bootstrap
COPY requirements.txt .
# chumpy (transitive dep, likely from smplx/mmpose) has a broken setup.py that
# imports `pip` directly, which fails inside PEP 517 build isolation. Pin a
# legacy setuptools toolchain and pre-install with --no-build-isolation so the
# main requirements install sees it as already satisfied.
RUN python3 -m pip install --upgrade "setuptools<70" "wheel<0.43" "pip<24" && \
    python3 -m pip install --no-build-isolation chumpy==0.70
RUN python3 -m pip install -r requirements.txt

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
