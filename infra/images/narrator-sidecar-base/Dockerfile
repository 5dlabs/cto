FROM python:3.11-slim

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    # TTS model cache
    TTS_HOME=/models/tts \
    COQUI_TOS_AGREED=1

WORKDIR /app

# System deps: ffmpeg for audio, libsrtp2 for WebRTC DTLS, libopus for WebRTC audio codec
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    libsrtp2-dev \
    libopus-dev \
    libvpx-dev \
    pkg-config \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt /tmp/requirements.txt
RUN pip install --upgrade pip && \
    pip install -r /tmp/requirements.txt

COPY app/ ./app/

RUN mkdir -p /workspace/.narrator /models/tts

EXPOSE 8081 8082

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8081"]
