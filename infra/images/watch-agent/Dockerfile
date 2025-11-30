# Watch Agent Image
# Contains play-monitor binary and required CLI tools for E2E monitoring loop

FROM alpine:3.20

# Install required tools
RUN apk add --no-cache \
    bash \
    curl \
    git \
    jq \
    yq \
    ca-certificates \
    openssl \
    openssh-client

# Install kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/arm64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install argo CLI
RUN curl -sLO https://github.com/argoproj/argo-workflows/releases/download/v3.6.2/argo-linux-arm64.gz && \
    gunzip argo-linux-arm64.gz && \
    chmod +x argo-linux-arm64 && \
    mv argo-linux-arm64 /usr/local/bin/argo

# Install gh CLI
RUN curl -sL https://github.com/cli/cli/releases/download/v2.62.0/gh_2.62.0_linux_arm64.tar.gz | \
    tar xz -C /tmp && \
    mv /tmp/gh_2.62.0_linux_arm64/bin/gh /usr/local/bin/ && \
    rm -rf /tmp/gh_*

# Copy play-monitor binary
COPY play-monitor /usr/local/bin/play-monitor
RUN chmod +x /usr/local/bin/play-monitor

# Create workspace directories
RUN mkdir -p /workspace/watch /workspace/config

WORKDIR /workspace

# Default entrypoint
ENTRYPOINT ["/usr/local/bin/play-monitor"]

