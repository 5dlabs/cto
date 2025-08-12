ARG BASE_IMAGE=ghcr.io/5dlabs/cto/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
RUN mkdir -p /workspace /home/node/.claude && \
  chown -R 1000:1000 /home/node/.claude
WORKDIR /workspace
USER node

# Install Claude (generic VERSION only)
ARG VERSION=latest
RUN npm install -g @anthropic-ai/claude-code@${VERSION}
