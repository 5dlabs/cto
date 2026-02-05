# syntax=docker/dockerfile:1
# OpenClaw Agent Image - CLI tool from npm
# Based on runtime:minimal with OpenClaw CLI installed via npm

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenClaw CLI from npm
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
