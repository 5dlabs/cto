# syntax=docker/dockerfile:1.7-labs
## Optimized runtime base with BuildKit caching for faster CI builds
FROM ubuntu:24.04

# Use bash for complex RUN steps
SHELL ["/bin/bash", "-lc"]

ARG TZ
ENV TZ="$TZ"

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
  sd \
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

# Install Node.js directly (not via nvm) for better compatibility
RUN --mount=type=cache,target=/root/.cache \
  curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
  && apt-get -o Acquire::ForceIPv4=true install -y nodejs \
  && npm install -g npm@latest \
  && rm -rf /var/lib/apt/lists/*

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

# Install Helm via direct download (script was failing on arm64)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    HELM_VERSION=$(curl -s https://api.github.com/repos/helm/helm/releases/latest | jq -r '.tag_name') && \
    HELM_TAR="helm-${HELM_VERSION}-linux-${DL_ARCH}.tar.gz" && \
    for i in 1 2 3; do \
        curl -fsSL -o "/tmp/${HELM_TAR}" "https://get.helm.sh/${HELM_TAR}" && \
        tar -C /tmp -xzf "/tmp/${HELM_TAR}" && \
        mv "/tmp/linux-${DL_ARCH}/helm" /usr/local/bin/helm && \
        chmod +x /usr/local/bin/helm && \
        rm -f "/tmp/${HELM_TAR}" "/tmp/linux-${DL_ARCH}" && \
        helm version && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Set architecture variables
ENV DL_ARCH_AMD64=amd64
ENV DL_ARCH_ARM64=arm64

# Install Argo CD CLI
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o /usr/local/bin/argocd "https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/argocd && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install Argo Workflows CLI
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o "/tmp/argo-linux-${DL_ARCH}.gz" "https://github.com/argoproj/argo-workflows/releases/latest/download/argo-linux-${DL_ARCH}.gz" && \
        gunzip -c "/tmp/argo-linux-${DL_ARCH}.gz" > /usr/local/bin/argo && \
        chmod +x /usr/local/bin/argo && \
        rm -f "/tmp/argo-linux-${DL_ARCH}.gz" && \
        argo version --short && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install talosctl
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o /usr/local/bin/talosctl "https://github.com/siderolabs/talos/releases/latest/download/talosctl-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/talosctl && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install yq
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o /usr/local/bin/yq "https://github.com/mikefarah/yq/releases/latest/download/yq_linux_${DL_ARCH}" && \
        chmod +x /usr/local/bin/yq && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install GitHub CLI extensions
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    gh extension install github/gh-copilot 2>/dev/null || true && \
    gh extension install github/gh-codeql 2>/dev/null || true

# Install GitHub CLI
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg && \
        echo "deb [arch=${DL_ARCH} signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
        apt-get -o Acquire::ForceIPv4=true update && apt-get -o Acquire::ForceIPv4=true install -y gh && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install AWS CLI v2
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=x86_64 ;; arm64) DL_ARCH=aarch64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o "/tmp/awscli.zip" "https://awscli.amazonaws.com/awscli-exe-linux-${DL_ARCH}.zip" && \
        unzip -q /tmp/awscli.zip -d /tmp && \
        /tmp/aws/install && \
        rm -rf /tmp/aws /tmp/awscli.zip && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install stern
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    STERN_TAR="stern_linux_${DL_ARCH}.tar.gz" && \
    for i in 1 2 3; do \
        curl -fsSL -o "/tmp/${STERN_TAR}" "https://github.com/stern/stern/releases/latest/download/${STERN_TAR}" && \
        tar -C /usr/local/bin -xzf "/tmp/${STERN_TAR}" stern && \
        rm -f "/tmp/${STERN_TAR}" && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install Docker Compose v2
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    COMPOSE_VERSION=$(curl -s https://api.github.com/repos/docker/compose/releases/latest | jq -r '.tag_name') && \
    for i in 1 2 3; do \
        curl -fsSL -o /usr/local/bin/docker-compose "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/docker-compose && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install k9s
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    K9S_TAR="k9s_Linux_${DL_ARCH}.tar.gz" && \
    for i in 1 2 3; do \
        curl -fsSL -o "/tmp/${K9S_TAR}" "https://github.com/derailed/k9s/releases/latest/download/${K9S_TAR}" && \
        tar -C /usr/local/bin -xzf "/tmp/${K9S_TAR}" k9s && \
        rm -f "/tmp/${K9S_TAR}" && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install kustomize
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    for i in 1 2 3; do \
        curl -fsSL https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh | bash && \
        mv ./kustomize /usr/local/bin/kustomize && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install cloudflared (Cloudflare Tunnel CLI)
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    for i in 1 2 3; do \
        curl -fsSL -o /usr/local/bin/cloudflared "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/cloudflared && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

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
        shadcn@latest \
        wrangler || true

# Install Morph MCP server for fast code editing (10,500 tok/s) and intelligent search
# Provides edit_file and warp_grep tools for all CLI agents
# Note: ripgrep is already installed above (required for warp_grep)
RUN npm install -g @morphllm/morphmcp

# Install Effect Solutions CLI for AI agent-optimized Effect.ts patterns
# Provides curated best practices queryable via: effect-solutions show <topic>
RUN npm install -g effect-solutions@latest

# Install Rust toolchain for root
RUN --mount=type=cache,target=/root/.cache \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer
ENV PATH="/root/.cargo/bin:${PATH}"

# Optional: sccache for root as well (helpful for scripts running as root)
RUN --mount=type=cache,target=/root/.cache \
  . $HOME/.cargo/env && cargo install sccache || true

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

# Install tools client from CTO monorepo unified release (skip if version is "skip" or empty)
# Downloads binary directly from v*.*.* unified releases
ARG TOOLS_VERSION=skip
RUN if [ "$TOOLS_VERSION" != "skip" ] && [ -n "$TOOLS_VERSION" ]; then \
      ARCH=$(dpkg --print-architecture) && \
      case "$ARCH" in amd64) DL_ARCH=x86_64 ;; arm64) DL_ARCH=aarch64 ;; *) DL_ARCH="" ;; esac && \
      if [ -n "$DL_ARCH" ]; then \
        echo "📦 Installing tools client v${TOOLS_VERSION} (${DL_ARCH})..." && \
        curl --proto '=https' --tlsv1.2 -LsSf -o /usr/local/bin/tools \
          "https://github.com/5dlabs/cto/releases/download/v${TOOLS_VERSION}/tools-client-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/tools && \
        echo "✅ tools client installed"; \
      else \
        echo "⏭️ Skipping tools client installation (unsupported architecture: ${ARCH})"; \
      fi; \
    else \
      echo "⏭️ Skipping tools client installation (no release available)"; \
    fi

# Install play-monitor from CTO monorepo unified release (skip if version is "skip" or empty)
# Downloads binary directly from v*.*.* unified releases
ARG PLAY_MONITOR_VERSION=skip
RUN if [ "$PLAY_MONITOR_VERSION" != "skip" ] && [ -n "$PLAY_MONITOR_VERSION" ]; then \
      ARCH=$(dpkg --print-architecture) && \
      case "$ARCH" in amd64) DL_ARCH=x86_64 ;; arm64) DL_ARCH=aarch64 ;; *) DL_ARCH="" ;; esac && \
      if [ -n "$DL_ARCH" ]; then \
        echo "📦 Installing play-monitor v${PLAY_MONITOR_VERSION} (${DL_ARCH})..." && \
        curl --proto '=https' --tlsv1.2 -LsSf -o /usr/local/bin/play-monitor \
          "https://github.com/5dlabs/cto/releases/download/v${PLAY_MONITOR_VERSION}/play-monitor-linux-${DL_ARCH}" && \
        chmod +x /usr/local/bin/play-monitor && \
        echo "✅ play-monitor installed"; \
      else \
        echo "⏭️ Skipping play-monitor installation (unsupported architecture: ${ARCH})"; \
      fi; \
    else \
      echo "⏭️ Skipping play-monitor installation (no release available)"; \
    fi

# Install TaskMaster globally (stable release)
RUN npm install -g task-master-ai@0.26.0

# Install git-delta
RUN --mount=type=cache,target=/tmp/downloads,sharing=locked \
    ARCH=$(dpkg --print-architecture) \
  && wget -q "https://github.com/dandavison/delta/releases/download/0.18.2/git-delta_0.18.2_${ARCH}.deb" \
  && dpkg -i "git-delta_0.18.2_${ARCH}.deb" \
  && rm -f "git-delta_0.18.2_${ARCH}.deb"

# Switch to non-root and install Rust for node as well
USER node
RUN --mount=type=cache,target=/home/node/.cache,uid=1000,gid=1000,mode=0777 \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer

# Install cursor-rust-tools MCP server for AI agent Rust analysis
RUN --mount=type=cache,target=/home/node/.cache,uid=1000,gid=1000,mode=0777 \
    . $HOME/.cargo/env \
  && cargo install --git https://github.com/terhechte/cursor-rust-tools

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

# Labels / defaults
USER node
WORKDIR /workspace
ENV DEVCONTAINER=true

# Rust dev tooling for node user (install via cargo) - ensure ownership first
USER root
RUN chown -R 1000:1000 /home/node/.cache /home/node/.cargo || true
USER node
RUN --mount=type=cache,target=/home/node/.cache,uid=1000,gid=1000,mode=0777 \
  set -eux; \
  . $HOME/.cargo/env; \
  mkdir -p "$HOME/.cache/sccache" "$HOME/.cargo/bin"; \
  cargo --version; \
  cargo install \
    sccache \
    cargo-edit \
    cargo-nextest \
    cargo-deny \
    cargo-outdated \
    cargo-watch \
    cargo-expand \
    cargo-udeps \
    cargo-audit \
    cargo-geiger || true

# Configure Rust build environment (without problematic sccache wrapper)
ENV CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold" \
    CARGO_TERM_COLOR=always
# Note: sccache available at /home/node/.cargo/bin/sccache but not set as RUSTC_WRAPPER
# to avoid path conflicts when containers run as different users

# Create global rustup default to work regardless of which user the container runs as
USER root
RUN rustup default stable 2>/dev/null || true
USER node
RUN rustup default stable 2>/dev/null || true
# Final fallback: ensure rustup has a default even if container runs as different user
USER root
RUN echo '#!/bin/sh\nrustup default stable 2>/dev/null || true' > /usr/local/bin/ensure-rust-default \
  && chmod +x /usr/local/bin/ensure-rust-default

# Ensure Rust environment is always available in shell profiles
RUN printf '%s\n' '# Add Cargo bin to PATH for interactive shells' 'if [ -d "$HOME/.cargo/bin" ]; then PATH="$HOME/.cargo/bin:$PATH"; fi' 'export PATH' > /etc/profile.d/cargo.sh \
  && chmod 0644 /etc/profile.d/cargo.sh

# Create a global Rust environment setup script
RUN echo '#!/bin/sh\n# Ensure Rust environment is always available\nif [ -f "$HOME/.cargo/env" ]; then\n    . "$HOME/.cargo/env"\nfi\nif [ -f "/root/.cargo/env" ]; then\n    . "/root/.cargo/env"\nfi\nrustup default stable 2>/dev/null || true' > /usr/local/bin/setup-rust-env \
  && chmod +x /usr/local/bin/setup-rust-env

