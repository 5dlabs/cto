# Local Every Code Image - For Testing
# Note: This is @just-every/code, a Codex fork with more functionality
FROM cto-runtime:local

USER root
RUN npm install -g @just-every/code openai@latest
RUN mkdir -p /home/node/.code /home/node/.codex && chown -R node:node /home/node/.code /home/node/.codex

USER node
WORKDIR /workspace
RUN code --version || echo "Every Code CLI installed"
