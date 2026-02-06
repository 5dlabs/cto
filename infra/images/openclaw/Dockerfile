# syntax=docker/dockerfile:1
# =============================================================================
# OpenClaw Agent Image - Composable multi-CLI agent image
# =============================================================================
#
# Composes multiple CLI tools into a single agent image using Docker
# multi-stage COPY --from= to pull binaries from pre-built images.
# Each CLI image is built independently by agents-build.yaml and
# referenced here by tag. No Dockerfile chaining or workflow changes needed.
#
# Includes:
#   - Runtime base (tools binary, git, node, npm, gh, ripgrep, jq, curl)
#   - Claude Code CLI (copied from ghcr.io/5dlabs/claude:latest)
#   - OpenClaw CLI (installed via npm)
#   - 1Password CLI (op) for headless secret access
#
# =============================================================================

# Pull Claude Code CLI from pre-built image (updated daily by agents-build.yaml)
# Uses the *previous* build's :latest tag since both build in parallel
FROM ghcr.io/5dlabs/claude:latest AS claude-cli

# Main image extends the runtime base
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:full
FROM ${BASE_IMAGE}

USER root

# ---------------------------------------------------------------------------
# 1Password CLI (op) - headless secret access for agents
# Agents use OP_SERVICE_ACCOUNT_TOKEN (injected via K8s secret) for auth
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
# Claude Code CLI - copied from pre-built image layer
# Avoids separate npm install; version tracks the daily claude:latest build
# ---------------------------------------------------------------------------
COPY --from=claude-cli /usr/local/lib/node_modules/@anthropic-ai /usr/local/lib/node_modules/@anthropic-ai
RUN ln -sf /usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js /usr/local/bin/claude && \
    chmod +x /usr/local/bin/claude /usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js

# ---------------------------------------------------------------------------
# OpenClaw CLI - installed via npm
# ---------------------------------------------------------------------------
ARG VERSION=latest

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    if [ "$VERSION" = "latest" ]; then \
        npm install -g openclaw; \
    else \
        npm install -g openclaw@${VERSION}; \
    fi && \
    echo "" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    echo "📋 OpenClaw Changelog" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    npm view openclaw changelog --json 2>/dev/null | head -50 || echo "Run 'openclaw --help' for usage" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ---------------------------------------------------------------------------
# inotify-tools (Linux file watcher for cto-services watch command)
# ---------------------------------------------------------------------------
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends inotify-tools && \
    rm -rf /var/lib/apt/lists/*

# ---------------------------------------------------------------------------
# Workspace and home directory setup (for both OpenClaw and Claude Code)
# ---------------------------------------------------------------------------
RUN mkdir -p /home/node/.openclaw /home/node/.claude /home/node/.config /home/node/.cache /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /home/node/.claude /home/node/.config /home/node/.cache /workspace

# Configure git identity (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

ENV HOME=/home/node

# Drop privileges back to non-root user
USER node
WORKDIR /workspace

# Verify all CLIs are installed
RUN openclaw --version && echo "✅ OpenClaw CLI ready" && \
    claude --version 2>/dev/null && echo "✅ Claude Code CLI ready" && \
    op --version && echo "✅ 1Password CLI ready"

CMD ["openclaw", "--help"]
