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
  CURSOR_SHARE="$HOME/.local/share/cursor-agent"; \
  VERSION_DIR="$(cd "$CURSOR_SHARE/versions" && ls -1 | sort | tail -n1)"; \
  install -d /usr/local/lib/cursor-agent; \
  cp -r "$CURSOR_SHARE/versions/$VERSION_DIR"/. /usr/local/lib/cursor-agent/; \
  install -m 0755 "$HOME/.local/bin/cursor-agent" /usr/local/lib/cursor-agent/cursor-agent; \
  ln -sf ../lib/cursor-agent/cursor-agent /usr/local/bin/cursor-agent; \
  chown -R 1000:1000 /usr/local/lib/cursor-agent; \
  rm -rf "$CURSOR_SHARE"; \
  command -v cursor-agent; \
  cursor-agent --help >/dev/null 2>&1 || true

USER node

# Cursor CLI is now available via `cursor-agent`
