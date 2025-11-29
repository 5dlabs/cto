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

# Configure MinIO client (mc) alias via environment variable if credentials are set
# MC_HOST_<alias> format: https://ACCESS_KEY:SECRET_KEY@ENDPOINT
if [ -n "$MINIO_ENDPOINT" ] && [ -n "$MINIO_ACCESS_KEY" ] && [ -n "$MINIO_SECRET_KEY" ]; then
    # Remove protocol prefix for MC_HOST format
    MINIO_HOST="${MINIO_ENDPOINT#http://}"
    MINIO_HOST="${MINIO_HOST#https://}"
    export MC_HOST_minio="http://${MINIO_ACCESS_KEY}:${MINIO_SECRET_KEY}@${MINIO_HOST}"
    echo "MinIO client configured with alias 'minio' -> ${MINIO_ENDPOINT}"
fi

# Run the application
cd /config
exec "$@"
