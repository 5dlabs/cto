#!/bin/bash
set -e

# Set up runtime paths
export PATH="/usr/local/bin:/usr/local/go/bin:/root/.cargo/bin:/usr/lib/jvm/java-17-openjdk/bin:$PATH"

# Wait for Docker socket (sidecar may take a few seconds to start)
echo "Waiting for Docker socket..."
WAIT_SECONDS=30
for i in $(seq 1 $WAIT_SECONDS); do
    if [ -S "/var/run/docker.sock" ]; then
        echo "Docker socket available - Docker-based MCP servers can run containers"
        break
    fi
    if [ "$i" -eq "$WAIT_SECONDS" ]; then
        echo "Warning: Docker socket not available after ${WAIT_SECONDS}s - Docker-based MCP servers may not work"
    fi
    sleep 1
done

# Configure S3 client (mc) alias via environment variable if credentials are set
# MC_HOST_<alias> format: PROTOCOL://ACCESS_KEY:SECRET_KEY@HOST
# Works with any S3-compatible storage (SeaweedFS, MinIO, AWS S3, etc.)
if [ -n "$S3_ENDPOINT" ] && [ -n "$S3_ACCESS_KEY" ] && [ -n "$S3_SECRET_KEY" ]; then
    # Detect protocol and extract host
    if [[ "$S3_ENDPOINT" == https://* ]]; then
        S3_PROTOCOL="https"
        S3_HOST="${S3_ENDPOINT#https://}"
    else
        S3_PROTOCOL="http"
        S3_HOST="${S3_ENDPOINT#http://}"
    fi
    export MC_HOST_s3="${S3_PROTOCOL}://${S3_ACCESS_KEY}:${S3_SECRET_KEY}@${S3_HOST}"
    echo "S3 client configured with alias 's3' -> ${S3_ENDPOINT}"
fi

# Run the application
cd /config
exec "$@"
