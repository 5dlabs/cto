## Implement Equipment Catalog Service (Rex - Rust/Axum)

### Objective
Build the high-performance Equipment Catalog REST API serving 533+ products across 24 categories with real-time availability checking, barcode/SKU lookup, image serving via R2, a machine-readable equipment API for AI agents, and rate limiting. This is the first service in the shared Rex Cargo workspace.

### Ownership
- Agent: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize Cargo workspace at repo root `services/rust/` with members: `catalog`, `finance`, `vetting`, `shared`. The `shared` crate contains common types (health check handlers, error types, middleware, DB pool setup, ConfigMap env parsing, API key validation middleware).
2. In `shared` crate:
   - Database pool setup using `sqlx` with `PgPool` reading `POSTGRES_URL` from env (injected via `sigma1-infra-endpoints` ConfigMap).
   - Health check handlers: `GET /health/live` (returns 200), `GET /health/ready` (checks DB + Valkey connectivity).
   - Prometheus metrics middleware using `metrics` + `metrics-exporter-prometheus` crates, exposed at `GET /metrics`.
   - Rate limiting middleware using Valkey (sliding window, `redis-rs` crate), configurable per-route.
   - API key validation middleware reading from `sigma1-service-api-keys` secret (for inter-service auth per D7).
   - Standard JSON error response type.
3. In `catalog` crate:
   - SQLx migrations for `catalog` schema: `categories` table, `products` table (with `specs` as JSONB, `image_urls` as TEXT[], `day_rate` as DECIMAL), `availability` table (product_id, date_from, date_to, quantity_total, reserved, booked), `bookings` table.
   - Implement all REST endpoints per PRD:
     - `GET /api/v1/catalog/categories` — list with optional `parent_id` filter
     - `GET /api/v1/catalog/products` — paginated, filterable by category_id, name search (trigram), specs JSONB queries
     - `GET /api/v1/catalog/products/:id` — full product detail with category join
     - `GET /api/v1/catalog/products/:id/availability?from=&to=` — real-time availability check against bookings, must respond < 500ms
     - `POST /api/v1/catalog/products` — admin create (API key auth)
     - `PATCH /api/v1/catalog/products/:id` — admin update (API key auth)
     - `GET /api/v1/equipment-api/catalog` — machine-readable JSON (flat structure optimized for AI agent consumption)
     - `POST /api/v1/equipment-api/checkout` — programmatic booking (creates reservation, decrements availability)
   - Image URLs stored as R2 presigned URLs or public CDN paths.
   - Generate OpenAPI spec using `utoipa` crate, served at `/api/v1/catalog/openapi.json`.
   - GDPR endpoint: `DELETE /api/v1/gdpr/customer/:id` — delete all customer-related booking data, return confirmation.
4. Dockerfile: multi-stage build (rust:1.75 builder → distroless/cc runtime).
5. Kubernetes Deployment manifest:
   - Namespace: `sigma1`
   - Replicas: 2
   - `envFrom: [{configMapRef: {name: sigma1-infra-endpoints}}]`
   - Secret refs for `sigma1-db-credentials`, `sigma1-r2-credentials`, `sigma1-service-api-keys`
   - Liveness/readiness probes on `/health/live` and `/health/ready`
   - Resource limits: 256Mi memory, 500m CPU
   - Service exposing port 8080.
6. Seed data script: SQL file with 24 categories and sample products for development.

### Subtasks
- [ ] Initialize Cargo workspace and shared crate with DB pool, error types, and ConfigMap env parsing: Create the Cargo workspace at services/rust/ with members catalog, finance, vetting, and shared. Implement the shared crate foundation: PgPool setup reading POSTGRES_URL from env (sigma1-infra-endpoints ConfigMap), standard JSON error response type, and env/config parsing utilities.
- [ ] Implement shared health check handlers (liveness and readiness): Add health check route handlers to the shared crate: GET /health/live returns 200 unconditionally, GET /health/ready checks PostgreSQL and Valkey connectivity and returns 200 or 503.
- [ ] Implement shared Prometheus metrics middleware: Add request metrics middleware to the shared crate using the metrics and metrics-exporter-prometheus crates, exposed at GET /metrics.
- [ ] Implement shared rate limiting middleware using Valkey: Build a configurable per-route rate limiting middleware in the shared crate using a sliding window algorithm backed by Valkey (redis-rs).
- [ ] Implement shared API key validation middleware: Build an API key authentication middleware in the shared crate that validates keys from the Authorization header against values in the sigma1-service-api-keys secret.
- [ ] Create SQLx migrations for catalog schema: Write and validate SQLx migrations for the catalog database schema: categories, products (with JSONB specs, TEXT[] image_urls, DECIMAL day_rate), availability, and bookings tables with appropriate indexes.
- [ ] Implement category and product CRUD endpoints with pagination and search: Build the core catalog REST endpoints: categories listing with parent_id filter, products listing with pagination/category filter/trigram search/JSONB spec queries, product detail, and admin create/update endpoints.
- [ ] Implement availability checking and atomic checkout endpoints: Build the real-time availability endpoint (GET /api/v1/catalog/products/:id/availability) and the programmatic checkout endpoint (POST /api/v1/equipment-api/checkout) with atomic inventory decrement.
- [ ] Implement machine-readable equipment API endpoint and OpenAPI spec generation: Build the GET /api/v1/equipment-api/catalog endpoint returning a flat JSON structure optimized for AI agent consumption, and generate the OpenAPI spec using utoipa served at /api/v1/catalog/openapi.json.
- [ ] Implement GDPR deletion endpoint: Build the DELETE /api/v1/gdpr/customer/:id endpoint that removes all customer-related booking data and returns a confirmation response.
- [ ] Create seed data script with 24 categories and sample products: Write a SQL seed data file with 24 equipment rental categories and representative sample products with realistic specs, pricing, and availability data for development.
- [ ] Build catalog service main entrypoint with router composition: Create the catalog service binary entrypoint that composes all routes, middleware layers, and shared infrastructure (DB pool, Valkey, metrics, rate limiting) into a running Axum server.
- [ ] Create multi-stage Dockerfile for catalog service: Write a multi-stage Dockerfile using rust:1.75 as builder and gcr.io/distroless/cc as runtime, producing a minimal production image for the catalog service.
- [ ] Create Kubernetes deployment manifests for catalog service: Write Kubernetes Deployment, Service, and related manifests for the catalog service in the sigma1 namespace with proper ConfigMap/Secret references, probes, and resource limits.