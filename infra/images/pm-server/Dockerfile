# Runtime stage - expects pre-built binaries
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

# Copy pre-built binary (placed by CI workflow into build context)
# The binary is built as 'linear' but deployed as 'pm-server'
COPY linear /app/pm-server

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
