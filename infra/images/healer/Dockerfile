# Healer Monitor Dockerfile
#
# The CI workflow builds the binary and copies it + prompts to repo root.
# Docker build context is the repo root (.).
#
# To build locally:
#   cargo build --release --package healer
#   cp target/release/healer healer
#   cp -r crates/healer/prompts prompts
#   docker build -f infra/images/healer/Dockerfile -t healer .

# hadolint ignore=DL3029,DL3007
FROM --platform=linux/amd64 ghcr.io/5dlabs/factory:latest

# Install additional runtime dependencies
USER root

# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install gh CLI
# hadolint ignore=DL3008
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y --no-install-recommends gh && \
    rm -rf /var/lib/apt/lists/*

# Copy pre-built healer binary (placed by CI workflow into build context)
COPY healer /usr/local/bin/healer
RUN chmod +x /usr/local/bin/healer

# Copy prompts (placed by CI workflow into build context)
COPY prompts /app/prompts

WORKDIR /app

# Verify tools are available
# hadolint ignore=SC2015
RUN echo "Checking installed tools..." && \
    which kubectl && kubectl version --client && \
    which gh && gh --version && \
    (which droid && droid --version || echo "droid not available") && \
    (which healer && healer --help | head -5 || true)

# Run as non-root user (factory base image user)
USER 1000

# Default command
ENTRYPOINT ["healer"]
CMD ["--help"]
