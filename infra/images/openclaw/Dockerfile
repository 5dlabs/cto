# syntax=docker/dockerfile:1
# =============================================================================
# OpenClaw Agent Image - Unified multi-CLI agent image
# =============================================================================
#
# Bundles ALL coding agent CLIs into a single image so OpenClaw agents can
# invoke any backend (Claude, Codex, Cursor, Factory, OpenCode, Every Code).
#
# CLIs included:
#   - Claude Code    (@anthropic-ai/claude-code)   — npm
#   - OpenAI Codex   (@openai/codex)               — npm
#   - Every Code     (@just-every/code)             — npm
#   - Cursor Agent   (cursor-agent)                 — curl installer
#   - Factory Droid  (droid)                        — curl installer
#   - Gemini CLI     (@google/gemini-cli)            — npm
#   - OpenCode       (opencode)                     — GitHub release binary
#   - OpenClaw       (openclaw)                     — npm (orchestrator)
#   - 1Password CLI  (op)                           — zip download
#
# =============================================================================

# ARG before any FROM so it's available for FROM instructions and --build-arg
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:full

FROM ${BASE_IMAGE}

USER root

# ---------------------------------------------------------------------------
# 1Password CLI (op) - headless secret access for agents
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# npm CLIs - Claude Code, Codex, Every Code, OpenAI SDK, OpenClaw
# ---------------------------------------------------------------------------
ARG VERSION=2026.2.6-3

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g \
      @anthropic-ai/claude-code \
      @openai/codex \
      @just-every/code \
      @google/gemini-cli \
      openai@latest \
    && echo "npm CLIs installed: claude, codex, code, gemini, openai"

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g openclaw@${VERSION}

# ---------------------------------------------------------------------------
# Cursor Agent CLI - curl installer + system-wide placement
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# Factory Droid CLI - curl installer
# ---------------------------------------------------------------------------
RUN set -eux; \
    curl -fsSL https://app.factory.ai/cli | sh; \
    install -m 0755 /root/.local/bin/droid /usr/local/bin/droid; \
    rm -f /root/.local/bin/droid; \
    install -d -m 0755 /home/node/.factory/bin /home/node/.factory/logs /home/node/.factory/cache; \
    ln -sf /usr/local/bin/rg /home/node/.factory/bin/rg 2>/dev/null || true; \
    printf '{}' > /home/node/.factory/settings.json; \
    chown -R 1000:1000 /home/node/.factory; \
    FACTORY_DISABLE_TELEMETRY=1 droid --version

# ---------------------------------------------------------------------------
# OpenCode CLI - binary from GitHub releases
# ---------------------------------------------------------------------------
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

# ---------------------------------------------------------------------------
# inotify-tools (Linux file watcher for cto-services watch command)
# ---------------------------------------------------------------------------
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      inotify-tools gettext-base && \
    rm -rf /var/lib/apt/lists/*

# ---------------------------------------------------------------------------
# Workspace and home directory setup
# ---------------------------------------------------------------------------
RUN mkdir -p /home/node/.openclaw /home/node/.claude /home/node/.code \
             /home/node/.codex /home/node/.cursor /home/node/.gemini /home/node/.opencode \
             /home/node/.config /home/node/.cache /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /home/node/.claude /home/node/.code \
                       /home/node/.codex /home/node/.cursor /home/node/.gemini /home/node/.opencode \
                       /home/node/.factory /home/node/.config /home/node/.cache /workspace

# Configure git identity (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

ENV HOME=/home/node

# Drop privileges back to non-root user
USER node
WORKDIR /workspace

# Verify all CLIs are installed
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

CMD ["openclaw", "--help"]
