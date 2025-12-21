# syntax=docker/dockerfile:1
# =============================================================================
# Tools Server - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binaries in context):
#     docker build -t ghcr.io/5dlabs/tools .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -t ghcr.io/5dlabs/tools:local .
#
# =============================================================================

# =============================================================================
# BUILD STAGES (used only for local development builds)
# =============================================================================
FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /build

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies - THIS IS THE CACHED LAYER!
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release --package tools --bin tools-server && \
    cargo build --release --package cto-mcp --bin cto-mcp

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM ubuntu:24.04 AS runtime-base

# Install system dependencies and all runtimes
# hadolint ignore=DL3008,DL3015
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    bash \
    nodejs \
    npm \
    python3 \
    python3-dev \
    python3-pip \
    golang-go \
    openjdk-17-jre \
    docker.io \
    coreutils \
    findutils \
    grep \
    sed \
    tar \
    gzip \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl (architecture-aware)
# hadolint ignore=DL3059
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN ARCH=$(dpkg --print-architecture) && \
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/${ARCH}/kubectl" && \
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl && \
    rm kubectl

# Install MinIO client (mc) for S3-compatible object storage operations
# hadolint ignore=DL3059
RUN ARCH=$(dpkg --print-architecture) && \
    curl -LO "https://dl.min.io/client/mc/release/linux-${ARCH}/mc" && \
    install -o root -g root -m 0755 mc /usr/local/bin/mc && \
    rm mc

# Create symbolic links for Python
RUN ln -sf python3 /usr/bin/python

# Install UV and uvx for Python package management
# hadolint ignore=DL3059,DL4006,SC1091
RUN curl -LsSf https://astral.sh/uv/install.sh | bash && \
    (mv /root/.local/bin/uv /usr/local/bin/ && mv /root/.local/bin/uvx /usr/local/bin/) 2>/dev/null || \
    (mv /root/.cargo/bin/uv /usr/local/bin/ && mv /root/.cargo/bin/uvx /usr/local/bin/) 2>/dev/null || \
    echo "UV and uvx installed successfully"

# Install minimal Rust toolchain
# hadolint ignore=DL3059,DL4006
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# Create non-root user
# hadolint ignore=DL3059
RUN useradd -m -u 1001 -s /bin/bash mcp

# Create directories for configs and npm cache
RUN mkdir -p /config /home/mcp/.npm && \
    chown -R mcp:mcp /config /home/mcp

# Set up environment for all runtimes
ENV PORT=3000 \
    PROJECT_DIR=/config \
    PATH="/usr/local/go/bin:/root/.cargo/bin:/usr/lib/jvm/java-17-openjdk/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    NODE_ENV=production \
    JAVA_HOME=/usr/lib/jvm/java-17-openjdk \
    DOCKER_HOST=unix:///var/run/docker.sock

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

# Copy binaries from builder
COPY --from=builder /build/target/release/tools-server /usr/local/bin/tools-server
COPY --from=builder /build/target/release/cto-mcp /usr/local/bin/cto-mcp
RUN chmod +x /usr/local/bin/tools-server /usr/local/bin/cto-mcp

# Copy entrypoint script
COPY infra/images/tools/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Create necessary directories
RUN mkdir -p /app/logs /app/tmp /workspace && \
    chown -R mcp:mcp /config /workspace

USER mcp
WORKDIR /config

ENV TOOLS_LOG_LEVEL=info \
    MCP_PORT=3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/health || exit 1

ENTRYPOINT ["/entrypoint.sh"]
CMD ["tools-server", "--port", "3000", "--project-dir", "/config"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binaries from CI)
# =============================================================================
FROM runtime-base AS production

# Copy the prebuilt server binary from CI/CD artifacts
# CI workflow should provide architecture-specific binaries
COPY tools-server /usr/local/bin/tools-server
RUN chmod +x /usr/local/bin/tools-server

# Copy the cto-mcp binary (MCP server for Argo Workflows integration)
COPY cto-mcp /usr/local/bin/cto-mcp
RUN chmod +x /usr/local/bin/cto-mcp

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

USER mcp
WORKDIR /config

EXPOSE 3000

ENTRYPOINT ["/entrypoint.sh"]
CMD ["tools-server", "--port", "3000", "--project-dir", "/config"]

# Default target for CI builds
FROM production
