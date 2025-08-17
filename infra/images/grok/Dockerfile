ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

# Use root for global install; base image has node user
USER root
WORKDIR /workspace

# Install Grok CLI globally (supports generic VERSION but defaults to latest)
ARG VERSION=latest
RUN npm install -g @vibe-kit/grok-cli@"${VERSION}"

# Copy any custom Grok configuration scripts
COPY --chown=1000:1000 scripts/ /workspace/scripts/
RUN chmod +x /workspace/scripts/*.sh

# Drop privileges for runtime
USER node
ENV NODE_ENV=production
ENV GROK_API_KEY=""

# Default working directory for tasks
WORKDIR /workspace/project

# Default command - can be overridden by TaskRun
CMD ["/workspace/scripts/grok-entrypoint.sh"]