# syntax=docker/dockerfile:1.7-labs

ARG BASE_IMAGE=registry.5dlabs.ai/5dlabs/agents:latest-arm64
FROM ${BASE_IMAGE}

LABEL org.opencontainers.image.source="https://gitlab.5dlabs.ai/5dlabs/cto"
LABEL org.opencontainers.image.description="Local OpenClaw platform image for CTO desktop bootstrap"

USER root

ENV NPM_CONFIG_UPDATE_NOTIFIER=false \
    NPM_CONFIG_AUDIT=false \
    NPM_CONFIG_FUND=false

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --ignore-scripts \
      clawhub@latest \
      @zed-industries/claude-agent-acp \
      @zed-industries/codex-acp \
      pi-acp \
      supergateway \
      mcporter && \
    openclaw --version && \
    clawhub --cli-version

RUN mkdir -p \
      /workspace \
      /workspace/.openclaw \
      /workspace/.claude \
      /workspace/.codex \
      /workspace/.gemini \
      /workspace/.cursor \
      /workspace/.factory \
      /workspace/.kimi \
      /workspace/.pi \
      /workspace/.local/bin && \
    chown -R node:node /workspace

USER node
ENV HOME=/workspace \
    PATH="/workspace/.local/bin:/usr/local/share/npm-global/bin:/usr/local/bin:${PATH}"
WORKDIR /workspace
