# syntax=docker/dockerfile:1
# =============================================================================
# Ralph Runner Image
# =============================================================================
# Persistent loop runner for CTO lifecycle tests.
# Based on claude image (has claude CLI, kubectl, gh, jq, etc.)
# =============================================================================

ARG BASE_IMAGE=ghcr.io/5dlabs/claude:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

USER root

# Copy ralph scripts
COPY scripts/ralph-cto.sh /usr/local/bin/ralph-cto.sh
COPY infra/images/ralph-runner/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/ralph-cto.sh /usr/local/bin/entrypoint.sh

# Create working directory for ralph state
RUN mkdir -p /ralph && chown -R node:node /ralph

# Switch back to non-root user
USER node

WORKDIR /ralph

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
