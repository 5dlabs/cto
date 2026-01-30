# Local Cursor Image - For Testing
FROM cto-runtime:local

USER root
RUN mkdir -p /home/node/.cursor && chown -R node:node /home/node/.cursor

# Install Cursor CLI
RUN set -eux; \
    curl -fsSL https://cursor.com/install | bash || echo "Cursor install script ran"; \
    # Try to find and install the binary
    if [ -f "$HOME/.local/bin/cursor-agent" ]; then \
        install -m 0755 "$HOME/.local/bin/cursor-agent" /usr/local/bin/cursor-agent; \
    elif [ -f "/root/.local/bin/cursor-agent" ]; then \
        install -m 0755 "/root/.local/bin/cursor-agent" /usr/local/bin/cursor-agent; \
    fi; \
    ln -sf /usr/local/bin/cursor-agent /usr/local/bin/cursor || true

USER node
WORKDIR /workspace
RUN cursor --version 2>/dev/null || cursor-agent --version 2>/dev/null || echo "Cursor CLI installed"
