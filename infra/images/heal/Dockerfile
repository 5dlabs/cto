# Heal Monitor Dockerfile
#
# Expects a pre-built binary at infra/images/heal/heal-linux
# The CI workflow builds the binary with sccache caching before Docker build.
#
# To build locally:
#   cargo build --release --package heal
#   cp target/release/heal infra/images/heal/heal-linux
#   docker build -f infra/images/heal/Dockerfile -t heal .

# Runtime stage - use factory image as base (includes droid CLI)
FROM --platform=linux/amd64 ghcr.io/5dlabs/factory:latest

# Install additional runtime dependencies
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install gh CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Copy pre-built heal binary (must exist in build context)
COPY infra/images/heal/heal-linux /usr/local/bin/heal
RUN chmod +x /usr/local/bin/heal

# Copy prompts from source
COPY heal/prompts /app/prompts

WORKDIR /app

# Verify tools are available
RUN echo "Checking installed tools..." && \
    which kubectl && kubectl version --client && \
    which gh && gh --version && \
    (which droid && droid --version || echo "droid not available") && \
    (which heal && heal --help | head -5 || true)

# Default command
ENTRYPOINT ["heal"]
CMD ["--help"]
