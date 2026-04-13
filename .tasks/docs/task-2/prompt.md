<identity>
You are rex, the Rust 1.75+/Axum 0.7 implementation agent. You own task 2 end-to-end.
</identity>

<context>
<task_overview>
Task 2: Build Equipment Catalog Service (Rex - Rust/Axum)
Implement the high-performance Equipment Catalog microservice in Rust using Axum 0.7. Serves 533+ products across 24 categories with real-time availability checking, barcode/SKU lookup, image URL serving via Cloudflare R2, a machine-readable AI agent API, and rate limiting via Valkey. Exposes all catalog REST endpoints plus health and metrics. Implements internal GDPR data export and deletion endpoints.
Priority: high
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Initialize Cargo workspace at services/equipment-catalog. Add dependencies: axum 0.7, tokio 1.x (full), sqlx 0.7 (postgres, uuid, chrono, rust_decimal, json), redis 0.24 (tokio-comp), aws-sdk-s3 1.x, tower 0.4, tower-http (cors, trace, compression), prometheus 0.13, uuid 1.x (v4, serde), serde/serde_json, anyhow, tracing/tracing-subscriber.
2. Database: Use DATABASE_URL from sigma1-infra-endpoints ConfigMap (envFrom). Run sqlx migrations in migrations/ targeting the catalog schema: set search_path=catalog. Tables: categories (id UUID PK, name TEXT, parent_id UUID nullable, icon TEXT, sort_order INT), products (id UUID PK, category_id UUID FK, name TEXT, description TEXT, day_rate NUMERIC(10,2), weight_kg FLOAT4, dimensions JSONB, image_urls TEXT[], specs JSONB, barcode TEXT, created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ), availability_blocks (id UUID PK, product_id UUID FK, date_from DATE, date_to DATE, quantity_reserved INT, quantity_booked INT).
3. Implement handlers: GET /api/v1/catalog/categories, GET /api/v1/catalog/products (query params: category_id, search, page, limit), GET /api/v1/catalog/products/:id, GET /api/v1/catalog/products/:id/availability?from=&to=, POST /api/v1/catalog/products (admin JWT required), PATCH /api/v1/catalog/products/:id (admin JWT required), GET /api/v1/equipment-api/catalog (machine-readable JSON for AI agents, include specs and availability windows), POST /api/v1/equipment-api/checkout (programmatic booking endpoint, validates availability and creates availability_block row).
4. Availability check logic: query availability_blocks for product_id WHERE date ranges overlap; compute quantity_available = product.stock_quantity - SUM(reserved + booked) for requested window. Target: < 500ms p99.
5. JWT middleware: extract Bearer token, validate against JWT_SECRET from sigma1-jwt-secret. Admin role check for write endpoints.
6. Valkey rate limiting: use sliding window counter per (tenant_id or IP) with 100 req/min limit. Use redis::aio::ConnectionManager.
7. Prometheus metrics: register Counter(requests_total, labels: method, route, status), Histogram(request_duration_seconds). Expose GET /metrics.
8. Health endpoints: GET /health/live returns 200 immediately; GET /health/ready checks db ping and redis ping, returns 200 or 503.
9. GDPR endpoints (internal, no external JWT required but validate internal-network origin via Cilium policy): GET /internal/gdpr/export/:customer_id returns all catalog data associated with customer (checkout records); DELETE /internal/gdpr/delete/:customer_id removes PII from checkout records.
10. Kubernetes Deployment: 2 replicas, envFrom sigma1-infra-endpoints ConfigMap and relevant Secrets, liveness/readiness probes, resource limits (256Mi RAM, 250m CPU). Image: ghcr.io/sigma1/equipment-catalog:latest.
11. Write unit tests for availability calculation logic and rate limit middleware. Write integration tests using sqlx test transactions.
</implementation_plan>

<acceptance_criteria>
1. GET /api/v1/catalog/products returns JSON array with at least 1 seeded product, status
200. 2. GET /api/v1/catalog/products/:id/availability?from=2025-01-01&to=2025-01-03 returns availability object with quantity_available >= 0 in under 500ms measured via curl -w '%{time_total}'.
3. POST /api/v1/catalog/products without JWT returns
401. 4. GET /health/ready returns 200 when postgres and valkey are reachable, 503 when either is down (test by temporarily scaling postgres to 0).
5. GET /metrics returns Prometheus text format with requests_total and request_duration_seconds metrics.
6. Rate limiter: 101 rapid requests from same IP returns HTTP 429 on the 101st.
7. GET /internal/gdpr/export/:id returns customer checkout JSON; subsequent DELETE /internal/gdpr/delete/:id causes re-fetch to return empty result.
8. cargo test passes with >= 80% coverage reported by cargo-tarpaulin.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Initialize Cargo workspace, declare all dependencies, and write sqlx catalog schema migrations: Bootstrap the services/equipment-catalog Cargo workspace with Cargo.toml, all required crate dependencies pinned to specified versions, and write the three sqlx migration files for the catalog schema tables.
- Implement application entrypoint, AppState, and database/Valkey connection pool setup: Write src/main.rs and src/state.rs establishing the Axum application entry point, shared AppState struct containing PgPool and Redis ConnectionManager, and configuration loading from environment variables.
- Implement JWT extraction and validation middleware for admin-protected write endpoints: Write a reusable Axum middleware (or extractor) that extracts Bearer tokens from Authorization headers, validates them against JWT_SECRET, and enforces an 'admin' role claim for write endpoints.
- Implement catalog CRUD handlers: categories list and products list/get/create/update: Write Axum handlers for GET /api/v1/catalog/categories, GET /api/v1/catalog/products (with pagination and search), GET /api/v1/catalog/products/:id, POST /api/v1/catalog/products, and PATCH /api/v1/catalog/products/:id.
- Implement availability calculation logic and GET /availability endpoint with unit tests: Write the core availability computation function and expose it via GET /api/v1/catalog/products/:id/availability?from=&to=, with unit tests covering overlap edge cases.
- Implement Valkey sliding-window rate limiter middleware: Write an Axum middleware layer that enforces a 100 requests/minute sliding-window rate limit per client IP (or tenant_id claim), using the Redis ConnectionManager.
- Implement AI agent machine-readable catalog endpoint and programmatic checkout endpoint: Write GET /api/v1/equipment-api/catalog returning full catalog JSON with specs and availability windows, and POST /api/v1/equipment-api/checkout atomically validating availability and creating an availability_block row.
- Implement Prometheus metrics middleware and /metrics endpoint: Register a request counter and request duration histogram with Prometheus, instrument all HTTP handlers via a tower middleware layer, and expose GET /metrics in Prometheus text format.
- Implement health check endpoints: /health/live and /health/ready: Write GET /health/live (always 200) and GET /health/ready (checks Postgres and Valkey connectivity, returns 200 or 503) handlers.
- Implement internal GDPR export and deletion endpoints: Write GET /internal/gdpr/export/:customer_id and DELETE /internal/gdpr/delete/:customer_id handlers that respectively return and remove all catalog-related PII for a given customer from availability_blocks checkout records.
- Write Dockerfile and Kubernetes Deployment/Service manifests for equipment-catalog: Create a multi-stage Dockerfile for the Rust service and Kubernetes Deployment (2 replicas) and Service manifests with envFrom referencing sigma1-infra-endpoints and required secrets, plus liveness/readiness probes.
- Write integration test suite and verify cargo-tarpaulin coverage >= 80%: Write sqlx-based integration tests using test transactions for catalog handlers, availability logic, and GDPR endpoints, then run cargo-tarpaulin to confirm >= 80% line coverage.
</subtasks>