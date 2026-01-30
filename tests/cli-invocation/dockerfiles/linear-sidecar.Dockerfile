# Local Linear Sidecar Image - For Testing
# Builds from source using the linear-sink crate
FROM rust:latest AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p linear-sink --bin linear-sidecar

# Runtime - Ubuntu 24.04 for glibc 2.39 compatibility
FROM ubuntu:24.04

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

USER app
ENV RUST_LOG=info

CMD ["./linear-sidecar"]
