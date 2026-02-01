# Local Linear Sidecar Image - For Testing
# Builds from source using the linear-sink crate
#
# Build args (passed by build-images.sh):
#   GIT_COMMIT - Short commit hash
#   GIT_BRANCH - Branch name
FROM rust:latest AS builder

# Build args for labels
ARG GIT_COMMIT=unknown
ARG GIT_BRANCH=unknown

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p linear-sink --bin linear-sidecar

# Runtime - Ubuntu 24.04 for glibc 2.39 compatibility
FROM ubuntu:24.04

# Build args must be re-declared in each stage
ARG GIT_COMMIT=unknown
ARG GIT_BRANCH=unknown

# OCI image labels for traceability
LABEL org.opencontainers.image.source="https://github.com/5dlabs/cto"
LABEL org.opencontainers.image.revision="${GIT_COMMIT}"
LABEL org.opencontainers.image.ref.name="${GIT_BRANCH}"
LABEL ai.5dlabs.cto.component="linear-sidecar"

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

# Handle existing UID 1000 in Ubuntu 24.04
RUN useradd -r -u 1000 -m -d /app -s /bin/bash app 2>/dev/null || \
    useradd -r -m -d /app -s /bin/bash app
WORKDIR /app

COPY --from=builder /build/target/release/linear-sidecar /app/linear-sidecar
RUN chmod +x /app/linear-sidecar

# Store commit info in the image for runtime inspection
RUN echo "commit=${GIT_COMMIT} branch=${GIT_BRANCH}" > /app/.build-info

USER app
ENV RUST_LOG=info

CMD ["./linear-sidecar"]
