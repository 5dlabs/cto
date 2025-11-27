#!/bin/bash
set -e

# Set up runtime paths
export PATH="/usr/local/bin:/usr/local/go/bin:/root/.cargo/bin:/usr/lib/jvm/java-17-openjdk/bin:$PATH"

# Check if Docker socket is available (for Docker-based MCP servers)
if [ -S "/var/run/docker.sock" ]; then
    echo "Docker socket available - Docker-based MCP servers can run containers"
else
    echo "Docker socket not available - Docker-based MCP servers may not work"
fi

# Run the application
cd /config
exec "$@"
