ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Install additional development tools as root
USER root

# Additional development tools for enhanced workflow
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
        # Code quality and linting
        shellcheck \
        yamllint \
        markdownlint \
        # Development utilities
        tmux \
        htop \
        ncdu \
        silversearcher-ag \
        # Cloud development tools
        awscli \
        # Container tools
        dive \
        # API testing
        httpie \
        # Database clients
        mysql-client \
        sqlite3 \
        # Documentation tools
        pandoc \
        # Version management
        direnv \
        # Process monitoring
        iotop \
        # Network debugging
        mtr-tiny \
        traceroute \
        nmap \
        # Text processing
        sd \
        # Git enhancements
        git-lfs \
        git-extras \
        # Build tools
        ninja-build \
        meson \
        # Compression
        pigz pbzip2 pxz \
        # Security
        gnupg-agent \
        # Development languages
        php-cli \
        ruby-full \
        perl \
        # Additional Python tools
        python3-dev \
        python3-setuptools \
        # Additional Node.js tools
        yarn \
    && rm -rf /var/lib/apt/lists/*

# Install additional tools via direct download
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    # Install Docker Compose v2
    COMPOSE_VERSION=$(curl -s https://api.github.com/repos/docker/compose/releases/latest | jq -r '.tag_name') && \
    curl -fsSL -o /usr/local/bin/docker-compose "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-linux-${DL_ARCH}" && \
    chmod +x /usr/local/bin/docker-compose && \
    # Install GitHub CLI extensions
    gh extension install github/gh-copilot 2>/dev/null || true && \
    gh extension install github/gh-codeql 2>/dev/null || true && \
    # Install additional linters/formatters
    pip3 install --break-system-packages --no-cache-dir \
        black \
        isort \
        flake8 \
        mypy \
        pre-commit \
        ruff \
        yamllint \
        sqlfluff \
        bandit \
        safety \
        pip-tools || true && \
    # Install Node.js development tools
    npm install -g --no-fund \
        eslint \
        prettier \
        typescript \
        @types/node \
        nodemon \
        concurrently \
        cross-env \
        dotenv-cli \
        zx \
        @anthropic-ai/claude-code || true

# Setup workspace and user environment
RUN mkdir -p /workspace /home/node/.claude /home/node/.config /home/node/.cache && \
    chown -R 1000:1000 /home/node/.claude /home/node/.config /home/node/.cache

WORKDIR /workspace

# Install Claude (generic VERSION only) as root, then drop to node
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}

# Configure development environment
USER node

# Setup development shell enhancements
RUN echo 'export EDITOR=vim' >> ~/.bashrc && \
    echo 'export EDITOR=vim' >> ~/.zshrc && \
    echo 'alias ll="ls -la"' >> ~/.bashrc && \
    echo 'alias ll="ls -la"' >> ~/.zshrc && \
    echo 'alias grep="grep --color=auto"' >> ~/.bashrc && \
    echo 'alias grep="grep --color=auto"' >> ~/.zshrc

# Configure git defaults for development
RUN git config --global init.defaultBranch main && \
    git config --global pull.rebase true && \
    git config --global push.autoSetupRemote true && \
    git config --global user.name "Claude Code Agent" && \
    git config --global user.email "claude@5dlabs.com"

# Setup Python virtual environment template
RUN python3 -m venv ~/.venv/base && \
    ~/.venv/base/bin/pip install --upgrade pip setuptools wheel

# Create development helper scripts
RUN echo '#!/bin/bash\n# Development helper script\nset -e\n\nif [ $# -eq 0 ]; then\n    echo "Usage: $0 <command>"\n    echo "Available commands: setup-py, setup-node, setup-go, clean"\n    exit 1\nfi\n\ncase "$1" in\n    setup-py)\n        if [ ! -d "venv" ]; then\n            python3 -m venv venv\n        fi\n        venv/bin/pip install --upgrade pip\n        ;;\n    setup-node)\n        if [ ! -d "node_modules" ]; then\n            npm install\n        fi\n        ;;\n    setup-go)\n        if [ ! -f "go.mod" ]; then\n            go mod init $(basename $(pwd))\n        fi\n        ;;\n    clean)\n        rm -rf venv node_modules target dist build *.log\n        ;;\n    *)\n        echo "Unknown command: $1"\n        exit 1\n        ;;\nesac' > ~/dev-helper.sh && \
    chmod +x ~/dev-helper.sh

# Add helper script to PATH
ENV PATH="/home/node:${PATH}"

# Final setup
USER node
