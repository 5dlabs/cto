Implement task 2: Implement NotifyCore Rust Service (Rex - Rust/Axum)

## Goal
Build the complete NotifyCore notification routing service in Rust using Axum 0.7 and sqlx 0.7, implementing all five REST endpoints, PostgreSQL persistence, optional Redis caching, structured tracing, graceful shutdown, and a production-ready Dockerfile.

## Task Context
- Agent owner: rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1

## Implementation Plan
1. **Project scaffold**: `cargo init notifycore`. Add dependencies in Cargo.toml: axum 0.7, tokio 1 (full features), sqlx 0.7 (postgres, runtime-tokio, tls-rustls, migrate), serde 1 + serde_json, uuid (v4, serde), chrono (serde), tracing 0.1, tracing-subscriber (json, env-filter), redis 0.25 (optional feature), tower-http (trace, cors).
2. **Data models**: Implement `Notification`, `Channel`, `Priority`, `NotificationStatus`, `CreateNotificationRequest`, `ListNotificationsQuery` as specified in the PRD. Derive `sqlx::Type` for enums or use string mapping.
3. **Database migrations**: Create `migrations/001_create_notifications.sql` with table: `id UUID PRIMARY KEY, channel VARCHAR NOT NULL, priority VARCHAR NOT NULL, title TEXT NOT NULL, body TEXT NOT NULL, status VARCHAR NOT NULL DEFAULT 'pending', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`. Add index on `(status, created_at DESC)` for list queries.
4. **App state**: Struct holding `PgPool` and optional `redis::Client`. Initialize from env vars `DATABASE_URL`, `REDIS_URL`, `PORT` (default 8080). Run sqlx migrations on startup.
5. **Endpoints**:
   a. `POST /api/v1/notifications` — validate request (title non-empty, body non-empty), insert row with status=Pending, return 201 with Notification JSON.
   b. `GET /api/v1/notifications/:id` — query by UUID, return 200 or 404 `{"error": "not found"}`.
   c. `GET /api/v1/notifications` — accept `page` (default 1), `per_page` (default 20, max 100), optional `status` filter. Return `{"data": [...], "page": N, "per_page": N, "total": N}`.
   d. `DELETE /api/v1/notifications/:id` — if status=Pending, set status=Cancelled and updated_at=NOW(), return 200. If not pending, return 409 `{"error": "only pending notifications can be cancelled"}`. If not found, 404.
   e. `GET /health` — check pg pool `sqlx::query("SELECT 1")`, return `{"status": "healthy", "database": "connected"}` 200 or `{"status": "degraded", ...}` 503.
6. **Redis caching** (optional path): On GET by ID, check Redis first (`notification:{id}`). On write/update, invalidate. If Redis unavailable, fall through to Postgres silently.
7. **Structured logging**: `tracing_subscriber` with JSON formatter, env filter from `RUST_LOG`.
8. **Graceful shutdown**: `tokio::signal::ctrl_c()` with Axum's `with_graceful_shutdown`.
9. **Error handling**: Implement `IntoResponse` for a custom `AppError` enum (NotFound, Validation, Conflict, Internal) that returns appropriate status codes and JSON bodies.
10. **Unit tests**: In `src/` modules, test validation logic, enum serialization, pagination math.
11. **Integration tests**: In `tests/`, use sqlx test fixtures or testcontainers-rs to spin up Postgres. Test all 5 endpoints end-to-end including error cases (404, 409, 422).
12. **Dockerfile**: Multi-stage build — `rust:1.75-slim` builder with `cargo build --release`, then `debian:bookworm-slim` runtime. Copy binary, expose PORT, set ENTRYPOINT. Ensure image < 100MB.
13. **Kubernetes manifest**: Deployment YAML referencing `notifycore-infra-endpoints` ConfigMap via `envFrom`, liveness probe on `/health`, readiness probe on `/health`, resource requests (64Mi/100m) and limits (256Mi/500m).

## Acceptance Criteria
1. `cargo test` passes all unit and integration tests (minimum 15 test cases covering: valid notification creation, creation with empty title returns 422, get by valid ID returns 200, get by unknown ID returns 404, list with default pagination returns correct structure, list with status filter returns only matching, list with per_page > 100 clamps to 100, cancel pending notification returns 200 with status=cancelled, cancel non-pending returns 409, cancel unknown returns 404, health check returns 200 with database=connected, enum serialization to lowercase JSON, pagination offset calculation, graceful shutdown signal handling). 2. `docker build .` completes successfully and image size < 100MB. 3. Running the container with valid DATABASE_URL against a test Postgres instance: POST /api/v1/notifications returns 201 with a valid UUID within 50ms. 4. GET /health returns `{"status": "healthy"}` with HTTP 200. 5. `cargo clippy -- -D warnings` passes with zero warnings.

## Subtasks
- Project scaffold with Cargo.toml, app state, and configuration: Initialize the Rust project with all dependencies, create the application state struct, configuration loading from environment variables, and the main entrypoint with Axum server setup, structured logging, and graceful shutdown.
- Data models, enum definitions, and database migration: Define all data models (Notification, Channel, Priority, NotificationStatus, CreateNotificationRequest, ListNotificationsQuery) with serde and sqlx derivations, and create the database migration SQL.
- Custom AppError enum and error handling middleware: Implement the custom `AppError` enum with variants NotFound, Validation, Conflict, and Internal, implementing `IntoResponse` to return appropriate HTTP status codes and JSON error bodies.
- Implement POST and GET /api/v1/notifications/:id endpoints: Implement the POST /api/v1/notifications endpoint for creating notifications with validation, and the GET /api/v1/notifications/:id endpoint for retrieving a single notification by UUID.
- Implement GET /api/v1/notifications (list) and DELETE /api/v1/notifications/:id (cancel) endpoints: Implement the paginated list endpoint with optional status filtering and the cancel endpoint with conflict detection for non-pending notifications.
- Implement health check endpoint and optional Redis caching layer: Implement the GET /health endpoint with database connectivity check, and the optional Redis caching layer for GET by ID with silent fallthrough on Redis failure.
- Unit tests for validation logic, enum serialization, and pagination math: Write unit tests within src/ modules covering validation logic, enum serialization/deserialization to lowercase JSON, and pagination offset calculation.
- Integration tests for all five endpoints including error cases: Write integration tests in the `tests/` directory using testcontainers-rs or sqlx test fixtures to test all five endpoints end-to-end, including error paths (404, 409, 422).
- Multi-stage Dockerfile and Kubernetes Deployment manifest: Create a multi-stage Dockerfile producing an image under 100MB and a Kubernetes Deployment manifest referencing the notifycore-infra-endpoints ConfigMap with health probes and resource limits.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.