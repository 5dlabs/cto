FROM python:3.11-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    ffmpeg \
    libgl1 \
    libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY avatar/agent/pyproject.toml avatar/agent/README.md ./
COPY avatar/agent/morgan_avatar_agent ./morgan_avatar_agent
COPY avatar/agent/agent.py ./
COPY avatar/morgan.jpg ./assets/morgan.jpg

RUN pip install --no-cache-dir --upgrade pip \
    && pip install --no-cache-dir '.[legacy,musetalk]'

CMD ["python", "agent.py", "start"]
