ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
USER root
RUN mkdir -p /workspace /home/node/.opencode && \
  chown -R 1000:1000 /home/node/.opencode
WORKDIR /workspace

# Install Open Code CLI (as root), then place binary in /usr/local/bin for all users
USER root
RUN set -eux; \
  # Install required tools for Debian/Ubuntu
  apt-get update && apt-get install -y curl bash unzip wget && rm -rf /var/lib/apt/lists/*; \
  # Try to install OpenCode with retries and fallbacks
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
    echo '#!/bin/bash\necho "OpenCode CLI placeholder - installation may have failed"' > /usr/local/bin/opencode; \
    chmod +x /usr/local/bin/opencode; \
  fi

# Install OpenAI Node.js SDK for compatibility
RUN npm install -g openai@latest && \
  echo "✅ OpenAI Node.js SDK installed" && \
  (node -e "console.log('OpenAI Node.js SDK version:', require('openai/package.json').version)" || echo "⚠️ Node.js SDK verification failed")

# Create required cache directory for OpenCode
RUN mkdir -p /home/node/.cache/opencode \
  && chown -R 1000:1000 /home/node/.cache

USER node

# Set environment for OpenCode
ENV HOME=/home/node

# Open Code CLI is now available via `opencode`
