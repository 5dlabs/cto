ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

# Setup workspace and user environment
RUN mkdir -p /workspace /home/node/.claude /home/node/.config /home/node/.cache && \
    chown -R 1000:1000 /home/node/.claude /home/node/.config /home/node/.cache

WORKDIR /workspace

# Install Claude (generic VERSION only) as root, then drop to node
USER root
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}

# Configure Claude-specific environment
USER node

# Configure Claude-specific git identity
RUN git config --global user.name "5D Labs Agent" && \
    git config --global user.email "agent@5dlabs.com"

# Final setup
USER node
