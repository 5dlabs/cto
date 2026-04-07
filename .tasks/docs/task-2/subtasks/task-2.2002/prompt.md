Implement subtask 2002: Implement application entrypoint, AppState, and database/Valkey connection pool setup

## Objective
Write src/main.rs and src/state.rs establishing the Axum application entry point, shared AppState struct containing PgPool and Redis ConnectionManager, and configuration loading from environment variables.

## Steps
Create src/main.rs: initialize tracing_subscriber with EnvFilter from RUST_LOG env var. Load config from env: DATABASE_URL (from CNPG_SIGMA1_URL + PGPASSWORD), VALKEY_URL (from VALKEY_SIGMA1_URL), JWT_SECRET. Create sqlx::PgPool via PgPoolOptions with max_connections: 10, connect_lazy: false. Create redis::aio::ConnectionManager from VALKEY_URL. Construct AppState { db: PgPool, redis: ConnectionManager, jwt_secret: String }. Wrap in Arc<AppState>. Build Axum Router (routes added in later subtasks), attach tower-http layers: TraceLayer, CompressionLayer, CorsLayer (permissive for dev). Bind to 0.0.0.0:8080. Create src/state.rs defining pub struct AppState and impl AppState with db(), redis(), jwt_secret() accessors. Create src/config.rs reading all env vars with descriptive errors on missing values.

## Validation
cargo build succeeds. Running the binary with valid DATABASE_URL and VALKEY_URL env vars starts without panic and logs 'listening on 0.0.0.0:8080'. curl http://localhost:8080/ returns 404 (router not yet populated — confirming server is alive).