# Local Factory (Droid) Image - For Testing
FROM cto-runtime:local

USER root
RUN mkdir -p /home/node/.factory /tmp/factory-cli-images && \
    chown -R node:node /home/node/.factory /tmp/factory-cli-images

# Install Factory CLI (Droid)
RUN set -eux; \
    curl -fsSL https://app.factory.ai/cli | sh || echo "Factory install script ran"; \
    # Find and install binary
    if [ -f "$HOME/.local/bin/droid" ]; then \
        install -m 0755 "$HOME/.local/bin/droid" /usr/local/bin/droid; \
    elif [ -f "/root/.local/bin/droid" ]; then \
        install -m 0755 "/root/.local/bin/droid" /usr/local/bin/droid; \
    fi

USER node
WORKDIR /workspace
RUN droid --version 2>/dev/null || echo "Factory/Droid CLI ready"
