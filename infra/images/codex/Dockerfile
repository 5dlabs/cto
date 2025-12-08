ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenAI Codex CLI
ARG VERSION=latest
RUN npm install -g @openai/codex@${VERSION} \
  && echo "✅ OpenAI Codex CLI installed" \
  && (codex --version || echo "⚠️ Codex CLI verification failed")

# Install OpenAI Node.js SDK globally for compatibility
RUN npm install -g openai@latest \
  && echo "✅ OpenAI Node.js SDK installed" \
  && (node -e "console.log('OpenAI Node.js SDK version:', require('openai/package.json').version)" || echo "⚠️ Node.js SDK verification failed")

# Create Codex config directory
RUN mkdir -p /home/node/.codex/log /home/node/.codex/cache \
  && chown -R 1000:1000 /home/node/.codex

# Create workspace directory
RUN mkdir -p /workspace && chown -R 1000:1000 /workspace

# Set up environment variables for Codex
ENV HOME=/home/node

# Drop privileges back to non-root user provided by base image
USER node
WORKDIR /workspace

# Verify Codex CLI installation
RUN codex --version && echo "✅ Codex CLI ready"

# Default entrypoint (can be overridden)
CMD ["codex", "--help"]