# Runtime stage - expects pre-built binaries
# Using regular Docker Hub (ECR auth issue resolved)
FROM debian:bookworm-slim

# Install only essential runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user
RUN useradd -r -u 1000 -m -d /app -s /bin/bash app

# Set working directory
WORKDIR /app

# Copy pre-built binary (must exist in build context)
COPY controller/agent-controller /app/agent-controller

# Make binary executable and change ownership to app user  
RUN chmod +x /app/agent-controller && \
    chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run the application
CMD ["./agent-controller"]
