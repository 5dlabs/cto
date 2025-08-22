ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenAI CLI and Python SDK into a dedicated virtualenv
ARG VERSION=latest
RUN python3 -m venv /opt/openai/.venv \
  && . /opt/openai/.venv/bin/activate \
  && pip install --no-cache-dir --upgrade pip \
  && if [ "$VERSION" = "latest" ]; then \
       pip install --no-cache-dir openai openai-cli requests aiohttp; \
     else \
       pip install --no-cache-dir "openai==${VERSION}" openai-cli requests aiohttp; \
     fi

# Install OpenAI Node.js SDK globally
RUN npm install -g openai@latest

# Make OpenAI venv available on PATH for all users
ENV VIRTUAL_ENV=/opt/openai/.venv
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

# Install Codex-specific tools from the official repository
RUN set -eux; \
  # Clone the official Codex repository for examples and tools
  git clone --depth=1 https://github.com/openai/codex.git /tmp/codex; \
  # Install any additional requirements if they exist
  if [ -f /tmp/codex/requirements.txt ]; then \
    . /opt/openai/.venv/bin/activate && pip install -r /tmp/codex/requirements.txt; \
  fi; \
  # Copy useful scripts to system path if they exist
  if [ -d /tmp/codex/scripts ]; then \
    cp -r /tmp/codex/scripts/* /usr/local/bin/ 2>/dev/null || true; \
  fi; \
  # Clean up
  rm -rf /tmp/codex

# Create workspace and config directories and set permissions
RUN mkdir -p /workspace /home/node/.openai \
  && chown -R 1000:1000 /home/node/.openai

# Set up environment variables for OpenAI API
ENV OPENAI_API_BASE=https://api.openai.com/v1
ENV OPENAI_MODEL=gpt-4

# Drop privileges back to non-root user provided by base image
USER node
WORKDIR /workspace

# Verify OpenAI CLI installation (non-fatal)
RUN openai --version >/dev/null 2>&1 || true \
  && python -c "import openai; print('OpenAI SDK available')" >/dev/null 2>&1 || true

# Default entrypoint (can be overridden)
CMD ["openai", "--help"]