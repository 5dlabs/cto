# =============================================================================
# Local Runtime Base Image - Simplified for Testing
# =============================================================================
# Minimal image with just what's needed to run CLI tests locally.
# No pre-built binaries required.
# =============================================================================

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install essential tools first
RUN apt-get update && apt-get install -y --no-install-recommends \
    git curl wget ca-certificates gnupg2 \
    jq ripgrep openssh-client \
    build-essential sudo \
    && rm -rf /var/lib/apt/lists/*

# Create node user (matching production) - handle existing GID/UID
RUN groupadd -g 1000 node 2>/dev/null || groupadd node 2>/dev/null || true && \
    useradd --shell /bin/bash --create-home -u 1000 -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home -g node node 2>/dev/null || true && \
    mkdir -p /etc/sudoers.d && \
    echo 'node ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/99-node-nopasswd && \
    chmod 0440 /etc/sudoers.d/99-node-nopasswd

# Install Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g npm@latest && \
    rm -rf /var/lib/apt/lists/*

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | \
    gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | \
    tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Setup workspace
RUN mkdir -p /workspace && chown -R node:node /workspace

WORKDIR /workspace
USER node
