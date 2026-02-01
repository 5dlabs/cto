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
# The cursor-agent wrapper script expects node and index.js in the same directory
USER root
RUN set -eux; \
  curl https://cursor.com/install -fsS | bash; \
  export PATH="$HOME/.local/bin:$PATH"; \
  CURSOR_SHARE="$HOME/.local/share/cursor-agent"; \
  VERSION_DIR="$(cd "$CURSOR_SHARE/versions" && ls -1 | sort | tail -n1)"; \
  # Create lib directory and copy the full cursor-agent package
  install -d /usr/local/lib/cursor-agent; \
  cp -r "$CURSOR_SHARE/versions/$VERSION_DIR"/. /usr/local/lib/cursor-agent/; \
  # Symlink system node into the cursor-agent directory (script expects $SCRIPT_DIR/node)
  ln -sf /usr/bin/node /usr/local/lib/cursor-agent/node; \
  # Create wrapper script that uses the lib directory
  printf '#!/bin/bash\nexec /usr/local/lib/cursor-agent/cursor-agent "$@"\n' > /usr/local/bin/cursor-agent; \
  chmod +x /usr/local/bin/cursor-agent; \
  ln -sf cursor-agent /usr/local/bin/cursor; \
  chown -R 1000:1000 /usr/local/lib/cursor-agent; \
  rm -rf "$CURSOR_SHARE"; \
  # Verify installation
  command -v cursor-agent; \
  cursor-agent --version || echo "Warning: cursor-agent version check failed"

USER node

# Cursor CLI is now available via `cursor-agent` or `cursor`
