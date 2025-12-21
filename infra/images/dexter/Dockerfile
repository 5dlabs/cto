# syntax=docker/dockerfile:1.7-labs
# =============================================================================
# Dexter Agent - Autonomous Financial Research Agent
# =============================================================================
# https://github.com/virattt/dexter
#
# Dexter performs deep financial research using task planning,
# self-reflection, and real-time market data via LangChain.
#
# NOTE: Dexter was rewritten from Python to TypeScript/Bun in Dec 2025.
# This Dockerfile uses Bun runtime instead of Python/uv.
#
# Supports both production (runtime base) and local (ubuntu) builds:
#
#   Production/GHCR build (default - uses runtime base):
#     docker build -t ghcr.io/5dlabs/dexter .
#
#   Local/Kind build (uses ubuntu:24.04 for simpler local testing):
#     docker build --build-arg BASE_IMAGE=ubuntu:24.04 -t ghcr.io/5dlabs/dexter:local .
#
# =============================================================================

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ=UTC
ENV TZ="$TZ"
ENV DEVCONTAINER=true
ENV DEBIAN_FRONTEND=noninteractive

USER root

# Install base dependencies (only needed if using ubuntu base, runtime has these)
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    ca-certificates \
    unzip \
    jq \
    && rm -rf /var/lib/apt/lists/* || true

# Create non-root user if it doesn't exist (runtime image has 'node' user)
RUN if ! id -u node > /dev/null 2>&1; then \
        if ! getent group node > /dev/null 2>&1; then \
            groupadd -g 1000 node 2>/dev/null || groupadd node; \
        fi && \
        useradd --shell /bin/bash --create-home -u 1000 -g node node 2>/dev/null || \
        useradd --shell /bin/bash --create-home -g node node; \
    fi

# Setup workspace and ensure correct ownership
RUN mkdir -p /workspace /home/node/.cache /home/node/.dexter /home/node/.bun && \
    chown -R 1000:1000 /home/node /workspace || true

WORKDIR /workspace

# Install Bun (JavaScript/TypeScript runtime)
RUN --mount=type=cache,target=/root/.cache \
    curl -fsSL https://bun.sh/install | bash && \
    ln -sf /root/.bun/bin/bun /usr/local/bin/bun && \
    ln -sf /root/.bun/bin/bunx /usr/local/bin/bunx
ENV PATH="/root/.bun/bin:$PATH"

# Clone Dexter repository and install dependencies with Bun
# Dexter is now a TypeScript/Bun project (rewritten Dec 2025)
ARG DEXTER_VERSION=latest
RUN git clone https://github.com/virattt/dexter.git /opt/dexter && \
    cd /opt/dexter && \
    if [ "$DEXTER_VERSION" != "latest" ]; then \
        git checkout "v${DEXTER_VERSION}"; \
    fi && \
    ls -la && \
    bun install && \
    # Create wrapper script for dexter command
    echo '#!/bin/bash' > /usr/local/bin/dexter-agent && \
    echo 'cd /opt/dexter && exec bun run src/index.ts "$@"' >> /usr/local/bin/dexter-agent && \
    chmod +x /usr/local/bin/dexter-agent

# Verify installation
WORKDIR /opt/dexter
RUN bun run --help || echo "Bun installed, dexter ready"

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
