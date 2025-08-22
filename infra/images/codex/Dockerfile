ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
USER root
RUN mkdir -p /workspace /home/node/.codex && \
  chown -R 1000:1000 /home/node/.codex
WORKDIR /workspace

# Install Codex-specific tools from the official repository
RUN set -eux; \
  # Clone the official Codex repository for examples and tools
  git clone --depth=1 https://github.com/openai/codex.git /tmp/codex; \
  # Install any additional requirements if they exist
  if [ -f /tmp/codex/requirements.txt ]; then \
    pip install -r /tmp/codex/requirements.txt; \
  fi; \
  # Copy useful scripts to system path if they exist
  if [ -d /tmp/codex/scripts ]; then \
    cp -r /tmp/codex/scripts/* /usr/local/bin/ 2>/dev/null || true; \
  fi; \
  # Clean up
  rm -rf /tmp/codex

# Set up environment variables for OpenAI API
ENV OPENAI_API_BASE=https://api.openai.com/v1
ENV OPENAI_MODEL=gpt-4

USER node

# OpenAI CLI and Codex tools are now available from runtime base image