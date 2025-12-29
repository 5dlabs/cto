ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Switch to root for all setup
USER root

# Install Claude CLI as root (before modifying permissions)
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}

# Setup workspace and user environment with proper permissions
# Create all directories Claude CLI needs and ensure node user owns entire home
RUN mkdir -p /workspace /home/node/.claude /home/node/.config /home/node/.cache && \
    chown -R 1000:1000 /home/node && \
    chown -R 1000:1000 /workspace

WORKDIR /workspace

# Configure git identity as root (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

# Switch to node user for runtime
USER node
