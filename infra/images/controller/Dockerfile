# syntax=docker/dockerfile:1
# =============================================================================
# Agent Controller - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binary in context):
#     docker build -t registry.5dlabs.ai/5dlabs/controller .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -t registry.5dlabs.ai/5dlabs/controller:local .
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

# Copy source and build the application
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release --bin agent-controller

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

WORKDIR /app

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

# Copy binary from builder stage
COPY --from=builder /build/target/release/agent-controller /app/agent-controller

# Copy templates directory
COPY templates /app/templates

RUN chmod +x /app/agent-controller && \
    chown -R app:app /app

USER app

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

CMD ["./agent-controller"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

# Copy pre-built binary (placed by CI workflow into build context)
COPY agent-controller /app/agent-controller

# Copy templates directory
COPY templates/ /app/templates/

RUN chmod +x /app/agent-controller && \
    chown -R app:app /app

USER app

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

CMD ["./agent-controller"]

# Default target for CI builds
FROM production
