# syntax=docker/dockerfile:1.7-labs
# =============================================================================
# Runtime Base Image - Unified Dockerfile
# =============================================================================
#
# Multi-target build supporting different use cases:
#
#   Minimal (for agents - default target):
#     docker build -t registry.5dlabs.ai/5dlabs/runtime .
#     docker build --target minimal -t registry.5dlabs.ai/5dlabs/runtime:minimal .
#     Contains: git, Node.js, npm, ripgrep, jq, curl, GitHub CLI
#     Size: ~500MB (vs ~2GB for full)
#
#   Full development environment:
#     docker build --target production -t registry.5dlabs.ai/5dlabs/runtime:full .
#     Contains: All development tools, languages, k8s CLIs, security scanners
#     Size: ~2GB
#
#   CI/GHCR build (downloads binaries from releases):
#     docker build -t registry.5dlabs.ai/5dlabs/runtime .
#
#   Local/Kind build (compiles intake/pm-activity from source):
#     docker build --target local -t registry.5dlabs.ai/5dlabs/runtime:local .
#
# =============================================================================

# =============================================================================
# BUILD STAGES (used only for local development builds with --target local)
# =============================================================================
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}rust:latest AS intake-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release --package intake --bin intake && \
    cargo build --release --package pm --bin pm-activity

# =============================================================================
# MINIMAL RUNTIME IMAGE (for agents) - DEFAULT TARGET
# =============================================================================
# Contains only tools essential for agent execution:
# - git, Node.js, npm, npx (all agents)
# - curl, jq (API/config operations)
# - ripgrep (MCP tools: warp_grep, search)
# - GitHub CLI (PR operations, code review)
# - Basic shell utilities
# =============================================================================
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}ubuntu:24.04 AS minimal-base
SHELL ["/bin/bash", "-lc"]

ARG TZ
ENV TZ="$TZ"

# Essential packages only (single layer for efficiency)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get -o Acquire::ForceIPv4=true update && \
    DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::ForceIPv4=true install -y --no-install-recommends \
      git procps sudo unzip gnupg2 ca-certificates curl wget \
      jq ripgrep openssh-client strace lsof \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user 'node' with stable UID/GID 1000
RUN groupadd -g 1000 node 2>/dev/null || groupadd node 2>/dev/null || true && \
    useradd --shell /bin/bash --create-home -u 1000 -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home node || true && \
    id node && getent group node && \
    echo 'node ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/99-node-nopasswd && \
    chmod 0440 /etc/sudoers.d/99-node-nopasswd

