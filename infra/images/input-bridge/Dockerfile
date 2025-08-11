# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Copy controller source files (input-bridge is part of controller crate)
COPY controller/Cargo.toml controller/Cargo.lock ./
COPY controller/src ./src

# Build only the input-bridge binary
RUN cargo build --release --bin input-bridge

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/input-bridge /usr/local/bin/input-bridge

# Create non-root user
RUN useradd -m -u 1000 appuser
USER appuser

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/input-bridge"]