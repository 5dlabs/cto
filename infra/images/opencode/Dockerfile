ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
USER root
RUN mkdir -p /workspace /home/node/.opencode && \
  chown -R 1000:1000 /home/node/.opencode
WORKDIR /workspace

# Install Open Code (as root), then place binary in /usr/local/bin for all users
USER root
RUN set -eux; \
  # Install required tools for Debian/Ubuntu
  apt-get update && apt-get install -y curl bash unzip && rm -rf /var/lib/apt/lists/*; \
  curl -fsSL https://opencode.ai/install | bash; \
  # Install the opencode binary
  install -m 0755 "/root/.opencode/bin/opencode" /usr/local/bin/opencode; \
  opencode --version

USER node

# Open Code CLI is now available via `opencode`
