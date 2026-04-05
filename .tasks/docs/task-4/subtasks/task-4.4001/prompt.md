Implement subtask 4001: Initialize Rust/Axum project with PostgreSQL and Redis connectivity

## Objective
Set up the Rust project with Axum 0.7 framework, configure database connection pools for PostgreSQL (via sqlx) and Redis, and establish the module structure for the finance service.

## Steps
1. Run `cargo init` and add dependencies: axum 0.7, tokio, sqlx (with postgres feature), redis-rs (or deadpool-redis), serde/serde_json, tower, tower-http, tracing, tracing-subscriber. 2. Create module structure: src/main.rs, src/config.rs, src/db.rs, src/routes/ (mod.rs, invoices.rs, payments.rs, payroll.rs, reports.rs, currency.rs), src/models/, src/services/, src/errors.rs. 3. In src/config.rs, read DATABASE_URL, REDIS_URL, STRIPE_SECRET_KEY, and STRIPE_WEBHOOK_SECRET from environment variables (sourced via envFrom on the infra-endpoints ConfigMap). 4. In src/db.rs, create a PostgreSQL connection pool using sqlx::PgPool and a Redis connection pool using deadpool-redis. 5. In src/main.rs, initialize tracing, build connection pools, create Axum router with shared state (AppState containing pools and Stripe config), and bind to port 8080. 6. Add a basic GET /healthz handler that returns 200. 7. Verify `cargo build` succeeds and the service starts and connects to both databases.

## Validation
cargo build compiles without errors; cargo run starts the server; /healthz returns 200; logs show successful PostgreSQL and Redis connection; AppState is properly shared across routes.