# =============================================================================
# Local Claude Image - For Testing
# =============================================================================
FROM cto-runtime:local

USER root

# Install Claude CLI
RUN npm install -g @anthropic-ai/claude-code

# Install tools binary (CTO MCP client)
COPY tools-linux-amd64 /usr/local/bin/tools
RUN chmod +x /usr/local/bin/tools

# Setup Claude directories
RUN mkdir -p /home/node/.claude /home/node/.config /home/node/.cache && \
    chown -R node:node /home/node

# Git config
RUN git config --system user.name "CTO Test Agent" && \
    git config --system user.email "test@5dlabs.ai"

WORKDIR /workspace
USER node

# Verify installation
RUN claude --version && tools --help | head -3
