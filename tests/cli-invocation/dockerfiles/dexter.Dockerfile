# Local Dexter Image - For Testing
# Note: Dexter uses Bun runtime (TypeScript)
FROM cto-runtime:local

USER root

# Install Bun using npm (more reliable than install script)
RUN npm install -g bun

RUN mkdir -p /home/node/.dexter && chown -R node:node /home/node/.dexter

USER node
WORKDIR /workspace
RUN bun --version && echo "Bun runtime ready for Dexter"
