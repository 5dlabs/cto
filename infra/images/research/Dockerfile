# syntax=docker/dockerfile:1
# =============================================================================
# Research Pipeline - Unified Dockerfile
# =============================================================================
#
# Supports both CI (pre-built binary) and local (from source) builds:
#
#   CI/GHCR build (default - expects pre-built binary in context):
#     cargo build --release --package research
#     cp target/release/research research
#     docker build -f infra/images/research/Dockerfile -t ghcr.io/5dlabs/research .
#
#   Local/Kind build (compiles from source):
#     docker build --target local -f infra/images/research/Dockerfile -t ghcr.io/5dlabs/research:local .
#
# Includes Chromium for headless browser automation
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

# Build dependencies - THIS IS THE CACHED LAYER!
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release --package research

# =============================================================================
# RUNTIME BASE (shared by all targets)
# =============================================================================
FROM debian:bookworm-slim AS runtime-base

# Install runtime dependencies including Chromium and git
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    chromium \
    fonts-liberation \
    git \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libx11-xcb1 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Set Chromium path for chromiumoxide
ENV CHROME_PATH=/usr/bin/chromium
ENV CHROMIUM_PATH=/usr/bin/chromium

# Create non-root user with explicit UID to match k8s securityContext
RUN useradd -u 1000 -ms /bin/bash research

# Create data directories
RUN mkdir -p /data/research && \
    chown -R research:research /data

WORKDIR /data

# =============================================================================
# LOCAL BUILD TARGET (compiles from source)
# =============================================================================
FROM runtime-base AS local

# Copy binary from builder stage
COPY --from=builder /build/target/release/research /usr/local/bin/research
RUN chmod +x /usr/local/bin/research

# Verify binary works
RUN research --help

USER research

ENTRYPOINT ["research"]
CMD ["--help"]

# =============================================================================
# PRODUCTION TARGET (default - uses pre-built binary from CI)
# =============================================================================
FROM runtime-base AS production

# Copy pre-built binary (placed by CI workflow into build context)
COPY research /usr/local/bin/research
RUN chmod +x /usr/local/bin/research

# Verify binary works
RUN research --help

USER research

ENTRYPOINT ["research"]
CMD ["--help"]

# Default target for CI builds
FROM production
