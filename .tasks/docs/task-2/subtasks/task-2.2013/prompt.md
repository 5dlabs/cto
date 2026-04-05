Implement subtask 2013: Create multi-stage Dockerfile for catalog service

## Objective
Write a multi-stage Dockerfile using rust:1.75 as builder and gcr.io/distroless/cc as runtime, producing a minimal production image for the catalog service.

## Steps
1. Create `services/rust/Dockerfile.catalog`:
   - Stage 1 (builder): `FROM rust:1.75-bookworm AS builder`
     - Install system dependencies needed for compilation (libssl-dev, pkg-config).
     - Copy `Cargo.toml`, `Cargo.lock`, and all workspace member `Cargo.toml` files first for dependency caching.
     - Run `cargo build --release --bin catalog -p catalog` to build dependencies layer.
     - Copy source code and rebuild.
   - Stage 2 (runtime): `FROM gcr.io/distroless/cc-debian12`
     - Copy the compiled binary from builder.
     - Copy SQLx migrations directory.
     - Set `EXPOSE 8080`.
     - Set entrypoint to the binary.
2. Add `.dockerignore` excluding `target/`, `.git/`, etc.
3. Optimize for layer caching: separate dependency and source copy steps.
4. Ensure the binary is statically linked or the distroless image has required shared libs.
5. Target final image size < 50MB.

## Validation
Run `docker build -f Dockerfile.catalog -t catalog:test .` and verify it completes successfully. Verify image size < 50MB with `docker images`. Run `docker run --rm -e POSTGRES_URL=... catalog:test` and verify it starts and responds to /health/live. Verify /health/ready fails gracefully without DB (expected 503).