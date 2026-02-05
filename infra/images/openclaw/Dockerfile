# syntax=docker/dockerfile:1
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenClaw from GitHub releases
# Downloads the latest release and extracts binaries
ARG VERSION=latest

RUN set -euo pipefail && \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
        amd64) OPENCLAW_ARCH="x86_64" ;; \
        arm64) OPENCLAW_ARCH="aarch64" ;; \
        *) OPENCLAW_ARCH="$ARCH" ;; \
    esac && \
    # Determine version to download
    if [ "$VERSION" = "latest" ]; then \
        TAG=$(curl -fsSL "https://api.github.com/repos/openclaw/openclaw/releases/latest" | jq -r '.tag_name'); \
    else \
        TAG="v${VERSION}"; \
    fi && \
    echo "📦 Installing OpenClaw version: $TAG" && \
    # Download and extract OpenClaw binaries
    curl -fsSL "https://github.com/openclaw/openclaw/releases/download/${TAG}/openclaw-${OPENCLAW_ARCH}-unknown-linux-gnu.tar.gz" | \
        tar -xzf - -C /usr/local/bin && \
    chmod +x /usr/local/bin/openclaw && \
    # Verify installation
    openclaw --version && \
    # Print changelog
    echo "" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    echo "📋 OpenClaw Changelog (${TAG})" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    curl -fsSL "https://api.github.com/repos/openclaw/openclaw/releases/tags/${TAG}" | \
        jq -r '.body // "No changelog available"' | head -100 && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create config and workspace directories
RUN mkdir -p /home/node/.openclaw /workspace && \
    chown -R 1000:1000 /home/node/.openclaw /workspace

ENV HOME=/home/node

# Drop privileges back to non-root user
USER node
WORKDIR /workspace

# Verify OpenClaw installation
RUN openclaw --version && echo "✅ OpenClaw CLI ready"

CMD ["openclaw", "--help"]
