Implement subtask 2012: Build catalog service main entrypoint with router composition

## Objective
Create the catalog service binary entrypoint that composes all routes, middleware layers, and shared infrastructure (DB pool, Valkey, metrics, rate limiting) into a running Axum server.

## Steps
1. In `catalog/src/main.rs`:
   - Parse config from environment using shared::config.
   - Initialize PgPool via shared::db::create_pool().
   - Run SQLx migrations on startup (`sqlx::migrate!().run(&pool).await`).
   - Initialize Valkey client via shared::valkey::create_valkey_client().
   - Initialize Prometheus metrics via shared::metrics::init_metrics().
   - Build AppState struct holding PgPool, Valkey client, ApiKeyStore, PrometheusHandle.
2. Compose the Axum router:
   - Mount health routes at `/health/*`
   - Mount metrics route at `/metrics`
   - Mount catalog category/product routes at `/api/v1/catalog/*`
   - Mount equipment API routes at `/api/v1/equipment-api/*`
   - Mount GDPR routes at `/api/v1/gdpr/*`
   - Apply metrics middleware globally
   - Apply rate limiting middleware to public endpoints
   - Apply API key auth middleware to admin and GDPR routes
3. Bind to `0.0.0.0:8080` and start serving.
4. Add graceful shutdown handling on SIGTERM.
5. Add startup log with version, bound address, and configured endpoints.

## Validation
Integration test: start the full application against test database and Valkey, verify all routes respond correctly (health, metrics, catalog, equipment-api, GDPR). Verify metrics middleware is recording. Verify rate limiting is active on public routes. Verify API key auth protects admin routes. Test graceful shutdown: send SIGTERM and verify in-flight requests complete.