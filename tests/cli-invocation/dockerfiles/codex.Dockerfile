# Local Codex Image - For Testing
FROM cto-runtime:local

USER root
RUN npm install -g @openai/codex openai@latest
RUN mkdir -p /home/node/.codex && chown -R node:node /home/node/.codex

USER node
WORKDIR /workspace
RUN codex --version || echo "Codex CLI installed"
