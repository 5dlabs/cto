# Runtime stage - Use Ubuntu for glibc compatibility
FROM ubuntu:24.04

# Install system dependencies and all runtimes
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    bash \
    # Node.js runtime
    nodejs \
    npm \
    # Python runtime
    python3 \
    python3-dev \
    python3-pip \
    # Go runtime
    golang-go \
    # Java runtime
    openjdk-17-jre \
    # Docker CLI for Docker-based MCP servers
    docker.io \
    # Core utilities
    coreutils \
    findutils \
    grep \
    sed \
    tar \
    gzip \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl separately
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/arm64/kubectl" && \
    install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl && \
    rm kubectl

# Create symbolic links for Python
RUN ln -sf python3 /usr/bin/python
# Install UV and uvx for Python package management
RUN curl -LsSf https://astral.sh/uv/install.sh | bash && \
    (mv /root/.local/bin/uv /usr/local/bin/ && mv /root/.local/bin/uvx /usr/local/bin/) 2>/dev/null || \
    (mv /root/.cargo/bin/uv /usr/local/bin/ && mv /root/.cargo/bin/uvx /usr/local/bin/) 2>/dev/null || \
    echo "UV and uvx installed successfully"

# Install minimal Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# Copy the prebuilt server binary from CI/CD artifacts
COPY tools-server-linux /usr/local/bin/tools-server
RUN chmod +x /usr/local/bin/tools-server

# Create non-root user (use a different UID since 1000 exists)
RUN useradd -m -u 1001 -s /bin/bash mcp

# Create directories for configs and npm cache
RUN mkdir -p /config /home/mcp/.npm && \
    chown -R mcp:mcp /config /home/mcp

# Set up environment for all runtimes
ENV PORT=3000 \
    PROJECT_DIR=/config \
    PATH="/usr/local/go/bin:/root/.cargo/bin:/usr/lib/jvm/java-17-openjdk/bin:$PATH" \
    PYTHONUNBUFFERED=1 \
    NODE_ENV=production \
    JAVA_HOME=/usr/lib/jvm/java-17-openjdk \
    DOTNET_ROOT=/usr/lib/dotnet \
    DOCKER_HOST=unix:///var/run/docker.sock

# Create entrypoint script to handle environment setup
RUN cat > /entrypoint.sh << 'EOF'
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
EOF

RUN chmod +x /entrypoint.sh

# Switch to non-root user
USER mcp
WORKDIR /config

# Expose default port
EXPOSE 3000

# Use custom entrypoint
ENTRYPOINT ["/entrypoint.sh"]
CMD ["tools-server", "--port", "3000", "--project-dir", "/config"]
