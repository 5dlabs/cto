# Rust builder image for fast CI builds
# Based on official GitHub Actions Runner Controller (ARC) image
FROM ghcr.io/actions/actions-runner:2.328.0

# Switch to root for installations
USER root

# Install system dependencies and repositories in a single layer for better caching
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    # Update and install basic tools first
    apt-get update && apt-get install -y \
    curl \
    wget \
    gnupg \
    software-properties-common \
    ca-certificates \
    lsb-release \
    && \
    # Add LLVM repository for modern clang/lld
    curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /usr/share/keyrings/llvm-archive-keyring.gpg \
    && ARCH=$(dpkg --print-architecture) \
    && echo "deb [arch=${ARCH} signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] http://apt.llvm.org/$(lsb_release -cs)/ llvm-toolchain-$(lsb_release -cs)-15 main" | tee /etc/apt/sources.list.d/llvm.list > /dev/null \
    && \
    # Add Docker's official GPG key and repository
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && ARCH=$(dpkg --print-architecture) \
    && echo "deb [arch=${ARCH} signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && \
    # Update package lists and install all packages
    apt-get update && apt-get install -y \
    # Build essentials (some may already be installed)
    build-essential \
    pkg-config \
    libssl-dev \
    # Modern tools
    git \
    unzip \
    jq \
    file \
    # Docker Engine for container builds
    docker-ce \
    docker-ce-cli \
    containerd.io \
    docker-buildx-plugin \
    docker-compose-plugin \
    # Clang for mold linker (from LLVM repo)
    clang-15 \
    lld-15 \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/clang-15 /usr/bin/clang \
    && ln -sf /usr/bin/clang++-15 /usr/bin/clang++

# Install Node.js + npm (v20) for npm-based tooling used in CI
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && npm -v && node -v \
    && corepack enable || true

# Install mold linker (fastest linker for Rust)
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    wget -qO mold.tar.gz https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-x86_64-linux.tar.gz && \
    tar xz -f mold.tar.gz && \
    cp mold-2.4.0-x86_64-linux/bin/mold /usr/local/bin/ && \
    chmod +x /usr/local/bin/mold && \
    # Create wrapper script for clang to use mold
    echo '#!/bin/sh' > /usr/local/bin/clang-mold && \
    echo 'exec clang -fuse-ld=/usr/local/bin/mold "$@"' >> /usr/local/bin/clang-mold && \
    chmod +x /usr/local/bin/clang-mold

# Install kubectl (with caching) - robust download with retries
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    # Get latest stable version with retries \
    KUBECTL_VERSION=$(curl --retry 3 --retry-delay 1 -L -s https://dl.k8s.io/release/stable.txt) && \
    # Download kubectl with robust options \
    curl --retry 5 --retry-delay 2 --retry-connrefused \
         --tlsv1.2 --proto '=https' --location --fail --silent --show-error \
         -O "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install Helm (with caching) - copy secure approach from runtime image
RUN --mount=type=cache,target=/tmp/downloads \
    cd /tmp/downloads && \
    ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in amd64) DL_ARCH=amd64 ;; arm64) DL_ARCH=arm64 ;; *) DL_ARCH=$ARCH ;; esac && \
    HELM_VERSION=$(curl -s https://api.github.com/repos/helm/helm/releases/latest | jq -r '.tag_name') && \
    HELM_TAR="helm-${HELM_VERSION}-linux-${DL_ARCH}.tar.gz" && \
    for i in 1 2 3; do \
        curl -fsSL -o "${HELM_TAR}" "https://get.helm.sh/${HELM_TAR}" && \
        tar -xzf "${HELM_TAR}" && \
        mv "linux-${DL_ARCH}/helm" /usr/local/bin/helm && \
        chmod +x /usr/local/bin/helm && \
        rm -f "${HELM_TAR}" && \
        helm version --client && break || \
        (echo "Attempt $i failed, retrying..." && sleep 5); \
    done

# Install GitHub CLI (separate layer for better caching)
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Switch to runner user (already exists in summerwind image)
USER runner
WORKDIR /home/runner

# Install Rust as runner user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    echo 'source ~/.cargo/env' >> /home/runner/.bashrc

# Add Rust to PATH
ENV PATH="/home/runner/.cargo/bin:${PATH}"

# Install essential Rust tools only (no pre-warming)
RUN /home/runner/.cargo/bin/rustup component add clippy rustfmt

# Install sccache with robust error handling
RUN mkdir -p /home/runner/.cargo/bin && \
    { \
        cd /tmp && \
        ARCH=$(dpkg --print-architecture) && \
        case "$ARCH" in \
            amd64) SCCACHE_ARCH=x86_64 ;; \
            arm64) SCCACHE_ARCH=aarch64 ;; \
            *) SCCACHE_ARCH=$ARCH ;; \
        esac && \
        SCCACHE_VERSION="0.8.2" && \
        SCCACHE_TARGET="${SCCACHE_ARCH}-unknown-linux-musl" && \
        echo "Attempting to download sccache v${SCCACHE_VERSION}..." && \
        curl -fsSL "https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/sccache-v${SCCACHE_VERSION}-${SCCACHE_TARGET}.tar.gz" | \
        tar xz -C /home/runner/.cargo/bin --strip-components=1 "sccache-v${SCCACHE_VERSION}-${SCCACHE_TARGET}/sccache" && \
        chmod +x /home/runner/.cargo/bin/sccache && \
        /home/runner/.cargo/bin/sccache --version && \
        echo "✅ sccache installed from binary"; \
    } || { \
        echo "Binary install failed, falling back to cargo install..." && \
        . /home/runner/.cargo/env && \
        cargo install sccache --locked && \
        echo "✅ sccache installed via cargo"; \
    }

