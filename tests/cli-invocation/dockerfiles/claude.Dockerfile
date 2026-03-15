# =============================================================================
# Local Claude Image - For Testing
#
# Build args (passed by build-images.sh):
#   GIT_COMMIT - Short commit hash
#   GIT_BRANCH - Branch name
# =============================================================================
FROM cto-runtime:local

# Build args for labels
ARG GIT_COMMIT=unknown
ARG GIT_BRANCH=unknown

# OCI image labels for traceability
LABEL org.opencontainers.image.source="https://git.5dlabs.ai/5dlabs/cto"
LABEL org.opencontainers.image.revision="${GIT_COMMIT}"
LABEL org.opencontainers.image.ref.name="${GIT_BRANCH}"
LABEL ai.5dlabs.cto.component="claude"

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
