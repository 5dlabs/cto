ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Switch to root for setup
USER root

# Setup workspace and user environment with proper permissions
# Create all directories Claude CLI needs and ensure node user owns them
RUN mkdir -p /workspace /home/node/.claude /home/node/.config /home/node/.cache && \
    chown -R 1000:1000 /home/node /workspace

WORKDIR /workspace

# Install Claude CLI as root
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}

# Switch to node user for runtime
USER node

# Configure Claude-specific git identity
RUN git config --global user.name "5D Labs Agent" && \
    git config --global user.email "agent@5dlabs.com"
