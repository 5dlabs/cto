# syntax=docker/dockerfile:1.7-labs
## Dexter Agent - Autonomous Financial Research Agent
## https://github.com/virattt/dexter
##
## Dexter performs deep financial research using task planning,
## self-reflection, and real-time market data via LangChain.

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Setup workspace and user environment
RUN mkdir -p /workspace /home/node/.cache/pip /home/node/.dexter && \
    chown -R 1000:1000 /home/node/.cache /home/node/.dexter

WORKDIR /workspace

# Install uv (fast Python package manager from Astral)
USER root
RUN --mount=type=cache,target=/root/.cache \
    curl -LsSf https://astral.sh/uv/install.sh | sh && \
    # Make uv immediately available in PATH for this and subsequent layers
    ln -sf /root/.local/bin/uv /usr/local/bin/uv
ENV PATH="/root/.local/bin:$PATH"

# Clone Dexter repository
ARG DEXTER_VERSION=latest
RUN git clone https://github.com/virattt/dexter.git /opt/dexter && \
    if [ "$DEXTER_VERSION" != "latest" ]; then \
      cd /opt/dexter && git checkout "v${DEXTER_VERSION}"; \
    fi

# Install Dexter using uv (run from within the project directory)
WORKDIR /opt/dexter
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync && \
    ln -sf /opt/dexter/.venv/bin/dexter-agent /usr/local/bin/dexter-agent

# Verify installation
RUN uv run python -c "from dexter.agent import Agent; print('✅ Dexter installed successfully')"

WORKDIR /workspace

# Configure git identity for agent operations
USER node
RUN git config --global user.name "5D Labs Agent" && \
    git config --global user.email "agent@5dlabs.com"

# Environment defaults
# Default to Claude Sonnet 4 for Anthropic compatibility
# Can be overridden with DEXTER_MODEL env var (gpt-4.1, claude-*, gemini-*)
ENV DEXTER_MODEL="claude-sonnet-4-20250514" \
    DEXTER_MAX_STEPS="20" \
    DEXTER_MAX_STEPS_PER_TASK="5"

USER node

