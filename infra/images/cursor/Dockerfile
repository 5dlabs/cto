ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true
USER root
RUN mkdir -p /workspace /home/node/.cursor && \
  chown -R 1000:1000 /home/node/.cursor
WORKDIR /workspace

# Install Cursor CLI (as root), then place binary in /usr/local/bin for all users
USER root
RUN set -eux; \
  curl https://cursor.com/install -fsS | bash; \
  export PATH="$HOME/.local/bin:$PATH"; \
  if [ -f "$HOME/.local/bin/cursor-agent" ]; then \
    install -m 0755 "$HOME/.local/bin/cursor-agent" /usr/local/bin/cursor-agent; \
  fi; \
  command -v cursor-agent; \
  cursor-agent --help >/dev/null 2>&1 || true

USER node

# Cursor CLI is now available via `cursor-agent`

