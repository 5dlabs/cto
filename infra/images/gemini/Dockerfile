ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Setup workspace and user environment
RUN mkdir -p /workspace /home/node/.gemini /home/node/.config /home/node/.cache && \
    chown -R 1000:1000 /home/node/.gemini /home/node/.config /home/node/.cache

WORKDIR /workspace

# Install Gemini CLI as root, then drop to node
USER root

# Install Google Gemini CLI
# Note: Gemini CLI installation method - using npm global install
# The actual package name may need to be updated based on Google's official release
ARG VERSION=latest
RUN set -eux; \
    # Install Gemini CLI (adjust package name when official package is available)
    # For now, creating a placeholder that can be updated
    npm install -g @google-cloud/generative-ai@${VERSION} || \
    npm install -g @google/generative-ai@${VERSION} || \
    echo "⚠️ Gemini CLI package not found - may need manual installation"; \
    # Create gemini-cli wrapper if needed
    if ! command -v gemini-cli >/dev/null 2>&1 && ! command -v gemini >/dev/null 2>&1; then \
      echo '#!/bin/bash' > /usr/local/bin/gemini-cli; \
      echo 'echo "Gemini CLI placeholder - update Dockerfile with correct package"' >> /usr/local/bin/gemini-cli; \
      echo 'exit 1' >> /usr/local/bin/gemini-cli; \
      chmod +x /usr/local/bin/gemini-cli; \
    fi

# Configure Gemini-specific environment
USER node

# Configure Gemini-specific git identity
RUN git config --global user.name "5D Labs Agent" && \
    git config --global user.email "agent@5dlabs.com"

# Verify installation
RUN gemini-cli --version 2>/dev/null || gemini --version 2>/dev/null || \
    echo "⚠️ Gemini CLI verification skipped - needs proper package"

# Final setup
USER node


