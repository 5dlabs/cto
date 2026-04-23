# syntax=docker/dockerfile:1.7
# -----------------------------------------------------------------------------
# project-api — HTTP sidecar for the Morgan OpenClaw pod.
#
# Two-stage build:
#   1. `builder` compiles the TS source into a single binary via `bun build`.
#   2. The runtime image is a slim Debian base with `git` + `ca-certificates`
#      (required for cloning GitHub repos over HTTPS).
# -----------------------------------------------------------------------------

ARG BASE_REGISTRY=""
ARG BUN_VERSION=1.1.40

# --- builder -----------------------------------------------------------------
FROM ${BASE_REGISTRY}oven/bun:${BUN_VERSION} AS builder
WORKDIR /app

COPY package.json ./
COPY tsconfig.json ./
RUN bun install --frozen-lockfile || bun install

COPY src/ ./src/
RUN bun build ./src/index.ts --compile --outfile ./project-api

# --- runtime ------------------------------------------------------------------
FROM ${BASE_REGISTRY}debian:stable-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates \
      git \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 1001 --home-dir /home/morgan --create-home morgan
USER morgan
WORKDIR /app

COPY --from=builder /app/project-api /app/project-api

ENV PORT=8091
EXPOSE 8091

ENTRYPOINT ["/app/project-api"]
