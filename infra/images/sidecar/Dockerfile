# Runtime stage - expects pre-built binary copied in build context
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy prebuilt binary from repository context (produced by CI into sidecar/sidecar)
COPY sidecar/sidecar /usr/local/bin/sidecar

RUN chmod +x /usr/local/bin/sidecar && chown -R appuser:appuser /usr/local/bin/sidecar

USER appuser

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/sidecar"]