Implement subtask 5001: Scaffold Rust Axum project with shared application state and configuration

## Objective
Initialize the Customer Vetting Service as a Rust Axum project with configuration loading (environment variables, config files), shared application state (database pool, HTTP clients), error handling patterns, and basic Axum router setup.

## Steps
1. Create a new Rust project using `cargo init` with workspace-aware Cargo.toml.
2. Add dependencies: axum 0.7, tokio, serde, serde_json, dotenvy, tracing, tracing-subscriber, thiserror.
3. Define an `AppConfig` struct loaded from environment variables (DATABASE_URL, OPENCORPORATES_API_KEY, LINKEDIN_API_KEY, GOOGLE_API_KEY, CREDIT_API_KEY, LISTEN_ADDR, etc.).
4. Define an `AppState` struct that holds the database pool (sqlx::PgPool) and a shared reqwest::Client for outbound HTTP calls.
5. Implement a `main.rs` that initializes tracing, loads config, builds AppState, and starts the Axum server with a placeholder health route.
6. Define a shared error type (`AppError`) implementing `IntoResponse` for consistent JSON error responses with appropriate HTTP status codes.
7. Reference the infra-endpoints ConfigMap via `envFrom` for database connection strings and service URLs.

## Validation
The application compiles, starts, and responds to `GET /healthz` with HTTP 200. Configuration loads correctly from environment variables. Tracing output appears in stdout.