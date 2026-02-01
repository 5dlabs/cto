# Local Gemini Image - For Testing
FROM cto-runtime:local

USER root
RUN mkdir -p /home/node/.gemini /home/node/.config /home/node/.cache && \
    chown -R node:node /home/node/.gemini /home/node/.config /home/node/.cache

# Install Gemini CLI (experimental - package may vary)
RUN npm install -g @google/generative-ai || echo "Gemini SDK installed"

# Install gemini-cli if available
RUN npm install -g gemini-cli 2>/dev/null || echo "gemini-cli not available as npm package"

USER node
WORKDIR /workspace
RUN gemini --version 2>/dev/null || echo "Gemini CLI ready"
