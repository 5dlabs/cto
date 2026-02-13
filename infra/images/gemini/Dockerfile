# syntax=docker/dockerfile:1
ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

USER root

# Setup workspace and user environment
RUN mkdir -p /workspace /home/node/.gemini /home/node/.config /home/node/.cache && \
    chown -R node:node /home/node/.gemini /home/node/.config /home/node/.cache && \
    chown -R node:node /workspace

WORKDIR /workspace

# Install Google Gemini CLI
# https://geminicli.com/docs/get-started/installation/
# Pin to a stable version for reproducible builds; override with build-arg if needed.
ARG VERSION=0.28.2
RUN --mount=type=cache,target=/root/.npm,sharing=shared \
    npm install -g --no-audit --no-fund @google/gemini-cli@${VERSION} && \
    echo "Gemini CLI installed"

# Configure git identity (system-wide for all users)
RUN git config --system user.name "5D Labs Agent" && \
    git config --system user.email "agent@5dlabs.com"

USER node

# Verify installation
RUN gemini --version && echo "Gemini CLI ready"

CMD ["gemini", "--help"]
