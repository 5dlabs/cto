ARG BASE_IMAGE=ghcr.io/5dlabs/runtime:latest
FROM ${BASE_IMAGE}

ARG TZ
ENV TZ="$TZ"

ARG USERNAME=node
ENV DEVCONTAINER=true

USER root

# Ensure workspace, Factory home, and temp directories exist for the non-root user
RUN mkdir -p /workspace /home/${USERNAME}/.factory /tmp/factory-cli-images && \
    chown -R 1000:1000 /workspace /home/${USERNAME}/.factory /tmp/factory-cli-images && \
    chmod 775 /tmp/factory-cli-images

WORKDIR /workspace

# Install Factory Droid CLI in a location accessible to all users
RUN set -eux; \
    curl -fsSL https://app.factory.ai/cli | sh; \
    if [ ! -f /root/.local/bin/droid ]; then \
      echo "Factory CLI installer failed" >&2; \
      exit 1; \
    fi; \
    install -m 0755 /root/.local/bin/droid /usr/local/bin/droid; \
    rm -f /root/.local/bin/droid; \
    install -d -m 0755 /home/${USERNAME}/.factory/bin; \
    if [ -f /root/.factory/bin/rg ]; then \
      install -m 0755 /root/.factory/bin/rg /home/${USERNAME}/.factory/bin/rg; \
    fi; \
    rm -rf /root/.factory || true

# Make sure the user can persist Factory CLI state and binaries
RUN install -d -m 0775 /home/${USERNAME}/.factory/logs /home/${USERNAME}/.factory/cache && \
    printf '{}' > /home/${USERNAME}/.factory/settings.json && \
    chown -R 1000:1000 /home/${USERNAME}/.factory

ENV PATH="/home/${USERNAME}/.factory/bin:${PATH}"

USER ${USERNAME}

# Print the CLI version during build for traceability
RUN FACTORY_DISABLE_TELEMETRY=1 droid --version
