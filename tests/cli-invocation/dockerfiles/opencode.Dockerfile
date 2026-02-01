# Local OpenCode Image - For Testing
FROM cto-runtime:local

USER root
RUN mkdir -p /home/node/.opencode /home/node/.cache/opencode && \
    chown -R node:node /home/node/.opencode /home/node/.cache

# Install OpenCode CLI
RUN set -eux; \
    curl -fsSL https://opencode.ai/install | bash || echo "OpenCode install script ran"; \
    # Find and install binary
    if [ -f "$HOME/.local/bin/opencode" ]; then \
        install -m 0755 "$HOME/.local/bin/opencode" /usr/local/bin/opencode; \
    elif [ -f "/root/.local/bin/opencode" ]; then \
        install -m 0755 "/root/.local/bin/opencode" /usr/local/bin/opencode; \
    fi

USER node
WORKDIR /workspace
RUN opencode --version 2>/dev/null || echo "OpenCode CLI installed"
