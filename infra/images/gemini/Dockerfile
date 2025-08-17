ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

# Extra utilities Gemini image expects beyond the base
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    rsync \
    ripgrep \
    psmisc \
    lsof \
    socat \
    bc \
  && rm -rf /var/lib/apt/lists/*

# Switch to root for npm global install, then drop back to node
USER root
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin

# Install Gemini CLI from npm (generic VERSION only)
ARG VERSION=latest
RUN npm install -g @google/gemini-cli@${VERSION} && \
    npm cache clean --force

# Drop privileges for runtime
USER node

# Default entrypoint
CMD ["gemini"]


