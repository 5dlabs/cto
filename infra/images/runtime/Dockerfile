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
    apt-get update && apt-get install -y --no-install-recommends \
  git procps sudo fzf zsh man-db unzip gnupg2 gh iproute2 dnsutils jq \
  netcat-openbsd ca-certificates curl wget docker.io postgresql-client \
  redis-tools tree build-essential binutils pkg-config libssl-dev lldb lld clang mold \
  libpq-dev \
  python3 python3-venv python3-pip python-is-python3 \
  strace lsof htop psmisc ripgrep fd-find bat httpie openssl \
  && rm -rf /var/lib/apt/lists/* \
  && update-alternatives --install /usr/local/bin/fd fd /usr/bin/fdfind 10 \
  && update-alternatives --install /usr/local/bin/bat bat /usr/bin/batcat 10

# Create non-root user 'node' with stable UID/GID 1000 to match cache mounts (robust if GID/UID exist)
RUN if ! id -u node >/dev/null 2>&1; then \
      if getent group 1000 >/dev/null; then \
        GN=$(getent group 1000 | cut -d: -f1); \
      else \
        groupadd -g 1000 node; GN=node; \
      fi; \
      if getent passwd 1000 >/dev/null; then \
        useradd --shell /bin/bash --create-home -g "$GN" node; \
      else \
        useradd --shell /bin/bash --create-home -u 1000 -g "$GN" node; \
      fi; \
    fi

# Passwordless sudo for the non-root user (node)
RUN echo 'node ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/99-node-nopasswd \
  && chmod 0440 /etc/sudoers.d/99-node-nopasswd

# Install Node.js directly (not via nvm) for better compatibility
RUN --mount=type=cache,target=/root/.cache \
  curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
  && apt-get install -y nodejs \
  && npm install -g npm@latest \
  && rm -rf /var/lib/apt/lists/*

# Enable corepack (yarn/pnpm) for Node.js (guard if missing)
RUN if command -v corepack >/dev/null 2>&1; then corepack enable; fi

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
  (tar -C /usr/local -xzf /tmp/go.tgz || (apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends golang-go && ln -sfn /usr/lib/go /usr/local/go)); \
  rm -f /tmp/go.tgz
ENV GOROOT=/usr/local/go
ENV PATH=$GOROOT/bin:/usr/lib/go/bin:$PATH

# Install kubectl via apt repository (more reliable than direct download)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.31/deb/Release.key | gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg \
  && echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.31/deb/ /' | tee /etc/apt/sources.list.d/kubernetes.list \
  && apt-get update && apt-get install -y kubectl && rm -rf /var/lib/apt/lists/*

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
        argo version --short --client && break || \
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

# Install kubectx/kubens from upstream (not available via default apt)
RUN git clone --depth=1 https://github.com/ahmetb/kubectx /opt/kubectx \
  && ln -sf /opt/kubectx/kubectx /usr/local/bin/kubectx \
  && ln -sf /opt/kubectx/kubens /usr/local/bin/kubens

# Install Rust toolchain for root
RUN --mount=type=cache,target=/root/.cache \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer
ENV PATH="/root/.cargo/bin:${PATH}"

# Optional: sccache for root as well (helpful for scripts running as root)
RUN --mount=type=cache,target=/root/.cache \
  . $HOME/.cargo/env && cargo install sccache || true

# Configure mold and sccache defaults (will also apply to node later)
ENV CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Prepare npm global and permissions
RUN mkdir -p /usr/local/share/npm-global && chown -R 1000:1000 /usr/local/share

# Persist bash history for convenience
ARG USERNAME=node
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Install toolman client
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/5dlabs/toolman/releases/download/v2.4.4/toolman-installer.sh | \
    TOOLMAN_NO_MODIFY_PATH=1 sh \
  && mv ~/.cargo/bin/toolman-client /usr/local/bin/toolman

# Install TaskMaster globally (latest version)
RUN npm install -g task-master-ai@latest

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

# Global npm config and PATH
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin:/home/node/.cargo/bin

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
    cargo-udeps || true

# Prefer sccache and mold for faster Rust builds
ENV RUSTC_WRAPPER=/home/node/.cargo/bin/sccache \
    SCCACHE_DIR=/home/node/.cache/sccache \
    SCCACHE_CACHE_SIZE=10G \
    CC=clang \
    CXX=clang++ \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold" \
    CARGO_TERM_COLOR=always


