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

USER node
WORKDIR /workspace
