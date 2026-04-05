Implement task 2: Build Equipment Catalog Service (Rex - Rust/Axum)

## Goal
Implement the high-performance Equipment Catalog API serving 533+ products across 24 categories with real-time availability checking, barcode/SKU lookup, image serving via R2 CDN, and a machine-readable equipment API for AI agents. This is the first Rust service and establishes the Cargo workspace mono-repo structure shared by Finance and Vetting services.

## Task Context
- Agent owner: rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize Cargo workspace mono-repo structure:
   - `sigma1-services/Cargo.toml` (workspace root)
   - `sigma1-services/crates/shared-auth/` — RBAC middleware crate reading `sigma1-rbac-roles` ConfigMap JSON, JWT validation
   - `sigma1-services/crates/shared-db/` — sqlx connection pool setup, migration runner, health check helpers
   - `sigma1-services/crates/shared-error/` — unified error types, Axum error response formatting
   - `sigma1-services/crates/shared-observability/` — Prometheus metrics (axum-prometheus), structured logging (tracing + tracing-subscriber)
   - `sigma1-services/services/equipment-catalog/` — binary crate
2. Define database migrations (sqlx) in `public` schema:
   - `categories` table: id (UUID PK), name, parent_id (self-ref FK), icon, sort_order, created_at
   - `products` table: id (UUID PK), name, category_id (FK), description, sku (UNIQUE), barcode (UNIQUE nullable), day_rate (BIGINT cents), weight_kg, dimensions (JSONB), image_urls (TEXT[]), specs (JSONB), created_at, updated_at
   - `availability` table: product_id (FK), date (DATE), quantity_total, quantity_reserved, quantity_booked, PRIMARY KEY (product_id, date)
   - Indexes: products(category_id), products(sku), products(barcode), availability(product_id, date range using GiST)
   - Seed migration with 24 categories from PRD
3. Implement Axum 0.7 router with endpoints:
   - `GET /api/v1/catalog/categories` — list all categories with product counts
   - `GET /api/v1/catalog/products` — paginated list with filters (category_id, search text, price range, availability date range), using sqlx query builder
   - `GET /api/v1/catalog/products/:id` — full product detail with current availability
   - `GET /api/v1/catalog/products/:id/availability?from=&to=` — date-range availability check, must respond < 500ms
   - `POST /api/v1/catalog/products` — admin-only create (RBAC: admin role)
   - `PATCH /api/v1/catalog/products/:id` — admin-only update
   - `GET /api/v1/equipment-api/catalog` — machine-readable JSON for AI agents (all products, simplified schema)
   - `POST /api/v1/equipment-api/checkout` — programmatic booking (creates availability reservation)
   - `GET /metrics` — Prometheus metrics via shared-observability crate
   - `GET /health/live` — liveness (process alive)
   - `GET /health/ready` — readiness (DB connection + Valkey connection healthy)
4. Rate limiting middleware using Valkey:
   - Token bucket per API key or IP, configurable via env vars
   - Default: 100 req/min for public, 1000 req/min for authenticated
5. Caching layer:
   - Category list cached in Valkey with 5-minute TTL
   - Product detail cached with 1-minute TTL, invalidated on PATCH
   - Availability queries NOT cached (real-time requirement)
6. Image URL generation:
   - Product image_urls stored as R2 object keys
   - Resolve to Cloudflare CDN URLs at response time via configured CDN base URL
7. Kubernetes Deployment manifest:
   - Namespace: `sigma1`, 2 replicas
   - `envFrom: configMapRef: sigma1-infra-endpoints`
   - Resource limits: 256Mi memory, 250m CPU
   - Liveness/readiness probes pointing to health endpoints
8. Dockerfile: multi-stage build (rust:1.75-slim builder, distroless runtime)
9. Use `rust_decimal` for any price arithmetic in reports/aggregations.

## Acceptance Criteria
1. Unit tests for availability calculation logic: given reservations and bookings, verify quantity_available is correct (>= 5 test cases). 2. Integration test: POST a product as admin, GET it back, verify all fields match including image CDN URLs. 3. Integration test: check availability for a product with existing bookings returns correct available quantity within < 500ms (measure with `std::time::Instant`). 4. Rate limiting test: send 101 requests in rapid succession from same IP, verify 429 status on request 101. 5. RBAC test: attempt POST /api/v1/catalog/products without admin role, verify 403. 6. Machine-readable endpoint test: GET /api/v1/equipment-api/catalog returns valid JSON array with expected fields (id, name, day_rate, category). 7. Health endpoint test: /health/ready returns 200 when DB and Valkey connected, 503 when either is down. 8. Cargo workspace builds all three service binaries from single `cargo build --workspace` command.

## Subtasks
- Initialize Cargo workspace root and shared-error crate: Create the sigma1-services Cargo workspace root with workspace-level dependency management and implement the shared-error crate providing unified error types and Axum error response formatting used by all services.
- Implement shared-db crate with sqlx connection pool and health checks: Build the shared-db crate providing PostgreSQL connection pool initialization via sqlx, migration runner helper, and database health check function, reading connection details from environment variables (sourced from sigma1-infra-endpoints ConfigMap).
- Implement shared-observability crate with Prometheus metrics and structured logging: Build the shared-observability crate providing Prometheus metrics integration for Axum via axum-prometheus and structured JSON logging via tracing + tracing-subscriber.
- Implement shared-auth crate with JWT validation and RBAC middleware: Build the shared-auth crate providing JWT token validation middleware and RBAC role-checking for Axum, reading role definitions from the sigma1-rbac-roles ConfigMap.
- Create database schema migrations for categories, products, and availability tables: Define sqlx migrations for the equipment catalog database schema including categories (with self-referencing hierarchy), products (with SKU, barcode, pricing, JSONB fields), and availability tables with appropriate indexes and seed data for 24 categories.
- Implement Axum router scaffold with health endpoints and application bootstrap: Create the equipment-catalog binary crate with Axum 0.7 application bootstrap, router composition, shared state (DB pool, Valkey connection), health/liveness/readiness endpoints, and middleware wiring.
- Implement category listing endpoint with product counts: Build the GET /api/v1/catalog/categories endpoint returning all categories with their hierarchical structure and product counts per category.
- Implement product CRUD endpoints (list, detail, create, update): Build the product listing with pagination/filtering, product detail, admin-only create, and admin-only update endpoints, including image URL resolution to CDN paths.
- Implement availability checking endpoint and reservation logic: Build the date-range availability checking endpoint and the reservation creation logic with <500ms performance requirement, including the availability calculation (total - reserved - booked = available).
- Implement Valkey-based rate limiting middleware: Build token-bucket rate limiting middleware using Valkey, supporting per-IP and per-API-key limits configurable via environment variables.
- Implement Valkey caching layer for categories and products: Add Valkey-based caching for category list (5-minute TTL) and product detail (1-minute TTL with invalidation on update), keeping availability queries uncached.
- Implement machine-readable equipment API endpoints for AI agents: Build the GET /api/v1/equipment-api/catalog and POST /api/v1/equipment-api/checkout endpoints providing a simplified, machine-optimized interface for AI agent integration.
- Create multi-stage Dockerfile and Kubernetes deployment manifest: Build the multi-stage Dockerfile using rust:1.75-slim builder and distroless runtime, plus the Kubernetes Deployment, Service, and related manifests for the equipment-catalog service.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.