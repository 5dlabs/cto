Implement subtask 2002: Implement shared health check handlers (liveness and readiness)

## Objective
Add health check route handlers to the shared crate: GET /health/live returns 200 unconditionally, GET /health/ready checks PostgreSQL and Valkey connectivity and returns 200 or 503.

## Steps
1. In `shared/src/health.rs`, implement `pub async fn liveness() -> impl IntoResponse` returning `StatusCode::OK` with `{"status": "ok"}`.
2. Implement `pub async fn readiness(State(pool): State<PgPool>, State(valkey): State<ValkeyCon>) -> impl IntoResponse` that:
   - Executes `SELECT 1` on PgPool
   - Executes `PING` on Valkey connection
   - Returns 200 if both succeed, 503 with `{"status": "degraded", "checks": {"db": "ok/fail", "valkey": "ok/fail"}}` if either fails.
3. Add `redis-rs` (with `tokio-comp` feature) to shared dependencies for Valkey connectivity.
4. Implement `shared::valkey` module: `pub async fn create_valkey_client() -> Result<Client>` reading `VALKEY_URL` from env.
5. Export a `pub fn health_routes() -> Router` that mounts both handlers.

## Validation
Unit test liveness always returns 200. Integration test with real Postgres and Valkey: readiness returns 200. Test readiness returns 503 with degraded status when Valkey is unavailable (use invalid URL).