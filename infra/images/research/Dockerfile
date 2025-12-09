# Research Pipeline Dockerfile with cargo-chef
#
# Uses cargo-chef for optimal dependency caching
# Includes Chromium for headless browser automation
#
# To build locally:
#   docker build -f infra/images/research/Dockerfile -t research .

# =============================================================================
# Stage 1: Chef - Prepare recipe
# =============================================================================
FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /build

# =============================================================================
# Stage 2: Planner - Analyze dependencies
# =============================================================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Builder - Build dependencies then application
# =============================================================================
FROM chef AS builder

# Build dependencies - THIS IS THE CACHED LAYER!
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo chef cook --release --recipe-path recipe.json

# Now copy source and build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry-shared \
    cargo build --release --package research && \
    cp /build/target/release/research /tmp/research

# =============================================================================
# Stage 4: Runtime - minimal image with Chromium
# =============================================================================
FROM debian:bookworm-slim

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

# Copy pre-built binary from builder
COPY --from=builder /tmp/research /usr/local/bin/research
RUN chmod +x /usr/local/bin/research

# Create data directories
RUN mkdir -p /data/research && \
    chown -R research:research /data

WORKDIR /data

# Verify binary works
RUN research --help

# Run as non-root user
USER research

# Default command - show help
ENTRYPOINT ["research"]
CMD ["--help"]
