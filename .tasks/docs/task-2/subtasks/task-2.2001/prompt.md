Implement subtask 2001: Initialize Rust project and configure Axum framework

## Objective
Set up a new Rust project for the Equipment Catalog Service, configure Axum 0.7.5 with Tokio runtime, and establish a basic server structure.

## Steps
1. Run `cargo new --bin equipment-catalog` targeting Rust 1.77.2.2. Add `axum = "0.7.5"` and `tokio = { version = "1", features = ["full"] }` to `Cargo.toml`.3. Implement a basic 'Hello World' Axum server to verify setup.

## Validation
Run `cargo run` and verify the server starts without errors and responds to a basic HTTP request (e.g., `curl http://localhost:3000`).