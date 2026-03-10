# syntax=docker/dockerfile:1.7-labs

ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:full
FROM ${BASE_IMAGE}

ARG OPENCLAW_VERSION=beta

LABEL org.opencontainers.image.source="https://github.com/5dlabs/cto"
LABEL org.opencontainers.image.description="Local OpenClaw platform image for CTO desktop bootstrap"

USER root

ENV NPM_CONFIG_UPDATE_NOTIFIER=false \
    NPM_CONFIG_AUDIT=false \
    NPM_CONFIG_FUND=false

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm install -g --ignore-scripts "openclaw@${OPENCLAW_VERSION}" clawhub@latest && \
    openclaw --version && \
    clawhub --cli-version

RUN mkdir -p /workspace /home/node/.openclaw && \
    chown -R node:node /workspace /home/node/.openclaw

USER node
ENV HOME=/workspace
WORKDIR /workspace
