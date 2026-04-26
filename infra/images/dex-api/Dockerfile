# syntax=docker/dockerfile:1
# =============================================================================
# DEX API - Multi-stage Dockerfile
# =============================================================================
#
# CI build (pre-built binary):
#   cargo build --release -p dex-indexer --bin dex-api
#   cp target/release/dex-api dex-api
#   docker build -f infra/images/dex-api/Dockerfile -t ghcr.io/5dlabs/dex-api .
#
# Local build (from source):
#   docker build --target local -f infra/images/dex-api/Dockerfile \
#     -t ghcr.io/5dlabs/dex-api:local .
#
# =============================================================================

# =============================================================================
# BUILD STAGE (local development only)
# =============================================================================
FROM rust:1.86-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release -p dex-indexer --bin dex-api

# =============================================================================
# RUNTIME BASE
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

WORKDIR /app

ENV RUST_LOG=dex_indexer=info

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

COPY --from=builder /build/target/release/dex-api /app/dex-api

RUN chmod +x /app/dex-api && chown -R app:app /app

USER app

EXPOSE 50051

CMD ["./dex-api"]

# =============================================================================
# PRODUCTION TARGET (default — uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

COPY dex-api /app/dex-api

RUN chmod +x /app/dex-api && chown -R app:app /app

USER app

EXPOSE 50051

CMD ["./dex-api"]

# Default target for CI builds
FROM production