# Install Node.js 22 LTS (required for modern MCP servers like firecrawl-mcp)
RUN --mount=type=cache,target=/root/.cache \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) NODE_ARCH=x64 ;; arm64) NODE_ARCH=arm64 ;; *) NODE_ARCH=$ARCH ;; esac && \
    NODE_VER=$(curl -fsSL https://nodejs.org/dist/index.json | jq -r '[.[] | select(.lts != false and (.version | startswith("v22")))][0].version') && \
    curl -fsSL "https://nodejs.org/dist/${NODE_VER}/node-${NODE_VER}-linux-${NODE_ARCH}.tar.xz" | tar -xJ -C /usr/local --strip-components=1 && \
    rm -rf /var/lib/apt/lists/*

# Install Stitch SDK globally so design-intake dependencies
# are always present in runtime images.
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --no-fund @google/stitch-sdk

# Install GitHub CLI (essential for PR operations)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=${ARCH} signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get -o Acquire::ForceIPv4=true update && \
    apt-get -o Acquire::ForceIPv4=true install -y gh && \
    rm -rf /var/lib/apt/lists/*

# GitLab CLI (glab) for dual SCM support
RUN GLAB_VERSION="1.91.0" && \
    ARCH=$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/') && \
    curl -sLo /tmp/glab.tar.gz "https://gitlab.com/gitlab-org/cli/-/releases/v${GLAB_VERSION}/downloads/glab_${GLAB_VERSION}_linux_${ARCH}.tar.gz" && \
    tar -xzf /tmp/glab.tar.gz -C /tmp && \
    mv /tmp/bin/glab /usr/local/bin/glab && \
    chmod +x /usr/local/bin/glab && \
    rm -rf /tmp/glab* /tmp/bin

# Install CTO binaries from pre-downloaded files in build context
COPY .docker-binaries/ /tmp/binaries/

ARG TOOLS_VERSION=skip
RUN if [ "${TOOLS_VERSION:-skip}" != "skip" ] && [ -n "${TOOLS_VERSION:-}" ] && [ -f /tmp/binaries/tools-client-linux-x86_64 ]; then \
      mv /tmp/binaries/tools-client-linux-x86_64 /usr/local/bin/tools && \
      chmod +x /usr/local/bin/tools; \
    fi

RUN rm -rf /tmp/binaries

# Configure git defaults
RUN git config --global init.defaultBranch main && \
    git config --global pull.rebase true && \
    git config --global push.autoSetupRemote true

# Prepare npm global and permissions
RUN mkdir -p /usr/local/share/npm-global/lib && chown -R node:node /usr/local/share/npm-global
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin:/home/node/.cargo/bin

# Setup workspace
RUN mkdir -p /workspace && chown -R node:node /workspace

# Copy GitHub App authentication script
COPY scripts/2026-01/github-app-auth.sh /usr/local/bin/github-app-auth
RUN chmod +x /usr/local/bin/github-app-auth

# =============================================================================
# 1Password CLI (op) for headless secret access
# Authenticated via OP_SERVICE_ACCOUNT_TOKEN env var (injected as K8s secret)
# Docs: https://developer.1password.com/docs/service-accounts/use-with-1password-cli
# =============================================================================
COPY --from=1password/op:2 /usr/local/bin/op /usr/local/bin/op

USER node
WORKDIR /workspace
ENV DEVCONTAINER=true

# =============================================================================
# MINIMAL TARGET (default for agents)
# =============================================================================
FROM minimal-base AS minimal

# Verify essential tools
RUN node --version && npm --version && git --version && gh --version && glab --version && op --version

# =============================================================================
# TRADER TARGET (for autonomous trading agent)
# =============================================================================
#   docker build --target trader -t registry.5dlabs.ai/5dlabs/runtime:trader .
#   Contains: minimal + Python 3.12 + all npm/pip trading deps + Solana CLI
#   Size: ~800MB estimated
# =============================================================================
FROM minimal-base AS trader

# Python 3.12 + pip (needed by 9 trading skills)
USER root
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get -o Acquire::ForceIPv4=true update && \
    DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::ForceIPv4=true install -y --no-install-recommends \
      python3 python3-pip python3-venv python-is-python3 python3-dev \
      build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Solana CLI tools (solana, spl-token — needed by solana-swaps, solana-transfer)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)" && \
    ln -sf /root/.local/share/solana/install/active_release/bin/solana /usr/local/bin/solana && \
    ln -sf /root/.local/share/solana/install/active_release/bin/spl-token /usr/local/bin/spl-token && \
    ln -sf /root/.local/share/solana/install/active_release/bin/solana-keygen /usr/local/bin/solana-keygen || true

# npm: all trading skill dependencies (pre-installed globally to avoid per-skill npm install)
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --no-fund \
      @solana/web3.js \
      @solana/spl-token \
      @coral-xyz/borsh \
      @raydium-io/raydium-sdk \
      bs58 \
      ws \
      axios \
      dotenv \
      commander \
      express \
      bn.js \
      pngjs \
      tweetnacl \
      tsx \
    && chown -R node:node /usr/local/share/npm-global

# pip: all trading skill dependencies (pre-installed to avoid per-skill pip install)
RUN --mount=type=cache,target=/root/.cache/pip \
    pip3 install --break-system-packages --no-cache-dir \
      simmer-sdk \
      simmer-mcp \
      ccxt \
      httpx \
      aiohttp \
      matplotlib \
      numpy \
      pandas \
      scipy \
      statsmodels \
      python-dotenv \
      requests \
      solana \
      solders

# Grok X Search MCP server (for sentiment-driven Polymarket trading)
COPY apps/grok/ /opt/grok-x-search/
RUN cd /opt/grok-x-search && npm install --no-fund && npm run build 2>/dev/null || true

USER node
WORKDIR /workspace

# Verify critical tools
RUN node --version && npm --version && python3 --version && pip3 --version && \
    node -e "require('@solana/web3.js')" && \
    python3 -c "import simmer_sdk; print('simmer-sdk OK')" && \
    echo "Trader runtime ready"

# =============================================================================
# MAIN RUNTIME IMAGE (full development environment)
# =============================================================================
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}ubuntu:24.04 AS production

# Use bash for complex RUN steps
SHELL ["/bin/bash", "-lc"]

ARG TZ
ENV TZ="$TZ"

# =============================================================================
# Frequently-updated tool versions (defined early, used late for better caching)
# Changing these ARGs only invalidates layers AFTER they are used
# =============================================================================
# TaskMaster removed - using Rust intake CLI instead

# Core tools and CLIs (cache apt metadata)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get -o Acquire::ForceIPv4=true update && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::ForceIPv4=true install -y --no-install-recommends \
  # Version control and development
  git procps sudo fzf zsh man-db unzip gnupg2 \
  # Networking tools
  iproute2 dnsutils netcat-openbsd nmap net-tools tcpdump tshark \
  ethtool bridge-utils vlan arping hping3 iperf3 socat telnet openssh-client \
  mtr-tiny traceroute \
  # System utilities
  jq ca-certificates curl wget docker.io postgresql-client redis-tools tree \
  build-essential binutils pkg-config libssl-dev lldb lld clang mold libpq-dev \
  # Programming languages
  python3 python3-venv python3-pip python-is-python3 \
  # Debugging and monitoring
  strace lsof htop psmisc ripgrep fd-find bat httpie openssl \
  # Code quality and linting
  shellcheck yamllint markdownlint \
  # Development utilities
  tmux ncdu silversearcher-ag \
  # Database clients
  mysql-client sqlite3 \
  # Documentation tools
  pandoc \
  # Version management
  direnv \
  # Process monitoring
  iotop \
  # Text processing
  sd gettext-base \
  # Git enhancements
  git-lfs git-extras \
  # Build tools
  ninja-build meson \
  # Compression
  pigz pbzip2 \
  # Security
  gnupg-agent \
  # Development languages
  php-cli ruby-full perl \
  # Additional Python tools
  python3-dev python3-setuptools \
  # Additional Node.js tools
  yarn \
  && rm -rf /var/lib/apt/lists/* \
  && update-alternatives --install /usr/local/bin/fd fd /usr/bin/fdfind 10 \
  && update-alternatives --install /usr/local/bin/bat bat /usr/bin/batcat 10

# Create non-root user 'node' with stable UID/GID 1000 to match cache mounts
RUN groupadd -g 1000 node 2>/dev/null || groupadd node 2>/dev/null || true && \
    useradd --shell /bin/bash --create-home -u 1000 -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home node || true && \
    id node && getent group node

# Passwordless sudo for the non-root user (node)
RUN echo 'node ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/99-node-nopasswd \
  && chmod 0440 /etc/sudoers.d/99-node-nopasswd

# Install Node.js 22 LTS (required for modern MCP servers like firecrawl-mcp)
RUN --mount=type=cache,target=/root/.cache \
  ARCH=$(dpkg --print-architecture) \
  && case "$ARCH" in amd64) NODE_ARCH=x64 ;; arm64) NODE_ARCH=arm64 ;; *) NODE_ARCH=$ARCH ;; esac \
  && NODE_VER=$(curl -fsSL https://nodejs.org/dist/index.json | jq -r '[.[] | select(.lts != false and (.version | startswith("v22")))][0].version') \
  && curl -fsSL "https://nodejs.org/dist/${NODE_VER}/node-${NODE_VER}-linux-${NODE_ARCH}.tar.xz" | tar -xJ -C /usr/local --strip-components=1 \
  && rm -rf /var/lib/apt/lists/*

# Keep Stitch SDK available in full runtime variants too.
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --no-fund @google/stitch-sdk

# Enable corepack for pnpm only (skip yarn since we use system yarn)
RUN if command -v corepack >/dev/null 2>&1; then \
    corepack enable pnpm && \
    corepack prepare pnpm@latest --activate; \
  fi

# Install latest stable Go (robust arch/version selection)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    set -eux; \
  ARCH=$(dpkg --print-architecture); \
  case "$ARCH" in \
    amd64) GO_ARCH=amd64 ;; \
    arm64) GO_ARCH=arm64 ;; \
    *) GO_ARCH=$ARCH ;; \
  esac; \
  GO_VER=""; GO_URL=""; \
  if curl -fsSL https://go.dev/dl/?mode=json -o /tmp/go.json; then \
    GO_VER=$(jq -r '[.[] | select(.stable==true)][0].version' /tmp/go.json || true); \
    GO_URL=$(jq -r --arg ver "$GO_VER" --arg arch "$GO_ARCH" '.[] | select(.version==$ver) | .files[] | select(.os=="linux" and .arch==$arch and .kind=="archive") | "https://go.dev" + .url' /tmp/go.json || true); \
  fi; \
  if [ -z "$GO_VER" ] || [ "$GO_VER" = "null" ]; then GO_VER=$(curl -fsSL https://go.dev/VERSION?m=text); fi; \
  if [ -z "$GO_URL" ] || [ "$GO_URL" = "null" ]; then GO_URL="https://go.dev/dl/${GO_VER}.linux-${GO_ARCH}.tar.gz"; fi; \
  curl -fsSL "$GO_URL" -o /tmp/go.tgz; \
  rm -rf /usr/local/go; \
  (tar -C /usr/local -xzf /tmp/go.tgz || (apt-get -o Acquire::ForceIPv4=true update && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::ForceIPv4=true install -y --no-install-recommends golang-go && ln -sfn /usr/lib/go /usr/local/go)); \
  rm -f /tmp/go.tgz
ENV GOROOT=/usr/local/go
ENV PATH=$GOROOT/bin:/usr/lib/go/bin:$PATH

# Install kubectl via apt repository (more reliable than direct download)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.31/deb/Release.key | gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg \
  && echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.31/deb/ /' | tee /etc/apt/sources.list.d/kubernetes.list \
  && apt-get -o Acquire::ForceIPv4=true update && apt-get -o Acquire::ForceIPv4=true install -y kubectl && rm -rf /var/lib/apt/lists/*

# =============================================================================
# Kubernetes and DevOps CLIs (consolidated into single layer for efficiency)
# =============================================================================
# Tools: helm, argocd, argo, talosctl, yq, stern, docker-compose, k9s, kustomize, cloudflared
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    set -eux && \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64; AWS_ARCH=x86_64 ;; arm64) DL_ARCH=arm64; AWS_ARCH=aarch64 ;; *) DL_ARCH=$ARCH; AWS_ARCH=$ARCH ;; esac && \
    # Helm
    HELM_VERSION=$(curl -s https://api.github.com/repos/helm/helm/releases/latest | jq -r '.tag_name') && \
    curl -fsSL "https://get.helm.sh/helm-${HELM_VERSION}-linux-${DL_ARCH}.tar.gz" | tar -xzf - -C /tmp && \
    mv /tmp/linux-${DL_ARCH}/helm /usr/local/bin/helm && chmod +x /usr/local/bin/helm && \
    # Argo CD CLI
    curl -fsSL -o /usr/local/bin/argocd "https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-${DL_ARCH}" && \
    chmod +x /usr/local/bin/argocd && \
    # Argo Workflows CLI
    curl -fsSL "https://github.com/argoproj/argo-workflows/releases/latest/download/argo-linux-${DL_ARCH}.gz" | gunzip > /usr/local/bin/argo && \
    chmod +x /usr/local/bin/argo && \
    # talosctl
    curl -fsSL -o /usr/local/bin/talosctl "https://github.com/siderolabs/talos/releases/latest/download/talosctl-linux-${DL_ARCH}" && \
    chmod +x /usr/local/bin/talosctl && \
    # yq
    curl -fsSL -o /usr/local/bin/yq "https://github.com/mikefarah/yq/releases/latest/download/yq_linux_${DL_ARCH}" && \
    chmod +x /usr/local/bin/yq && \
    # stern
    STERN_VERSION=$(curl -s https://api.github.com/repos/stern/stern/releases/latest | jq -r '.tag_name' | sed 's/^v//') && \
    curl -fsSL "https://github.com/stern/stern/releases/latest/download/stern_${STERN_VERSION}_linux_${DL_ARCH}.tar.gz" | tar -xzf - -C /usr/local/bin stern && \
    # Docker Compose
    COMPOSE_VERSION=$(curl -s https://api.github.com/repos/docker/compose/releases/latest | jq -r '.tag_name') && \
    curl -fsSL -o /usr/local/bin/docker-compose "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-linux-${AWS_ARCH}" && \
    chmod +x /usr/local/bin/docker-compose && \
    # k9s
    curl -fsSL "https://github.com/derailed/k9s/releases/latest/download/k9s_Linux_${DL_ARCH}.tar.gz" | tar -xzf - -C /usr/local/bin k9s && \
    # kustomize
    curl -fsSL https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh | bash && \
    mv ./kustomize /usr/local/bin/kustomize && \
    # cloudflared
    curl -fsSL -o /usr/local/bin/cloudflared "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-${DL_ARCH}" && \
    chmod +x /usr/local/bin/cloudflared && \
    # Cleanup
    rm -rf /tmp/linux-* /tmp/*.tar.gz

# GitHub CLI (requires apt, separate layer)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=${ARCH} signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get -o Acquire::ForceIPv4=true update && apt-get -o Acquire::ForceIPv4=true install -y gh && \
    rm -rf /var/lib/apt/lists/* && \
    gh extension install github/gh-copilot 2>/dev/null || true && \
    gh extension install github/gh-codeql 2>/dev/null || true && \
    gh extension install https://github.com/nektos/gh-act 2>/dev/null || true

# GitLab CLI (glab) for dual SCM support
RUN GLAB_VERSION="1.91.0" && \
    ARCH=$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/') && \
    curl -sLo /tmp/glab.tar.gz "https://gitlab.com/gitlab-org/cli/-/releases/v${GLAB_VERSION}/downloads/glab_${GLAB_VERSION}_linux_${ARCH}.tar.gz" && \
    tar -xzf /tmp/glab.tar.gz -C /tmp && \
    mv /tmp/bin/glab /usr/local/bin/glab && \
    chmod +x /usr/local/bin/glab && \
    rm -rf /tmp/glab* /tmp/bin

# nektos/act — run GitHub Actions locally (uses Docker; docker.io is installed above)
# https://github.com/nektos/act
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    set -eux && \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
      amd64) ACT_ASSET=act_Linux_x86_64.tar.gz ;; \
      arm64) ACT_ASSET=act_Linux_arm64.tar.gz ;; \
      *) echo "act: unsupported arch: $ARCH" >&2; exit 1 ;; \
    esac && \
    ACT_VERSION=$(curl -s https://api.github.com/repos/nektos/act/releases/latest | jq -r '.tag_name') && \
    curl -fsSL "https://github.com/nektos/act/releases/download/${ACT_VERSION}/${ACT_ASSET}" | tar -xzf - -C /usr/local/bin act && \
    chmod +x /usr/local/bin/act && \
    act --version

# AWS CLI v2
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) AWS_ARCH=x86_64 ;; arm64) AWS_ARCH=aarch64 ;; *) AWS_ARCH=$ARCH ;; esac && \
    curl -fsSL -o /tmp/awscli.zip "https://awscli.amazonaws.com/awscli-exe-linux-${AWS_ARCH}.zip" && \
    unzip -q /tmp/awscli.zip -d /tmp && \
    /tmp/aws/install && \
    rm -rf /tmp/aws /tmp/awscli.zip

# Install kubectx/kubens from upstream (not available via default apt)
RUN git clone --depth=1 https://github.com/ahmetb/kubectx /opt/kubectx \
  && ln -sf /opt/kubectx/kubectx /usr/local/bin/kubectx \
  && ln -sf /opt/kubectx/kubens /usr/local/bin/kubens

# Install Python development tools
RUN --mount=type=cache,target=/root/.cache/pip \
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
        pip-tools || true

# Install Node.js development tools
RUN --mount=type=cache,target=/root/.cache \
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
        shadcn@5.3.0 \
        wrangler \
        @xdevplatform/xurl || true

# Install Expo / React Native tooling (Tap mobile agent)
# EAS CLI is essential for cloud builds, submissions, and OTA updates
RUN --mount=type=cache,target=/root/.cache \
    npm install -g --no-fund \
        eas-cli@latest \
        expo-cli@latest || true

# Install Morph MCP server for fast code editing (10,500 tok/s) and intelligent search
# Provides edit_file and warp_grep tools for all CLI agents
# Note: ripgrep is already installed above (required for warp_grep)
RUN npm install -g @morphllm/morphmcp

# Install Effect Solutions CLI for AI agent-optimized Effect.ts patterns
# Provides curated best practices queryable via: effect-solutions show <topic>
RUN npm install -g effect-solutions@latest

# =============================================================================
# steipete CLI tools (https://github.com/steipete) - AI/agent workflow utilities
# =============================================================================
# - oracle: Bundle prompts + files for second-model review (GPT-5 Pro, Claude, Gemini)
# - summarize: Point at URL/file/media, get the gist (streaming, YouTube, podcasts)
# - mcporter: Call MCPs via TypeScript or package as CLI; discover/list/call tools
# - poltergeist: Universal hot-reload & file watcher for builds
# - markdansi: Render Markdown with ANSI colors/links in terminal output
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --no-fund \
      @steipete/oracle \
      @steipete/summarize \
      mcporter \
      @steipete/poltergeist \
      markdansi \
    && command -v oracle && command -v summarize && command -v mcporter \
    && command -v poltergeist && command -v markdansi \
    && echo "steipete npm tools OK"

# gogcli (gog) - Google CLI: Gmail, Calendar, Drive, Contacts, Docs, Sheets, etc.
# goplaces  - Google Places API client + CLI
# tmuxwatch - TUI to watch tmux sessions (useful for agent session monitoring)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    set -eux && \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    # --- gogcli ---
    GOG_VERSION=$(curl -s https://api.github.com/repos/steipete/gogcli/releases/latest | jq -r '.tag_name') && \
    mkdir -p /tmp/gog_extract && \
    curl -fsSL "https://github.com/steipete/gogcli/releases/download/${GOG_VERSION}/gogcli_${GOG_VERSION#v}_linux_${DL_ARCH}.tar.gz" | tar -xzf - -C /tmp/gog_extract && \
    install -m 755 /tmp/gog_extract/gog /usr/local/bin/gog && \
    rm -rf /tmp/gog_extract && \
    gog --version && \
    # --- goplaces ---
    GP_VERSION=$(curl -s https://api.github.com/repos/steipete/goplaces/releases/latest | jq -r '.tag_name') && \
    mkdir -p /tmp/gp_extract && \
    curl -fsSL "https://github.com/steipete/goplaces/releases/download/${GP_VERSION}/goplaces_${GP_VERSION#v}_linux_${DL_ARCH}.tar.gz" | tar -xzf - -C /tmp/gp_extract && \
    install -m 755 /tmp/gp_extract/goplaces /usr/local/bin/goplaces && \
    rm -rf /tmp/gp_extract && \
    goplaces --version && \
    # --- tmuxwatch ---
    TW_VERSION=$(curl -s https://api.github.com/repos/steipete/tmuxwatch/releases/latest | jq -r '.tag_name') && \
    mkdir -p /tmp/tw_extract && \
    curl -fsSL "https://github.com/steipete/tmuxwatch/releases/download/${TW_VERSION}/tmuxwatch_${TW_VERSION#v}_linux_${DL_ARCH}.tar.gz" | tar -xzf - -C /tmp/tw_extract && \
    install -m 755 /tmp/tw_extract/tmuxwatch /usr/local/bin/tmuxwatch && \
    rm -rf /tmp/tw_extract && \
    tmuxwatch --version

# Install Rust toolchain for root
RUN --mount=type=cache,target=/root/.cache \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer
ENV PATH="/root/.cargo/bin:${PATH}"

# Optional: sccache for root as well (helpful for scripts running as root)
ARG INSTALL_SCCACHE=false
RUN --mount=type=cache,target=/root/.cache \
  if [ "${INSTALL_SCCACHE:-false}" = "true" ]; then \
    . "$HOME/.cargo/env" && cargo install sccache; \
  else \
    echo "Skipping sccache install"; \
  fi

# Configure mold defaults but disable problematic sccache by default
ENV CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold"
# Note: RUSTC_WRAPPER intentionally not set to avoid sccache path issues

# Prepare npm global and permissions
RUN mkdir -p /usr/local/share/npm-global/lib && chown -R node:node /usr/local/share/npm-global

# Persist bash history for convenience
ARG USERNAME=node
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Install CTO binaries from pre-downloaded files in build context
# The workflow downloads these from GitHub releases before building
# This avoids authentication issues with private repos during Docker build
COPY .docker-binaries/ /tmp/binaries/
ARG TOOLS_VERSION=skip
RUN set -e && \
    # tools client (MCP bridge for agent pods)
    if [ "${TOOLS_VERSION:-skip}" != "skip" ] && [ -n "${TOOLS_VERSION:-}" ] && [ -f /tmp/binaries/tools-client-linux-x86_64 ]; then \
      mv /tmp/binaries/tools-client-linux-x86_64 /usr/local/bin/tools && chmod +x /usr/local/bin/tools; \
    fi && \
    rm -rf /tmp/binaries

# Install git-delta
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) \
  && wget -q "https://github.com/dandavison/delta/releases/download/0.18.2/git-delta_0.18.2_${ARCH}.deb" \
  && dpkg -i "git-delta_0.18.2_${ARCH}.deb" \
  && rm -f "git-delta_0.18.2_${ARCH}.deb"

# Switch to non-root and install Rust for node as well
# Remove any stale .cargo directory from cached layers, then install fresh
USER node
RUN rm -rf $HOME/.cargo && mkdir -p $HOME/.cargo $HOME/.cache \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer

# Install cursor-rust-tools MCP server for AI agent Rust analysis
# TEMPORARILY DISABLED - compilation fails in CI (exit code 101)
# RUN --mount=type=cache,target=/home/node/.cache,uid=1000,gid=1000,mode=0777 \
#     . $HOME/.cargo/env \
#   && cargo install --git https://github.com/terhechte/cursor-rust-tools

# Install Qlty for unified code quality across all languages
RUN curl -fsSL https://qlty.sh | bash \
  && echo 'export PATH="$HOME/.qlty/bin:$PATH"' >> ~/.bashrc \
  && echo 'export PATH="$HOME/.qlty/bin:$PATH"' >> ~/.zshrc

# Install security scanning tools for agent-generated code
USER root
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    set -eux; \
    ARCH=$(dpkg --print-architecture); \
    case "$ARCH" in \
        amd64) DL_ARCH=amd64; GITLEAKS_CANDIDATES="x64 amd64"; HADOLINT_CANDIDATES="x86_64 amd64" ;; \
        arm64) DL_ARCH=arm64; GITLEAKS_CANDIDATES="arm64 aarch64"; HADOLINT_CANDIDATES="arm64 aarch64" ;; \
        *) DL_ARCH="$ARCH"; GITLEAKS_CANDIDATES="$ARCH"; HADOLINT_CANDIDATES="$ARCH" ;; \
    esac; \
    # Install gitleaks (secret scanning)
    GITLEAKS_VERSION=$(curl -s https://api.github.com/repos/gitleaks/gitleaks/releases/latest | jq -r '.tag_name'); \
    if [ -z "$GITLEAKS_VERSION" ] || [ "$GITLEAKS_VERSION" = "null" ]; then GITLEAKS_VERSION="v8.18.2"; fi; \
    DOWNLOAD_SUCCESS=0; \
    for candidate in $GITLEAKS_CANDIDATES; do \
        if curl -fsSL -o /tmp/gitleaks.tar.gz "https://github.com/gitleaks/gitleaks/releases/download/${GITLEAKS_VERSION}/gitleaks_${GITLEAKS_VERSION#v}_linux_${candidate}.tar.gz"; then \
            DOWNLOAD_SUCCESS=1; \
            break; \
        else \
            rm -f /tmp/gitleaks.tar.gz; \
        fi; \
    done; \
    if [ "$DOWNLOAD_SUCCESS" -ne 1 ]; then \
        echo "Failed to download gitleaks for architecture $ARCH"; \
        exit 1; \
    fi; \
    tar -xzf /tmp/gitleaks.tar.gz -C /usr/local/bin gitleaks; \
    chmod +x /usr/local/bin/gitleaks; \
    rm -f /tmp/gitleaks.tar.gz; \
    # Install trivy (vulnerability scanning)
    curl -fsSL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin; \
    # Install hadolint (Dockerfile linting)
    DOWNLOAD_SUCCESS=0; \
    for candidate in $HADOLINT_CANDIDATES; do \
        if curl -fsSL -o /usr/local/bin/hadolint "https://github.com/hadolint/hadolint/releases/latest/download/hadolint-Linux-${candidate}"; then \
            DOWNLOAD_SUCCESS=1; \
            break; \
        fi; \
    done; \
    if [ "$DOWNLOAD_SUCCESS" -ne 1 ]; then \
        echo "Failed to download hadolint for architecture $ARCH"; \
        exit 1; \
    fi; \
    chmod +x /usr/local/bin/hadolint

# Install Semgrep for comprehensive static analysis
# Install rich separately with --ignore-installed to avoid conflicts with system package
RUN --mount=type=cache,target=/root/.cache/pip \
    pip3 install --break-system-packages --no-cache-dir --ignore-installed rich && \
    pip3 install --break-system-packages --no-cache-dir semgrep

# Configure npm globals before installing CLI tooling as non-root
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin:/home/node/.cargo/bin

# Install mermaid-cli for diagram generation (docs/architecture work)
USER node
RUN sudo npm install -g @mermaid-js/mermaid-cli && \
    sudo chown -R node:node /usr/local/share/npm-global

# Install Playwright for frontend E2E testing (Blaze agent)
USER root
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get -o Acquire::ForceIPv4=true update && \
    npm install -g @playwright/test && \
    npx playwright install --with-deps chromium && \
    chown -R node:node /usr/local/share/npm-global

# Install agent-browser CLI for AI agent browser automation
# Uses Playwright's Chromium installation to avoid duplicate browser downloads
RUN npm install -g agent-browser@0.9.1 && \
    chown -R node:node /usr/local/share/npm-global && \
    # Create profile script to set Chromium path for agent-browser at runtime
    echo 'export AGENT_BROWSER_EXECUTABLE_PATH="$(find /root/.cache/ms-playwright -name chrome -type f 2>/dev/null | head -1)"' > /etc/profile.d/agent-browser.sh && \
    chmod +x /etc/profile.d/agent-browser.sh
USER node

# Go environment for non-root user
ENV GOPATH=/home/node/go
ENV PATH=$PATH:$GOPATH/bin:$GOROOT/bin
RUN mkdir -p "$GOPATH" && mkdir -p "$GOPATH/bin"

# Zsh and prompt setup
ENV SHELL=/bin/zsh
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v1.2.0/zsh-in-docker.sh)" -- \
  -p git \
  -p fzf \
  -a "source /usr/share/doc/fzf/examples/key-bindings.zsh" \
  -a "source /usr/share/doc/fzf/examples/completion.zsh" \
  -a "export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" -x

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
    git config --global push.autoSetupRemote true

# Setup Python virtual environment template
RUN python3 -m venv ~/.venv/base && \
    ~/.venv/base/bin/pip install --upgrade pip setuptools wheel

# Create development helper scripts
RUN echo '#!/bin/bash\n# Development helper script\nset -e\n\nif [ $# -eq 0 ]; then\n    echo "Usage: $0 <command>"\n    echo "Available commands: setup-py, setup-node, setup-go, clean"\n    exit 1\nfi\n\ncase "$1" in\n    setup-py)\n        if [ ! -d "venv" ]; then\n            python3 -m venv venv\n        fi\n        venv/bin/pip install --upgrade pip\n        ;;\n    setup-node)\n        if [ ! -d "node_modules" ]; then\n            npm install\n        fi\n        ;;\n    setup-go)\n        if [ ! -f "go.mod" ]; then\n            go mod init $(basename $(pwd))\n        fi\n        ;;\n    clean)\n        rm -rf venv node_modules target dist build *.log\n        ;;\n    *)\n        echo "Unknown command: $1"\n        exit 1\n        ;;\nesac' > ~/dev-helper.sh && \
    chmod +x ~/dev-helper.sh

# Add helper script to PATH
ENV PATH="/home/node:${PATH}"

# =============================================================================
# 1Password CLI (op) for headless secret access from agents
# Authenticated via OP_SERVICE_ACCOUNT_TOKEN env var (injected as K8s secret)
# Docs: https://developer.1password.com/docs/service-accounts/use-with-1password-cli
# =============================================================================
COPY --from=1password/op:2 /usr/local/bin/op /usr/local/bin/op

# Labels / defaults
USER node
WORKDIR /workspace
ENV DEVCONTAINER=true

# Rust dev tooling for node user (install via cargo)
# Only cache the registry for faster rebuilds - bin directory is written to image
# MINIMAL SET: Only essential tools to reduce build time
# - sccache: compilation cache
# - cargo-nextest: fast test runner
# - cargo-watch: file watcher for development
# Removed slow-to-compile tools that can be installed on-demand:
# - cargo-outdated (requires full cargo crate)
# - cargo-deny (heavy deps: gix, rustls)
# - cargo-audit (can use cargo-deny instead)
# - cargo-edit (use `cargo add` in recent Rust)
# - cargo-geiger, cargo-expand, cargo-udeps (rarely used)
USER node
RUN --mount=type=cache,target=/home/node/.cargo/registry,id=node-cargo-registry-v3,uid=1000,gid=1000,mode=0777 \
  . $HOME/.cargo/env && \
  mkdir -p "$HOME/.cache/sccache" 2>/dev/null || true && \
  cargo --version && \
  cargo install \
    sccache \
    cargo-nextest \
    cargo-watch || true

# Configure Rust build environment (without problematic sccache wrapper)
ENV CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold" \
    CARGO_TERM_COLOR=always
# Note: sccache available at /home/node/.cargo/bin/sccache but not set as RUSTC_WRAPPER
# to avoid path conflicts when containers run as different users

# Configure Rust environment for all users (consolidated)
USER root
RUN rustup default stable 2>/dev/null || true && \
    # Ensure rust default script
    echo '#!/bin/sh\nrustup default stable 2>/dev/null || true' > /usr/local/bin/ensure-rust-default && \
    chmod +x /usr/local/bin/ensure-rust-default && \
    # Shell profile integration
    printf '%s\n' '# Add Cargo bin to PATH for interactive shells' 'if [ -d "$HOME/.cargo/bin" ]; then PATH="$HOME/.cargo/bin:$PATH"; fi' 'export PATH' > /etc/profile.d/cargo.sh && \
    chmod 0644 /etc/profile.d/cargo.sh && \
    # Global Rust environment setup script
    echo '#!/bin/sh\n# Ensure Rust environment is always available\nif [ -f "$HOME/.cargo/env" ]; then\n    . "$HOME/.cargo/env"\nfi\nif [ -f "/root/.cargo/env" ]; then\n    . "/root/.cargo/env"\nfi\nrustup default stable 2>/dev/null || true' > /usr/local/bin/setup-rust-env && \
    chmod +x /usr/local/bin/setup-rust-env
USER node
RUN rustup default stable 2>/dev/null || true


# =============================================================================
# LOCAL BUILD TARGET (compiles intake/pm-activity from source)
# =============================================================================
FROM production AS local

# Copy locally-built binaries from builder stage
COPY --from=intake-builder /build/target/release/intake /usr/local/bin/intake
COPY --from=intake-builder /build/target/release/pm-activity /usr/local/bin/pm-activity
RUN chmod +x /usr/local/bin/intake /usr/local/bin/pm-activity && \
    (intake --version 2>&1 || intake --help 2>&1 | head -5) && \
    (pm-activity --help 2>&1 | head -5)
