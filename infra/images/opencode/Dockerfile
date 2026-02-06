# =============================================================================
# OpenCode CLI Agent Image - Unified Dockerfile
# =============================================================================
#
# Supports both production (runtime base) and local (node slim) builds:
#
#   Production/GHCR build (default - uses runtime base):
#     docker build -t ghcr.io/5dlabs/opencode .
#
#   Local/Kind build (uses node:18-slim for simpler local testing):
#     docker build --build-arg BASE_IMAGE=node:18-slim -t ghcr.io/5dlabs/opencode:local .
#
# =============================================================================

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ=UTC
ENV TZ="$TZ"
ENV DEVCONTAINER=true

USER root

# Create workspace and user directories (node user is uid 1001 in runtime image)
RUN mkdir -p /workspace /home/node/.opencode /home/node/.cache/opencode && \
    chown -R node:ubuntu /workspace /home/node/.opencode /home/node/.cache

WORKDIR /workspace

# Install Open Code CLI directly from GitHub releases
# The installer script can fail in containers, so we download the binary directly
# Asset naming: opencode-linux-{x64|arm64}.tar.gz
ARG OPENCODE_VERSION=1.1.44
RUN set -eux; \
    # Detect architecture and map to OpenCode naming
    ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) ARCH_NAME="x64" ;; \
        aarch64|arm64) ARCH_NAME="arm64" ;; \
        *) echo "Unsupported architecture: $ARCH"; exit 1 ;; \
    esac; \
    # Download from GitHub releases (format: opencode-linux-{arch}.tar.gz)
    DOWNLOAD_URL="https://github.com/anomalyco/opencode/releases/download/v${OPENCODE_VERSION}/opencode-linux-${ARCH_NAME}.tar.gz"; \
    echo "Downloading OpenCode v${OPENCODE_VERSION} from ${DOWNLOAD_URL}"; \
    curl -fsSL "${DOWNLOAD_URL}" -o /tmp/opencode.tar.gz; \
    # Extract and install
    tar -xzf /tmp/opencode.tar.gz -C /tmp; \
    install -m 0755 /tmp/opencode /usr/local/bin/opencode; \
    rm -rf /tmp/opencode.tar.gz /tmp/opencode; \
    # Verify installation
    opencode --version && echo "✅ OpenCode v${OPENCODE_VERSION} installed successfully"

# Install OpenAI Node.js SDK for compatibility
RUN npm install -g openai@latest && \
    echo "✅ OpenAI Node.js SDK installed" && \
    (node -e "console.log('OpenAI Node.js SDK version:', require('openai/package.json').version)" || echo "⚠️ Node.js SDK verification failed")

# Ensure cache directory ownership (using node user uid 1001, group ubuntu gid 1000)
RUN mkdir -p /home/node/.cache/opencode /home/node/.opencode && \
    chown -R node:ubuntu /home/node/.cache /home/node/.opencode && \
    chmod -R 755 /home/node/.cache /home/node/.opencode

USER node

# Set environment for OpenCode
ENV HOME=/home/node

# Open Code CLI is now available via `opencode`