# Install cargo-nextest with robust error handling
RUN mkdir -p /home/runner/.cargo/bin && \
    { \
        cd /tmp && \
        ARCH=$(dpkg --print-architecture) && \
        case "$ARCH" in \
            amd64) NEXTEST_ARCH=x86_64 ;; \
            arm64) NEXTEST_ARCH=aarch64 ;; \
            *) NEXTEST_ARCH=$ARCH ;; \
        esac && \
        NEXTEST_VERSION="0.9.82" && \
        NEXTEST_TARGET="${NEXTEST_ARCH}-unknown-linux-gnu" && \
        echo "Attempting to download cargo-nextest v${NEXTEST_VERSION}..." && \
        curl -fsSL "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${NEXTEST_VERSION}/cargo-nextest-${NEXTEST_VERSION}-${NEXTEST_TARGET}.tar.gz" | \
        tar xz -C /home/runner/.cargo/bin && \
        chmod +x /home/runner/.cargo/bin/cargo-nextest && \
        /home/runner/.cargo/bin/cargo-nextest --version && \
        echo "✅ cargo-nextest installed from binary"; \
    } || { \
        echo "Binary install failed, falling back to cargo install..." && \
        . /home/runner/.cargo/env && \
        cargo install cargo-nextest --locked && \
        echo "✅ cargo-nextest installed via cargo"; \
    }

# Set up optimized Cargo config (stay as runner user)
RUN mkdir -p /home/runner/.cargo && cat > /home/runner/.cargo/config.toml << 'EOF'
[build]
rustc-wrapper = "sccache"
jobs = 16  # Use all cores

[target.x86_64-unknown-linux-gnu]
# Use mold linker for faster linking
linker = "/usr/local/bin/clang-mold"
rustflags = [
    "-C", "link-arg=-Wl,--threads=16",  # Parallel linking with mold
    "-C", "target-cpu=x86-64-v2",  # Modern baseline CPU
]

# Fast release profile
[profile.release]
codegen-units = 16  # Good parallelism
lto = false  # Disable LTO for faster builds
debug = false
strip = true
opt-level = 2  # Good balance of speed and performance

# Fast dev builds
[profile.dev]
debug = 0
strip = "debuginfo"
incremental = true
codegen-units = 16

[registries.crates-io]
protocol = "sparse"

[net]
git-fetch-with-cli = true
jobs = 16
EOF

# Add runner to docker group and set ownership (switch to root temporarily)
USER root
RUN usermod -aG docker runner \
    && chown -R runner:runner /home/runner

# Environment variables for PVC-based caching
# These will be overridden by the workflow to point to PVC mounts
ENV CARGO_HOME="/cache/cargo"
ENV RUSTUP_HOME="/cache/rustup"
ENV SCCACHE_DIR="/cache/sccache"
ENV SCCACHE_CACHE_SIZE="2G"
ENV RUSTC_WRAPPER="sccache"

# Switch back to runner user (required for ARC)
USER runner

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD which rustc && which cargo && which sccache && which docker || exit 1

# Keep the default entrypoint from the base ARC image
# Do not override ENTRYPOINT or CMD - let ARC handle runner lifecycle