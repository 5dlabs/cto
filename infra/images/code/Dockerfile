ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

USER root

# Install Every Code CLI (fork of Codex with multi-agent support)
# https://github.com/just-every/code
ARG VERSION=latest
RUN npm install -g @just-every/code@${VERSION} \
  && echo "✅ Every Code CLI installed" \
  && (code --version || echo "⚠️ Every Code CLI verification failed")

# Install OpenAI Node.js SDK globally for compatibility (Every Code supports OpenAI provider)
RUN npm install -g openai@latest \
  && echo "✅ OpenAI Node.js SDK installed" \
  && (node -e "console.log('OpenAI Node.js SDK version:', require('openai/package.json').version)" || echo "⚠️ Node.js SDK verification failed")

# Create Every Code config directory
# Every Code reads from ~/.code/ (primary) and ~/.codex/ (legacy)
RUN mkdir -p /home/node/.code/log /home/node/.code/cache \
  && mkdir -p /home/node/.codex \
  && chown -R node:node /home/node/.code /home/node/.codex

# Create workspace directory
RUN mkdir -p /workspace && chown -R node:node /workspace

# Set up environment variables for Every Code
ENV HOME=/home/node
ENV CODE_HOME=/home/node/.code

# Configure git identity as root (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

# Drop privileges back to non-root user provided by base image
USER node
WORKDIR /workspace

# Verify Every Code CLI installation
RUN code --version && echo "✅ Every Code CLI ready"

# Default entrypoint (can be overridden)
CMD ["code", "--help"]
