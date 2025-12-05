# Research Pipeline Dockerfile
#
# This image includes Chromium for headless browser automation
# to fetch Twitter/X bookmarks.
#
# The CI workflow builds the binary and copies it to repo root.
# Docker build context is the repo root (.).
#
# To build locally:
#   cargo build --release --package research
#   cp target/release/research research
#   docker build -f infra/images/research/Dockerfile -t research .

# =============================================================================
# Builder stage - compile the Rust binary
# =============================================================================
# hadolint ignore=DL3007
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --package research

# =============================================================================
# Runtime stage - minimal image with Chromium
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies including Chromium
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    chromium \
    fonts-liberation \
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
COPY --from=builder /build/target/release/research /usr/local/bin/research
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

