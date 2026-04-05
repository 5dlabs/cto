Implement subtask 2002: Implement shared-db crate with sqlx connection pool and health checks

## Objective
Build the shared-db crate providing PostgreSQL connection pool initialization via sqlx, migration runner helper, and database health check function, reading connection details from environment variables (sourced from sigma1-infra-endpoints ConfigMap).

## Steps
1. Create `crates/shared-db/Cargo.toml` depending on sqlx (from workspace), shared-error, tracing.
2. Implement `pub async fn create_pool(database_url: &str) -> Result<PgPool>` — creates an sqlx PgPool with configurable max_connections (env var, default 10), min_connections (2), acquire_timeout (3s), idle_timeout (300s).
3. Implement `pub async fn run_migrations(pool: &PgPool) -> Result<()>` — wraps `sqlx::migrate!()` macro call.
4. Implement `pub async fn check_health(pool: &PgPool) -> bool` — executes `SELECT 1` with a 2-second timeout, returns true/false.
5. Create a `DatabaseConfig` struct that reads `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB` from env and constructs the connection URL. These come from the infra-endpoints ConfigMap via `envFrom`.
6. Re-export PgPool from sqlx for convenience.

## Validation
Unit test: DatabaseConfig correctly builds connection URL from env vars. Integration test (requires running PostgreSQL): create_pool connects, run_migrations succeeds, check_health returns true. Test check_health returns false when given an invalid pool/connection.