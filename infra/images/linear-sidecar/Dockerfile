# syntax=docker/dockerfile:1
# =============================================================================
# Linear Sidecar (status-sync) - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binary in context):
#     cargo build --release -p pm --bin status-sync
#     cp target/release/status-sync status-sync
#     docker build -f infra/images/linear-sidecar/Dockerfile -t ghcr.io/5dlabs/linear-sidecar .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -f infra/images/linear-sidecar/Dockerfile -t ghcr.io/5dlabs/linear-sidecar:local .
#
# Note: Uses Rust nightly for edition 2024 support
#
# =============================================================================

# =============================================================================
# BUILD STAGES (used only for local development builds)
# =============================================================================
FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the status-sync binary (enhanced sidecar v2)
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release -p pm --bin status-sync

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

WORKDIR /app

# Default environment variables
ENV STATUS_FILE=/workspace/status.json \
    LINEAR_SERVICE_URL=http://linear-svc.cto.svc.cluster.local:8081 \
    STATUS_POLL_INTERVAL_MS=5000 \
    LOG_FILE_PATH=/workspace/agent.log \
    LOG_POST_INTERVAL_MS=5000 \
    CLAUDE_STREAM_FILE=/workspace/claude-stream.jsonl \
    STREAM_POLL_INTERVAL_MS=500 \
    INPUT_FIFO_PATH=/workspace/agent-input.jsonl \
    INPUT_POLL_INTERVAL_MS=2000 \
    HTTP_PORT=8080 \
    RUST_LOG=info

EXPOSE 8080

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

COPY --from=builder /build/target/release/status-sync /app/status-sync

RUN chmod +x /app/status-sync && chown -R app:app /app

USER app

CMD ["./status-sync"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

# Copy pre-built binary (placed by CI workflow into build context)
COPY status-sync /app/status-sync

RUN chmod +x /app/status-sync && chown -R app:app /app

USER app

CMD ["./status-sync"]

# Default target for CI builds
FROM production

