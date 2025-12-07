# CODEX Task Generation Results

**Model:** gpt-5.1-codex
**Duration:** 18.78s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** api, error, database, test, project, jwt, task, auth, docker

---

## Task 1: Bootstrap Axum service and environment

**Status:** pending | **Priority:** high

### Description

Scaffold the Rust workspace with Axum + Tokio runtime, shared configuration, and PostgreSQL connectivity to create a stable foundation for later features.

### Implementation Details

• Initialize Cargo binary (edition 2021) and add deps: axum=0.7, tokio={version="1.35", features=["full"]}, sqlx={version="0.7", features=["runtime-tokio", "postgres", "macros"]}, serde=1.0, serde_json=1.0, tracing=0.1, tracing-subscriber=0.3, tower-http=0.5, anyhow=1.0, thiserror=1.0, dotenvy=0.15.
• Wire config loader (e.g., Settings struct) reading .env for DB URL, JWT secret, refresh TTL using envy or manual env parsing.
• Provide `main.rs` skeleton: init tracing_subscriber, build PgPoolOptions::new().max_connections(5).connect_lazy_with options, attach to Axum state, mount readiness probe.
• Add Cargo features for `sqlx offline` builds and set up `sqlx-data.json` via `cargo sqlx prepare` when write access available.
• Document Makefile/dev scripts for `cargo watch -x run` and database migration hooks (`sqlx migrate run`).

### Test Strategy

Run `cargo fmt && cargo clippy --all-targets -- -D warnings` and `cargo test` to ensure clean build; smoke-test `cargo run` confirms server boots and `/healthz` returns 200 via curl.

---

## Task 2: Implement authentication domain with JWT

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Deliver secure login/logout/refresh flows using password hashing and JWT/refresh tokens while exposing the required endpoints.

### Implementation Details

• Create `auth` module with user entity matching DB table users(id UUID, email TEXT UNIQUE, password_hash TEXT, refresh_token TEXT, refresh_expires_at TIMESTAMPTZ).
• Use argon2=0.5 with rand_chacha salts for password hashing; enforce OWASP iteration counts.
• Introduce jsonwebtoken=9 for signing (HS256) with configurable `JWT_SECRET`, `JWT_EXP_MINUTES`, `REFRESH_EXP_HOURS`.
• Handlers: POST /auth/login accepts {email,password}, verifies user via sqlx::query_as!, issues access+refresh tokens, persists hashed refresh token.
`POST /auth/logout` invalidates stored refresh entry; `POST /auth/refresh` validates refresh token (check expiry + hash match) and rotates tokens.
• Provide `AuthLayer` middleware using `tower_http::auth::RequireAuthorizationLayer` with custom extractor reading `Authorization: Bearer` and injecting `Claims` into request extensions.
• Add unit tests covering password hashing, token issue/validation, refresh rotation, plus integration test hitting login with mocked DB using `sqlx::test-fixture`.

### Test Strategy

Mock user repo to unit-test hashing/token creation logic; run integration tests with test DB to ensure login/logout/refresh flows return correct HTTP status and token payloads; include property test verifying expired refresh tokens fail.

---

## Task 3: Design task schema and persistence layer

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2

### Description

Define database schema for tasks with statuses/priorities and implement repository functions that abstract PostgreSQL access.

### Implementation Details

• Write sqlx migrations: `tasks` table (id UUID PK, owner_id UUID FK users, title TEXT NOT NULL, description TEXT, status TEXT CHECK IN ('pending','in-progress','done'), priority TEXT CHECK IN ('low','medium','high'), due_date TIMESTAMPTZ NULL, created_at TIMESTAMPTZ DEFAULT now(), updated_at TIMESTAMPTZ DEFAULT now()). Add triggers or use `UPDATED AT` via `GENERATED ALWAYS` for timestamp maintenance.
• Implement `Task` struct deriving Serialize/Deserialize + sqlx::FromRow; create `TaskRepository` with methods `create_task`, `get_task`, `list_tasks`, `update_task`, `delete_task`, `list_by_status` using sqlx query macros and `&PgPool`.
• Expose service layer for business rules (status transitions, priority validation), returning domain errors via thiserror enums mapped to Axum responses.
• Provide repository tests using `#[sqlx::test(migrated = "./migrations")]` to ensure CRUD functions behave and constraints enforce statuses/priorities.

### Test Strategy

Execute sqlx migration tests; repository unit/integration tests check inserts/updates respect enum constraints and rows scoped by owner_id; use `cargo sqlx prepare --check` to ensure query validity.

---

## Task 4: Build task management REST endpoints

**Status:** pending | **Priority:** high

**Dependencies:** 2, 3

### Description

Expose authenticated Axum routes that leverage the repository/service to deliver CRUD with validation and pagination filters.

### Implementation Details

• Define request/response DTOs (CreateTaskRequest, UpdateTaskRequest, TaskResponse) with serde validation; optionally use `validator` crate for field length/due_date in future.
• Router: under `/api/tasks`, register POST (create), GET /:id, GET / (list with query params `status`, `priority`, `page`, `page_size`), PUT /:id, DELETE /:id. All routes use `AuthLayer` to ensure owner scoping (claims.sub -> owner_id).
• Handler pseudo:
```rust
pub async fn create_task(State(ctx): State<AppState>, AuthUser(user): AuthUser, Json(payload): Json<CreateTaskRequest>) -> Result<Json<TaskResponse>, ApiError> {
    let task = ctx.services.tasks.create(user.id, payload).await?;
    Ok(Json(TaskResponse::from(task)))
}
```
• Implement pagination using `LIMIT/OFFSET` and total count header (`x-total-count`). Map domain errors (NotFound, Forbidden, Validation) to appropriate HTTP codes.
• Add tracing spans per handler (`info_span!("create_task", user_id=%user.id)`), propagate errors via anyhow -> ApiError conversions.


### Test Strategy

Use Axum integration tests via `axum::Router::into_make_service()` + hyper client to validate each endpoint (auth header required, status filtering works, pagination metadata present). Add negative tests for invalid status transitions and unauthorized access.

---

## Task 5: End-to-end validation, observability, and hardening

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2, 3, 4

### Description

Ensure the API meets production readiness with test coverage, security checks, and operational visibility.

### Implementation Details

• Compose Docker-based test stack (PostgreSQL 15 + service) for CI; include scripts to run migrations automatically before tests.
• Implement `tracing` + `tower_http::trace::TraceLayer` with JSON logs and request IDs, integrate `metrics` crate if required.
• Add rate limiting (e.g., `tower_http::limit::RequestBodyLimitLayer`) for auth routes and enforce secure cookie settings if refresh tokens returned via cookies.
• Ensure JWT secret sourced from env + validate length at startup; add security headers middleware (`tower_http::set_header::SetResponseHeaderLayer`).
• Expand test suite: E2E scenario covering login -> create task -> update -> list -> logout; fuzz unauthorized access cases; run load test script (k6 or bombardier) to confirm concurrency under Tokio runtime defaults.
• Document deployment checklist (DB migrations, env vars, health endpoints) in README.

### Test Strategy

CI pipeline runs `cargo test --all`, `cargo clippy`, `cargo fmt`, integration/E2E tests using docker-compose; monitor logs to confirm tracing spans; run security scan (cargo audit) and ensure rate-limit + auth paths behave under load tests (expect <1% errors at target RPS).

---

