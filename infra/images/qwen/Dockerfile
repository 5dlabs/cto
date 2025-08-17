ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

# Use root for global npm install, then drop privileges
USER root

# Allow version override via build-arg (generic VERSION only)
ARG VERSION=latest

# Install Qwen Code CLI globally
RUN npm install -g @qwen-code/qwen-code@${VERSION}

USER node

# Set working directory
WORKDIR /workspace

# Default command to run the CLI
CMD ["qwen"]
