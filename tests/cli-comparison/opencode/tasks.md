# OPENCODE Task Generation Results

**Model:** anthropic/claude-opus-4-5
**Duration:** 159.53s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** api, error, auth, task, project, jwt, database

---

## Task 1: Setup Rust project with Axum and dependencies

**Status:** pending | **Priority:** high

### Description

Initialize the Rust project structure with all required dependencies, configuration, and database connection pool setup for the Task Manager API.

### Implementation Details

1. Create new Rust project with `cargo new task-manager-api`

2. Configure Cargo.toml with dependencies:
```toml
[dependencies]
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tokio = { version = "1.44", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
jsonwebtoken = "10.2"
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono", "migrate"] }
argon2 = { version = "0.5", features = ["password-hash"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
thiserror = "2.0"
```

3. Create project structure:
```
src/
├── main.rs           # Entry point
├── config.rs         # Config loading
├── error.rs          # AppError type
├── state.rs          # AppState with PgPool
├── db/
│   ├── mod.rs
│   └── pool.rs       # Connection pool
├── auth/
│   └── mod.rs
├── routes/
│   └── mod.rs
└── models/
    └── mod.rs
migrations/
```

4. Implement database pool setup in `src/db/pool.rs`:
```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}
```

5. Create AppState in `src/state.rs`:
```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
```

6. Setup basic main.rs with tracing, dotenvy, and health endpoint

### Test Strategy

1. Run `cargo build` to verify all dependencies resolve
2. Run `cargo clippy` for lint checks
3. Create .env with DATABASE_URL pointing to test PostgreSQL
4. Start server and verify GET /health returns 200 OK
5. Verify tracing output appears in console
6. Test database connection by checking pool connects successfully

---

## Task 2: Create database schema and migrations

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Design and implement PostgreSQL database schema for users and tasks tables with proper indexes and constraints using SQLx migrations.

### Implementation Details

