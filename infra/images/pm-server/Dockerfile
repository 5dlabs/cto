# syntax=docker/dockerfile:1
# =============================================================================
# PM Server - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binary in context):
#     cargo build --release --package pm --bin pm-server
#     cp target/release/pm-server pm-server
#     docker build -f infra/images/pm-server/Dockerfile -t ghcr.io/5dlabs/pm-server .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -f infra/images/pm-server/Dockerfile -t ghcr.io/5dlabs/pm-server:local .
#
# Note: Uses Rust nightly for edition 2024 support
#
# =============================================================================

# =============================================================================
# BUILD STAGES (used only for local development builds)
# =============================================================================
FROM rustlang/rust:nightly-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /build

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Build dependencies - THIS IS THE CACHED LAYER!
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release -p pm --bin pm-server

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

# Install runtime dependencies and argo CLI
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install argo CLI (architecture-aware)
ARG ARGO_VERSION=v3.7.1
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN ARCH=$(dpkg --print-architecture) && \
    curl -sLO "https://github.com/argoproj/argo-workflows/releases/download/${ARGO_VERSION}/argo-linux-${ARCH}.gz" && \
    gunzip "argo-linux-${ARCH}.gz" && \
    chmod +x "argo-linux-${ARCH}" && \
    mv "argo-linux-${ARCH}" /usr/local/bin/argo

RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

WORKDIR /app

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

# Copy the pm-server binary from builder
COPY --from=builder /build/target/release/pm-server /app/pm-server

RUN chmod +x /app/pm-server && chown -R app:app /app

USER app

EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8081/health || exit 1

CMD ["./pm-server"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

# Copy pre-built binary (placed by CI workflow into build context)
COPY pm-server /app/pm-server

RUN chmod +x /app/pm-server && \
    chown -R app:app /app

USER app

EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8081/health || exit 1

CMD ["./pm-server"]

# Default target for CI builds
FROM production
