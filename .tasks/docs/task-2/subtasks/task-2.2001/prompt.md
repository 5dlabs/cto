Implement subtask 2001: Project scaffold with Cargo.toml, app state, and configuration

## Objective
Initialize the Rust project with all dependencies, create the application state struct, configuration loading from environment variables, and the main entrypoint with Axum server setup, structured logging, and graceful shutdown.

## Steps
1. Run `cargo init notifycore` and configure Cargo.toml with all dependencies: axum 0.7, tokio 1 (full features), sqlx 0.7 (postgres, runtime-tokio, tls-rustls, migrate), serde 1 + serde_json, uuid (v4, serde), chrono (serde), tracing 0.1, tracing-subscriber (json, env-filter), redis 0.25 (as optional feature behind `cache` flag), tower-http (trace, cors).
2. Create `src/config.rs` — read `DATABASE_URL` (required), `REDIS_URL` (optional), `PORT` (default 8080), `RUST_LOG` (default info) from env vars.
3. Create `src/state.rs` — `AppState` struct holding `PgPool` and `Option<redis::Client>`. Implement constructor that creates the PgPool, optionally connects to Redis (failing silently), and runs sqlx migrations.
4. Create `src/main.rs` — initialize tracing_subscriber with JSON formatter and env filter from RUST_LOG. Build `AppState`, construct Axum Router (placeholder routes), bind to `0.0.0.0:{PORT}`, add `with_graceful_shutdown` using `tokio::signal::ctrl_c()`.
5. Add tower-http TraceLayer and CorsLayer to the router.
6. Ensure `cargo build` compiles successfully.

## Validation
`cargo build` compiles without errors. `cargo clippy -- -D warnings` passes. The binary starts and binds to the configured port when DATABASE_URL is provided (will fail healthily if no DB is available).