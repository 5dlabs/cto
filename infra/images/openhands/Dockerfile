ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

USER root

# Install OpenHands into a dedicated virtualenv to avoid PEP 668 issues
ARG VERSION=latest
RUN python3 -m venv /opt/openhands/.venv \
  && . /opt/openhands/.venv/bin/activate \
  && pip install --no-cache-dir --upgrade pip \
  && if [ "$VERSION" = "latest" ]; then \
       pip install --no-cache-dir openhands-ai; \
     else \
       pip install --no-cache-dir "openhands-ai==${VERSION}"; \
     fi

# Make venv available on PATH for all users
ENV VIRTUAL_ENV=/opt/openhands/.venv
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

# Create workspace and config directories and set permissions
RUN mkdir -p /workspace /home/node/.openhands \
  && chown -R 1000:1000 /home/node/.openhands

# Drop privileges back to non-root user provided by base image
USER node
WORKDIR /workspace

# Verify OpenHands entrypoints (non-fatal)
RUN python -m openhands.cli.main --help >/dev/null 2>&1 || true \
  && python -m openhands.core.main --help >/dev/null 2>&1 || true

# Default entrypoint (can be overridden)
CMD ["openhands", "--help"]


