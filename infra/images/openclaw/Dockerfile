# syntax=docker/dockerfile:1
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenClaw from GitHub releases
# Downloads the latest release and extracts binaries
ARG VERSION=latest

RUN set -eo pipefail && \
    # Determine version to download (OpenClaw uses date-based versioning like 2026.2.3)
    if [ "$VERSION" = "latest" ]; then \
        VERSION=$(curl -fsSL "https://api.github.com/repos/openclaw/openclaw/releases/latest" | jq -r '.name' | sed 's/OpenClaw-//'); \
    fi && \
    echo "📦 Installing OpenClaw version: $VERSION" && \
    # Download and extract OpenClaw binary (tgz format)
    curl -fsSL "https://github.com/openclaw/openclaw/releases/download/OpenClaw-${VERSION}/openclaw-${VERSION}.tgz" | \
        tar -xzf - -C /usr/local/bin && \
    chmod +x /usr/local/bin/openclaw && \
    # Verify installation
    openclaw --version && \
    # Print changelog
    echo "" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    echo "📋 OpenClaw Changelog (${VERSION})" && \
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" && \
    curl -fsSL "https://api.github.com/repos/openclaw/openclaw/releases/latest" | \
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