1. Create migration file `migrations/20241207000001_initial_schema.sql`:
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table for authentication
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Tasks table
CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'done');
CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status task_status NOT NULL DEFAULT 'pending',
    priority task_priority NOT NULL DEFAULT 'medium',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_user_id ON tasks(user_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_priority ON tasks(priority);

-- Refresh tokens for JWT refresh capability
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
```

2. Add migration runner to main.rs:
```rust
sqlx::migrate!("./migrations").run(&pool).await?;
```

3. Create Rust models in `src/models/`:
```rust
// src/models/user.rs
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

// src/models/task.rs
#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "task_priority", rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Test Strategy

1. Run `sqlx migrate run` against test database
2. Verify all tables created with `\dt` in psql
3. Verify enum types with `\dT+`
4. Test foreign key constraint by attempting orphan task insert
5. Verify indexes exist with `\di`
6. Run `sqlx migrate info` to confirm migration status
7. Test rollback capability if needed

---

## Task 3: Implement JWT authentication system

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2

### Description

Build complete JWT authentication including password hashing with Argon2, token generation/validation, login/register endpoints, and token refresh capability.

### Implementation Details

1. Create password hashing module `src/auth/password.rs`:
```rust
use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}
```

2. Create JWT module `src/auth/jwt.rs`:
```rust
use std::sync::LazyLock;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET required");
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn new(user_id: Uuid, email: &str, expires_in_secs: i64) -> Self {
        let now = chrono::Utc::now();
        Self {
            sub: user_id.to_string(),
            email: email.to_owned(),
            iat: now.timestamp() as usize,
            exp: (now + chrono::Duration::seconds(expires_in_secs)).timestamp() as usize,
        }
    }
}

pub fn create_access_token(claims: &Claims) -> Result<String, AuthError> { ... }
pub fn create_refresh_token(claims: &Claims) -> Result<String, AuthError> { ... }
pub fn verify_token(token: &str) -> Result<Claims, AuthError> { ... }
```

3. Implement FromRequestParts extractor for Claims (auto-auth middleware)

4. Create auth routes `src/routes/auth.rs`:
- POST /api/auth/register - Create user, hash password, return tokens
- POST /api/auth/login - Verify credentials, return access + refresh tokens
- POST /api/auth/refresh - Validate refresh token, issue new access token
- POST /api/auth/logout - Invalidate refresh token in DB

5. Token expiry strategy:
- Access token: 15 minutes (900 seconds)
- Refresh token: 7 days (604800 seconds)

6. Store refresh token hash in DB for revocation support

### Test Strategy

1. Unit test password hashing: hash then verify returns true
2. Unit test password verification: wrong password returns false
3. Unit test JWT creation and verification round-trip
4. Unit test expired token rejection
5. Integration test: POST /api/auth/register creates user, returns tokens
6. Integration test: POST /api/auth/login with valid creds returns 200
7. Integration test: POST /api/auth/login with invalid creds returns 401
8. Integration test: POST /api/auth/refresh with valid refresh token returns new access token
9. Integration test: Protected route without token returns 401
10. Integration test: Protected route with expired token returns 401
11. Verify refresh token is stored hashed in database

---

## Task 4: Implement Task CRUD endpoints

**Status:** pending | **Priority:** high

**Dependencies:** 2, 3

### Description

Create REST API endpoints for complete task management including create, read (single and list), update, and delete operations with proper authorization.

### Implementation Details

1. Create request/response DTOs in `src/models/task.rs`:
```rust
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,  // defaults to medium
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

2. Implement task routes in `src/routes/tasks.rs`:
```rust
// All routes require authentication via Claims extractor

// POST /api/tasks - Create new task
pub async fn create_task(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = sqlx::query_as!(
        Task,
        r#"INSERT INTO tasks (user_id, title, description, priority)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, title, description, 
                     status as "status: TaskStatus", 
                     priority as "priority: TaskPriority",
                     created_at, updated_at"#,
        claims.user_id(), payload.title, payload.description,
        payload.priority.unwrap_or(TaskPriority::Medium) as TaskPriority
    ).fetch_one(&state.db).await?;
    Ok(Json(task.into()))
}

// GET /api/tasks - List user's tasks (with optional filters)
// GET /api/tasks/:id - Get single task (verify ownership)
// PUT /api/tasks/:id - Update task (verify ownership)
// DELETE /api/tasks/:id - Delete task (verify ownership)
```

3. Add query parameters for list endpoint:
```rust
#[derive(Debug, Deserialize)]
pub struct TaskFilters {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub limit: Option<i64>,   // default 50
    pub offset: Option<i64>,  // default 0
}
```

4. Ensure all task operations verify user_id matches authenticated user

5. Register routes in main router:
```rust
Router::new()
    .route("/api/tasks", post(create_task).get(list_tasks))
    .route("/api/tasks/:id", get(get_task).put(update_task).delete(delete_task))
```

### Test Strategy

1. Integration test: POST /api/tasks without auth returns 401
2. Integration test: POST /api/tasks with valid data creates task, returns 201
3. Integration test: GET /api/tasks returns only authenticated user's tasks
4. Integration test: GET /api/tasks/:id returns task if owned by user
5. Integration test: GET /api/tasks/:id returns 404 if not owned by user (not 403 to avoid enumeration)
6. Integration test: PUT /api/tasks/:id updates task fields correctly
7. Integration test: PUT /api/tasks/:id with status transition works
8. Integration test: DELETE /api/tasks/:id removes task, returns 204
9. Integration test: Filter by status returns correct subset
10. Integration test: Filter by priority returns correct subset
11. Integration test: Pagination with limit/offset works correctly
12. Verify updated_at timestamp changes on update

---

## Task 5: Add API middleware, error handling, and documentation

**Status:** pending | **Priority:** medium

**Dependencies:** 3, 4

### Description

Implement production-ready middleware stack including CORS, request tracing, rate limiting, and comprehensive error handling with consistent API responses.

### Implementation Details

1. Create comprehensive error type in `src/error.rs`:
```rust
use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication required")]
    Unauthorized,
    #[error("Resource not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

2. Configure middleware stack in main.rs:
```rust
use tower_http::{cors::CorsLayer, trace::TraceLayer, compression::CompressionLayer};

let app = Router::new()
    .nest("/api", api_routes())
    .with_state(state)
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]));
```

3. Add request ID middleware for tracing:
```rust
use uuid::Uuid;
use axum::middleware;

async fn add_request_id(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(RequestId(request_id.clone()));
    let mut response = next.run(req).await;
    response.headers_mut().insert("x-request-id", request_id.parse().unwrap());
    response
}
```

4. Add graceful shutdown handling:
```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("Shutdown signal received");
}
```

5. Create OpenAPI documentation (optional, using utoipa):
- Document all endpoints with request/response schemas
- Add Swagger UI at /swagger-ui

6. Add health and readiness endpoints:
- GET /health - Basic liveness
- GET /ready - Check DB connection

### Test Strategy

1. Test CORS: OPTIONS request returns correct headers
2. Test CORS: Request from allowed origin succeeds
3. Test error responses: All errors return consistent JSON format
4. Test 404: Unknown routes return proper JSON error, not HTML
5. Test request ID: Response includes x-request-id header
6. Test tracing: Requests logged with timing and status
7. Test graceful shutdown: In-flight requests complete before exit
8. Test /health returns 200 immediately
9. Test /ready returns 503 when DB unavailable
10. Load test: Verify no memory leaks under sustained traffic
11. Test compression: Large responses are gzip compressed

---

