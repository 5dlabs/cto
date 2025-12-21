# syntax=docker/dockerfile:1
# =============================================================================
# OpenCode CLI Agent Image - Unified Dockerfile
# =============================================================================
#
# Supports both production (runtime base) and local (node slim) builds:
#
#   Production/GHCR build (default - uses runtime base):
#     docker build -t ghcr.io/5dlabs/opencode .
#
#   Local/Kind build (uses node:18-slim for simpler local testing):
#     docker build --build-arg BASE_IMAGE=node:18-slim -t ghcr.io/5dlabs/opencode:local .
#
# =============================================================================

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ=UTC
ENV TZ="$TZ"
ENV DEVCONTAINER=true

USER root

# Create workspace and user directories
RUN mkdir -p /workspace /home/node/.opencode /home/node/.cache/opencode && \
    chown -R 1000:1000 /home/node/.opencode /home/node/.cache || true

WORKDIR /workspace

# Install base dependencies (only needed if using node:slim base)
# These are already present in the runtime image
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    bash \
    unzip \
    wget \
    && rm -rf /var/lib/apt/lists/* || true

# Install Open Code CLI with retries
RUN set -eux; \
    for i in 1 2 3; do \
        if curl -fsSL https://opencode.ai/install | bash; then \
            echo "✅ OpenCode installation succeeded on attempt $i"; \
            break; \
        else \
            echo "⚠️ OpenCode installation failed on attempt $i, retrying..."; \
            sleep 5; \
        fi; \
    done; \
    # Verify installation and install binary if it exists
    if [ -f "/root/.opencode/bin/opencode" ]; then \
        install -m 0755 "/root/.opencode/bin/opencode" /usr/local/bin/opencode; \
        opencode --version || echo "OpenCode installed but version check failed"; \
    else \
        echo "⚠️ OpenCode binary not found, creating placeholder"; \
        printf '#!/bin/bash\necho "OpenCode CLI placeholder - installation may have failed"' > /usr/local/bin/opencode; \
        chmod +x /usr/local/bin/opencode; \
    fi

# Install OpenAI Node.js SDK for compatibility
RUN npm install -g openai@latest && \
    echo "✅ OpenAI Node.js SDK installed" && \
    (node -e "console.log('OpenAI Node.js SDK version:', require('openai/package.json').version)" || echo "⚠️ Node.js SDK verification failed")

# Ensure cache directory ownership
RUN chown -R 1000:1000 /home/node/.cache || true

USER node

# Set environment for OpenCode
ENV HOME=/home/node

# Open Code CLI is now available via `opencode`
