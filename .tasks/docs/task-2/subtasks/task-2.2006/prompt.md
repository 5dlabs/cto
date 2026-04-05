Implement subtask 2006: Implement Axum router scaffold with health endpoints and application bootstrap

## Objective
Create the equipment-catalog binary crate with Axum 0.7 application bootstrap, router composition, shared state (DB pool, Valkey connection), health/liveness/readiness endpoints, and middleware wiring.

## Steps
1. Create `services/equipment-catalog/Cargo.toml` depending on all shared crates, axum 0.7, tokio, serde, serde_json, uuid, chrono, rust_decimal.
2. Define `AppState` struct (Clone): pg_pool (PgPool), valkey_pool (redis connection manager), cdn_base_url (String). Implement as Axum state with `FromRef` for sub-extractors.
3. In `main.rs`: read config from env vars, call shared-db `create_pool`, run migrations, initialize Valkey connection (using `redis` crate with `sigma1-infra-endpoints` VALKEY_URL), call `init_logging()`, build router.
4. Router composition: nest `/api/v1/catalog` routes, nest `/api/v1/equipment-api` routes, add metrics layer from shared-observability, add auth middleware on protected routes only.
5. `GET /health/live` — returns 200 `{"status": "ok"}` always.
6. `GET /health/ready` — calls shared-db `check_health` AND pings Valkey with `PING`; returns 200 if both healthy, 503 with details if either fails.
7. Bind to `0.0.0.0:8080` (configurable via `PORT` env var), graceful shutdown on SIGTERM.
8. Create `mod routes`, `mod models`, `mod handlers` module structure.

## Validation
Integration test: start the application, verify GET /health/live returns 200. Verify GET /health/ready returns 200 when DB and Valkey are available. Verify GET /health/ready returns 503 when DB is unavailable. Verify GET /metrics returns Prometheus text format. Verify the server binds and accepts connections on configured port.