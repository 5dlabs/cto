ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
USER root
RUN mkdir -p /workspace /home/node/.claude && \
  chown -R 1000:1000 /home/node/.claude
WORKDIR /workspace

# Install Claude (generic VERSION only) as root, then drop to node
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}
USER node
