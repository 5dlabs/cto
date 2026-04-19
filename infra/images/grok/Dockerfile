# syntax=docker/dockerfile:1
# =============================================================================
# Grok X Search Cron Job
# =============================================================================
#
# Lightweight Bun image for the Grok X Search cron job.
# No browser automation — API-only via the Grok/xAI API.
#
#   CI build (default — uses pre-compiled standalone binary):
#     bun build apps/grok/src/cron.ts --compile --target=bun-linux-x64 --outfile=grok-cron
#     docker build -f infra/images/grok/Dockerfile -t registry.5dlabs.ai/5dlabs/grok .
#
#   Local build (compiles from source inside the container):
#     docker build --target local -f infra/images/grok/Dockerfile -t registry.5dlabs.ai/5dlabs/grok:local .
#
# =============================================================================

# =============================================================================
# LOCAL BUILD TARGET (compiles from source using Bun)
# =============================================================================
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}oven/bun:1-slim AS local

WORKDIR /app

# Install dependencies first (cached layer)
COPY apps/grok/package.json apps/grok/bun.lock* ./
RUN bun install --frozen-lockfile 2>/dev/null || bun install

# Copy source
COPY apps/grok/ ./

# Create non-root user
RUN useradd -u 1000 -ms /bin/bash grok || true
RUN mkdir -p /data && chown -R 1000:1000 /data

USER 1000
ENV GROK_OUTPUT_DIR=/data

ENTRYPOINT ["bun", "run", "src/cron.ts"]

# =============================================================================
# PRODUCTION TARGET (uses pre-compiled standalone binary from CI)
# =============================================================================
ARG BASE_REGISTRY=""
FROM ${BASE_REGISTRY}debian:bookworm-slim AS production

# Install minimal runtime dependencies
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -u 1000 -ms /bin/bash grok
RUN mkdir -p /data && chown -R grok:grok /data

WORKDIR /data

# Copy pre-built standalone binary (placed by CI into build context)
COPY grok-cron /usr/local/bin/grok-cron
RUN chmod +x /usr/local/bin/grok-cron

USER grok
ENV GROK_OUTPUT_DIR=/data

ENTRYPOINT ["grok-cron"]

# Default target for CI builds
FROM production
