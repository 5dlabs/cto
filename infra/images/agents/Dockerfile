# syntax=docker/dockerfile:1
# =============================================================================
# Unified Agent Image
# =============================================================================
#
# Single container image for ALL CTO platform agents. Agent identity is
# determined at runtime via environment variables and mounted config, NOT
# by which image is used.
#
# This replaces the previous per-agent image model (openclaw, claude, cursor,
# codex, etc.) with a single image that contains every tool an agent might
# need. The OpenClaw orchestrator selects which CLI to invoke based on the
# agent's configuration in cto-config.json.
#
# Runtime identity (set via env vars or K8s pod spec):
#   AGENT_NAME      - Agent identifier (e.g., "rex", "morgan", "blaze")
#   AGENT_ROLE      - Role description (e.g., "rust-engineer", "pm", "frontend")
#   AGENT_CLI       - Preferred coding CLI (claude, cursor, codex, etc.)
#   AGENT_MODEL     - Default LLM model for this agent
#   GITHUB_APP_NAME - GitHub App identity (e.g., "5DLabs-Rex")
#
# Mounted at runtime:
#   /etc/cto/config.json   - Canonical cto-config.json (read-only)
#   /etc/cto/workflows/    - Lobster workflow definitions
#   /etc/cto/templates/    - Agent prompt templates and skills
#
# Build:
#   docker build -t ghcr.io/5dlabs/agents .
#   docker build --build-arg VERSION=1.2.3 -t ghcr.io/5dlabs/agents:1.2.3 .
#
# =============================================================================

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:full

FROM ${BASE_IMAGE}

USER root

# =============================================================================
# 1Password CLI (op) - headless secret access for agents
# =============================================================================
RUN set -eux; \
    OP_VERSION="v2.30.3"; \
    ARCH=$(dpkg --print-architecture); \
    case "$ARCH" in \
      amd64) OP_ARCH=amd64 ;; \
      arm64) OP_ARCH=arm64 ;; \
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac; \
    curl -fsSL "https://cache.agilebits.com/dist/1P/op2/pkg/${OP_VERSION}/op_linux_${OP_ARCH}_${OP_VERSION}.zip" -o /tmp/op.zip && \
    unzip -o /tmp/op.zip -d /usr/local/bin && \
    rm -f /tmp/op.zip && \
    chmod +x /usr/local/bin/op && \
    op --version

# =============================================================================
# npm CLIs - All coding agent backends
# =============================================================================
ARG VERSION=latest

# Core coding CLIs (claude, codex, every-code, gemini, openai SDK)
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g \
      @anthropic-ai/claude-code \
      @openai/codex \
      @just-every/code \
      @google/gemini-cli \
      openai@latest \
    && echo "npm CLIs installed: claude, codex, code, gemini, openai"

# OpenClaw orchestrator
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    if [ "$VERSION" = "latest" ]; then \
        npm install -g openclaw; \
    else \
        npm install -g openclaw@${VERSION}; \
    fi

# =============================================================================
# Cursor Agent CLI
# =============================================================================
RUN set -eux; \
    curl https://cursor.com/install -fsS | bash; \
    export PATH="$HOME/.local/bin:$PATH"; \
    CURSOR_SHARE="$HOME/.local/share/cursor-agent"; \
    VERSION_DIR="$(cd "$CURSOR_SHARE/versions" && ls -1 | sort | tail -n1)"; \
    install -d /usr/local/lib/cursor-agent; \
    cp -r "$CURSOR_SHARE/versions/$VERSION_DIR"/. /usr/local/lib/cursor-agent/; \
    ln -sf /usr/bin/node /usr/local/lib/cursor-agent/node; \
    printf '#!/bin/bash\nexec /usr/local/lib/cursor-agent/cursor-agent "$@"\n' > /usr/local/bin/cursor-agent; \
    chmod +x /usr/local/bin/cursor-agent; \
    ln -sf cursor-agent /usr/local/bin/cursor; \
    chown -R 1000:1000 /usr/local/lib/cursor-agent; \
    rm -rf "$CURSOR_SHARE"; \
    cursor-agent --version || echo "Warning: cursor-agent version check failed"

