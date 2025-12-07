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
    curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:$PATH"

# Install Dexter and dependencies
# Uses pip with --break-system-packages since we're in a container
ARG DEXTER_VERSION=latest
RUN --mount=type=cache,target=/root/.cache/pip \
    if [ "$DEXTER_VERSION" = "latest" ]; then \
      pip install --break-system-packages \
        git+https://github.com/virattt/dexter.git; \
    else \
      pip install --break-system-packages \
        git+https://github.com/virattt/dexter.git@v${DEXTER_VERSION}; \
    fi

# Verify installation
RUN python3 -c "from dexter.agent import Agent; print('✅ Dexter installed successfully')"

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

