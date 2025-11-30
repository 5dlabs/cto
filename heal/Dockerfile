# Build stage for heal (cross-compile for amd64)
FROM --platform=linux/amd64 rust:1.83-bookworm AS builder

WORKDIR /app

# Copy workspace Cargo files
COPY Cargo.toml Cargo.lock ./
COPY heal/Cargo.toml ./heal/
COPY cli/Cargo.toml ./cli/
COPY controller/Cargo.toml ./controller/
COPY tools/Cargo.toml ./tools/
COPY mcp/Cargo.toml ./mcp/
COPY tasks/Cargo.toml ./tasks/

# Create lib.rs stubs for workspace members we don't need
RUN mkdir -p cli/src controller/src tools/src mcp/src tasks/src && \
    touch cli/src/lib.rs controller/src/lib.rs tools/src/lib.rs mcp/src/lib.rs tasks/src/lib.rs

# Copy the actual heal source
COPY heal/src ./heal/src

# Build the heal binary
RUN cargo build --release --package heal

# Copy prompts
COPY heal/prompts ./heal/prompts

# Runtime stage - use factory image as base (includes droid CLI)
FROM --platform=linux/amd64 ghcr.io/5dlabs/factory:latest

# Install additional runtime dependencies
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Install kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/

# Install gh CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt-get update && \
    apt-get install -y gh && \
    rm -rf /var/lib/apt/lists/*

# Copy heal binary and prompts
COPY --from=builder /app/target/release/heal /usr/local/bin/
COPY --from=builder /app/heal/prompts /app/prompts

WORKDIR /app

# Verify tools are available
RUN echo "Checking installed tools..." && \
    which kubectl && kubectl version --client && \
    which gh && gh --version && \
    (which droid && droid --version || echo "droid not available") && \
    (which heal && heal --help | head -5 || true)

# Default command
ENTRYPOINT ["heal"]
CMD ["--help"]