# =============================================================================
# Factory Droid CLI
# =============================================================================
RUN set -eux; \
    curl -fsSL https://app.factory.ai/cli | sh; \
    install -m 0755 /root/.local/bin/droid /usr/local/bin/droid; \
    rm -f /root/.local/bin/droid; \
    install -d -m 0755 /home/node/.factory/bin /home/node/.factory/logs /home/node/.factory/cache; \
    ln -sf /usr/local/bin/rg /home/node/.factory/bin/rg 2>/dev/null || true; \
    printf '{}' > /home/node/.factory/settings.json; \
    chown -R 1000:1000 /home/node/.factory; \
    FACTORY_DISABLE_TELEMETRY=1 droid --version

# =============================================================================
# OpenCode CLI - binary from GitHub releases
# =============================================================================
ARG OPENCODE_VERSION=1.1.44
RUN set -eux; \
    ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) ARCH_NAME="x64" ;; \
        aarch64|arm64) ARCH_NAME="arm64" ;; \
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/anomalyco/opencode/releases/download/v${OPENCODE_VERSION}/opencode-linux-${ARCH_NAME}.tar.gz" -o /tmp/opencode.tar.gz; \
    tar -xzf /tmp/opencode.tar.gz -C /tmp; \
    install -m 0755 /tmp/opencode /usr/local/bin/opencode; \
    rm -rf /tmp/opencode.tar.gz /tmp/opencode; \
    opencode --version

# =============================================================================
# inotify-tools (Linux file watcher for cto-services watch command)
# =============================================================================
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      inotify-tools gettext-base && \
    rm -rf /var/lib/apt/lists/*

# =============================================================================
# Agent runtime directories and config mount points
# =============================================================================

# Per-CLI home directories
RUN mkdir -p /home/node/.openclaw /home/node/.claude /home/node/.code \
             /home/node/.codex /home/node/.cursor /home/node/.gemini /home/node/.opencode \
             /home/node/.config /home/node/.cache /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /home/node/.claude /home/node/.code \
                       /home/node/.codex /home/node/.cursor /home/node/.gemini /home/node/.opencode \
                       /home/node/.factory /home/node/.config /home/node/.cache /workspace

# Config mount points (populated at runtime via K8s ConfigMap/Secret mounts)
RUN mkdir -p /etc/cto/workflows /etc/cto/templates /etc/cto/skills && \
    chown -R 1000:1000 /etc/cto

# Configure git identity (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

# =============================================================================
# Agent entrypoint script
# =============================================================================
# This script reads AGENT_* env vars to configure the agent's identity at
# startup, then delegates to openclaw which orchestrates the actual work.
COPY entrypoint.sh /usr/local/bin/agent-entrypoint
RUN chmod +x /usr/local/bin/agent-entrypoint

ENV HOME=/home/node

# Drop privileges back to non-root user
USER node
WORKDIR /workspace

# =============================================================================
# Verify all CLIs are installed
# =============================================================================
RUN set -e; \
    openclaw --version  && echo "openclaw       OK"; \
    claude --version    && echo "claude         OK"; \
    codex --version     && echo "codex          OK"; \
    code --version      && echo "code           OK"; \
    gemini --version    && echo "gemini         OK"; \
    cursor-agent --version 2>/dev/null && echo "cursor-agent   OK" || echo "cursor-agent   SKIP"; \
    FACTORY_DISABLE_TELEMETRY=1 droid --version && echo "droid          OK"; \
    opencode --version  && echo "opencode       OK"; \
    op --version        && echo "op             OK"

# Default: run the agent entrypoint which reads identity from env vars
ENTRYPOINT ["agent-entrypoint"]
CMD ["--help"]
