# Runtime stage - expects pre-built binary copied in build context
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy prebuilt binary from repository context
COPY input-bridge/input-bridge /usr/local/bin/input-bridge

RUN chmod +x /usr/local/bin/input-bridge && chown -R appuser:appuser /usr/local/bin/input-bridge

USER appuser

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/input-bridge"]