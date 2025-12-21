# syntax=docker/dockerfile:1
# =============================================================================
# Healer Monitor - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binary in context):
#     cargo build --release --package healer
#     cp target/release/healer healer
#     docker build -f infra/images/healer/Dockerfile -t ghcr.io/5dlabs/healer .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -f infra/images/healer/Dockerfile -t ghcr.io/5dlabs/healer:local .
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
    cargo build --release --package healer

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

# Install runtime dependencies including gh CLI and kubectl for CodeRun operations
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    jq \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
# hadolint ignore=DL3059
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install gh CLI
# hadolint ignore=DL3008,DL3059
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y --no-install-recommends gh && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 1000 -m -s /bin/bash app

WORKDIR /app

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

# Copy built binary to PATH location
COPY --from=builder /build/target/release/healer /usr/local/bin/healer

# Copy healer templates
COPY templates/healer /app/templates/healer

RUN chmod +x /usr/local/bin/healer && chown -R app:app /app

USER 1000

EXPOSE 8080

ENTRYPOINT ["healer"]
CMD ["--help"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

# Copy pre-built healer binary (placed by CI workflow into build context)
COPY healer /usr/local/bin/healer

# Copy templates directory
COPY templates/ /app/templates/

RUN chmod +x /usr/local/bin/healer && chown -R app:app /app

# Verify tools are available
# hadolint ignore=SC2015
RUN echo "Checking installed tools..." && \
    which kubectl && kubectl version --client && \
    which gh && gh --version && \
    (which healer && healer --help | head -5 || true)

USER 1000

EXPOSE 8080

ENTRYPOINT ["healer"]
CMD ["--help"]

# Default target for CI builds
FROM production
