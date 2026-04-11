FROM ghcr.io/5dlabs/agents:latest

USER root

# Pre-install ACP adapters so acpx doesn't need runtime npm downloads
# codex-acp: https://github.com/zed-industries/codex-acp
# claude-agent-acp: https://github.com/agentclientprotocol/claude-agent-acp
RUN npm uninstall -g @zed-industries/claude-agent-acp 2>/dev/null; \
    npm install -g --no-audit --no-fund --force \
      @zed-industries/codex-acp@latest \
      @agentclientprotocol/claude-agent-acp@latest && \
    echo "=== ACP adapters installed ===" && \
    which codex-acp && \
    which claude-agent-acp

# Install Deno (used by cto-tools SDK)
RUN curl -fsSL https://deno.land/install.sh | DENO_INSTALL=/usr/local sh

USER node
WORKDIR /workspace
