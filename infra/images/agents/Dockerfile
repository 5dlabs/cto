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
#   docker build -t registry.5dlabs.ai/5dlabs/agents .
#   docker build --build-arg OPENCLAW_VERSION=1.2.3 -t registry.5dlabs.ai/5dlabs/agents:1.2.3 .
#
# =============================================================================

ARG BASE_IMAGE=registry.5dlabs.ai/5dlabs/runtime:full

FROM ${BASE_IMAGE}

LABEL org.opencontainers.image.source="https://git.5dlabs.ai/5dlabs/cto"
LABEL org.opencontainers.image.description="CTO AI Agent - Unified multi-agent platform image"
LABEL org.opencontainers.image.licenses="MIT"

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

# Bun runtime - required to compile intake-util during image build
RUN set -eux; \
    curl -fsSL https://bun.sh/install | bash; \
    ln -sf /root/.bun/bin/bun /usr/local/bin/bun; \
    ln -sf /root/.bun/bin/bunx /usr/local/bin/bunx; \
    bun --version

# Core coding CLIs (claude, copilot, codex, every-code, gemini, openai SDK)
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund \
      @anthropic-ai/claude-code \
      @github/copilot \
      @openai/codex \
      @just-every/code \
      @google/gemini-cli \
      openai@latest \
    && echo "npm CLIs installed: claude, copilot, codex, code, gemini, openai"

# =============================================================================
# OpenClaw (agent CLI) - orchestrator that delegates to claude/codex/cursor/etc.
# =============================================================================
ARG OPENCLAW_VERSION=2026.2.12
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    if [ "$OPENCLAW_VERSION" = "latest" ]; then \
        npm install -g --no-audit --no-fund --ignore-scripts openclaw; \
    else \
        npm install -g --no-audit --no-fund --ignore-scripts openclaw@${OPENCLAW_VERSION}; \
    fi && \
    echo "--- OpenClaw (agent CLI) included ---" && \
    openclaw --version

# =============================================================================
# ClawHub CLI — skill registry search and installation
# =============================================================================
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund clawhub@latest && \
    echo "clawhub installed" && \
    clawhub --cli-version

# =============================================================================
# intake-util — deterministic pipeline operations (tally, sync-linear, write-files)
# =============================================================================
COPY apps/intake-util/ /tmp/intake-util-build/
RUN cd /tmp/intake-util-build && \
    npm ci --ignore-scripts && \
    npm run build && \
    cp intake-util /usr/local/bin/intake-util && \
    chmod +x /usr/local/bin/intake-util && \
    cd / && \
    rm -rf /tmp/intake-util-build && \
    intake-util --help >/dev/null 2>&1 || true

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
# Kimi Code CLI
# =============================================================================
RUN set -eux; \
    curl -LsSf https://astral.sh/uv/install.sh | bash; \
    . "$HOME/.local/bin/env"; \
    UV_TOOL_DIR=/opt/uv-tools UV_TOOL_BIN_DIR=/usr/local/bin uv tool install kimi-cli; \
    chmod -R a+rX /opt/uv-tools; \
    kimi --version || echo "kimi installed (version check may require config)"

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
# GitHub CLI (gh) - PR creation, issue management, repo operations
# =============================================================================
RUN (type -p wget >/dev/null || (apt-get update && apt-get install wget -y)) \
    && mkdir -p -m 755 /etc/apt/keyrings \
    && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg | tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
    && chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install gh -y \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# =============================================================================
# System packages — SSH server, browser automation, file watcher
# =============================================================================
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      openssh-server \
      inotify-tools \
      gettext-base \
      wget \
      gnupg2 \
      ca-certificates \
      ffmpeg && \
    ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
      mkdir -p /etc/apt/keyrings && \
      wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /etc/apt/keyrings/google-chrome.gpg && \
      echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/google-chrome.gpg] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list && \
      apt-get update && \
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        google-chrome-stable; \
    else \
      echo "Skipping Google Chrome install for ${ARCH}; Playwright-managed browsers will be used on arm64."; \
    fi && \
    mkdir -p /run/sshd && \
    rm -rf /var/lib/apt/lists/*

# =============================================================================
# Agent runtime directories and config mount points
# =============================================================================

# Per-CLI home directories
RUN mkdir -p /home/node/.openclaw /home/node/.claude /home/node/.code \
             /home/node/.codex /home/node/.copilot /home/node/.cursor /home/node/.gemini \
             /home/node/.kimi /home/node/.opencode /home/node/.config /home/node/.cache \
             /home/node/.local /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /home/node/.claude /home/node/.code \
                       /home/node/.codex /home/node/.copilot /home/node/.cursor /home/node/.gemini \
                       /home/node/.kimi /home/node/.opencode /home/node/.factory /home/node/.config \
                       /home/node/.cache /home/node/.local /workspace

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
COPY infra/images/agents/entrypoint.sh /usr/local/bin/agent-entrypoint
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
    copilot --version 2>/dev/null && echo "copilot        OK" || echo "copilot        SKIP"; \
    codex --version     && echo "codex          OK"; \
    code --version      && echo "code           OK"; \
    timeout 10 gemini --version 2>/dev/null && echo "gemini         OK" || echo "gemini         SKIP"; \
    cursor-agent --version 2>/dev/null && echo "cursor-agent   OK" || echo "cursor-agent   SKIP"; \
    FACTORY_DISABLE_TELEMETRY=1 droid --version && echo "droid          OK"; \
    kimi --version 2>/dev/null && echo "kimi           OK" || echo "kimi           SKIP"; \
    opencode --version  && echo "opencode       OK"; \
    op --version        && echo "op             OK"; \
    gh --version        && echo "gh             OK"; \
    clawhub --cli-version && echo "clawhub        OK"; \
    intake-util --help >/dev/null 2>&1 || true; \
    echo "intake-util    OK"

# Health check — confirms the openclaw gateway process is running
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD ps aux | grep -q "[o]penclaw gateway" || exit 1

# Default: run the agent entrypoint which reads identity from env vars
ENTRYPOINT ["agent-entrypoint"]
CMD ["--help"]
