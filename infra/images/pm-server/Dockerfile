# PM Server Dockerfile
#
# The CI workflow builds the binary and copies it to repo root.
# Docker build context is the repo root (.).
#
# To build locally:
#   cargo build --release --package pm --bin pm-server
#   cp target/release/pm-server pm-server
#   docker build -f infra/images/pm-server/Dockerfile -t pm-server .

FROM debian:bookworm-slim

# Install only essential runtime dependencies
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user
RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

# Set working directory
WORKDIR /app

# Copy pre-built binary (placed by CI workflow into build context)
COPY pm-server /app/pm-server

# Make binary executable and change ownership to app user  
RUN chmod +x /app/pm-server && \
    chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8081/health || exit 1

# Run the application
CMD ["./pm-server"]
