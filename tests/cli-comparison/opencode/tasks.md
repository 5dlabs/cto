# OPENCODE Task Generation Results

**Model:** anthropic/claude-opus-4-5
**Duration:** 175.07s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** error, api, task, project, database, jwt, docker, auth

---

## Task 1: Setup Rust project with Axum and database infrastructure

**Status:** pending | **Priority:** high

### Description

Initialize the Rust project with Axum framework, PostgreSQL connection via SQLx, and establish the foundational project structure with proper error handling patterns.

### Implementation Details

1. Initialize Cargo project:
   cargo new task-manager-api && cd task-manager-api

2. Configure Cargo.toml with dependencies:
   ```toml
   [dependencies]
   axum = "0.8"
   tokio = { version = "1.40", features = ["full"] }
   tower-http = { version = "0.5", features = ["trace", "cors"] }
   sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls-ring", "postgres", "uuid", "chrono"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   thiserror = "2.0"
   anyhow = "1.0"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   uuid = { version = "1.0", features = ["v4", "serde"] }
   chrono = { version = "0.4", features = ["serde"] }
   dotenvy = "0.15"
   ```

3. Create project structure:
   src/
   ├── main.rs           # Entry point, server bootstrap
   ├── config.rs         # Environment configuration
   ├── db/
   │   ├── mod.rs
   │   └── pool.rs       # PgPool setup with connection limits
   ├── api/
   │   ├── mod.rs
   │   ├── routes.rs     # Router composition
   │   └── handlers/     # Request handlers
   ├── domain/
   │   ├── mod.rs
   │   └── errors.rs     # AppError with IntoResponse impl
   └── services/

4. Implement AppError enum with thiserror:
   ```rust
   #[derive(Error, Debug)]
   pub enum AppError {
       #[error("Not found")] NotFound,
       #[error("Unauthorized")] Unauthorized,
       #[error("Bad request: {0}")] BadRequest(String),
       #[error("Database error")] Database(#[from] sqlx::Error),
       #[error("Internal error")] Internal(#[from] anyhow::Error),
   }
   ```

5. Setup PgPool with connection pooling:
   ```rust
   PgPoolOptions::new()
       .max_connections(20)
       .min_connections(5)
       .acquire_timeout(Duration::from_secs(3))
       .connect(&database_url).await
   ```

6. Create AppState struct holding PgPool and config

7. Bootstrap main.rs with tracing, router, and graceful shutdown

### Test Strategy

1. Verify `cargo build` succeeds with no warnings
2. Verify `cargo clippy -- -D warnings` passes
3. Write integration test that app starts and /health endpoint returns 200 OK
4. Verify database connection succeeds with valid DATABASE_URL
5. Test that invalid DATABASE_URL produces clear error message

---

## Task 2: Implement database schema and migrations for users and tasks

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Create PostgreSQL migrations for users table (authentication) and tasks table (task management) using sqlx-cli, establishing the data model foundation.

### Implementation Details

1. Install sqlx-cli:
   cargo install sqlx-cli --no-default-features --features postgres

2. Create migrations directory and .env file:
   DATABASE_URL=postgres://user:pass@localhost:5432/task_manager

3. Create users migration:
   sqlx migrate add create_users_table
   ```sql
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   
   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
       email VARCHAR(255) NOT NULL UNIQUE,
       password_hash VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   
   CREATE INDEX idx_users_email ON users(email);
   ```

4. Create tasks migration:
   sqlx migrate add create_tasks_table
   ```sql
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
   ```

5. Create refresh_tokens table for JWT refresh:
   sqlx migrate add create_refresh_tokens_table
   ```sql
   CREATE TABLE refresh_tokens (
       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
       user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
       token_hash VARCHAR(255) NOT NULL UNIQUE,
       expires_at TIMESTAMPTZ NOT NULL,
       revoked BOOLEAN NOT NULL DEFAULT FALSE,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   
   CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
   ```

6. Run migrations: sqlx migrate run

7. Create Rust domain models:
   ```rust
   #[derive(Debug, sqlx::FromRow, Serialize)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       #[serde(skip)] pub password_hash: String,
       pub created_at: DateTime<Utc>,
   }
   
   #[derive(Debug, sqlx::Type, Serialize, Deserialize)]
   #[sqlx(type_name = "task_status", rename_all = "snake_case")]
   pub enum TaskStatus { Pending, InProgress, Done }
   
   #[derive(Debug, sqlx::Type, Serialize, Deserialize)]
   #[sqlx(type_name = "task_priority", rename_all = "snake_case")]
   pub enum TaskPriority { Low, Medium, High }
   ```

### Test Strategy

1. Run `sqlx migrate run` and verify all migrations apply cleanly
2. Run `sqlx migrate revert` and `sqlx migrate run` to test idempotency
3. Verify foreign key constraint by attempting to insert task with invalid user_id
4. Verify enum types work correctly with test inserts
5. Test unique constraint on users.email
6. Verify indexes exist using pg_indexes query

---

## Task 3: Implement JWT authentication system with login, register, and token refresh

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2

### Description

Build complete authentication flow including user registration, login with JWT access token generation, and secure token refresh mechanism with rotation.

### Implementation Details

1. Add auth dependencies to Cargo.toml:
   ```toml
   jsonwebtoken = "9.3"
   argon2 = "0.5"
   rand = "0.8"
   ```

2. Create auth service (src/services/auth_service.rs):
   ```rust
   pub struct AuthService { pool: PgPool, jwt_secret: String }
   
   impl AuthService {
       pub async fn register(&self, email: &str, password: &str) -> Result<User, AppError>;
       pub async fn login(&self, email: &str, password: &str) -> Result<TokenPair, AppError>;
       pub async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AppError>;
       pub async fn logout(&self, user_id: Uuid) -> Result<(), AppError>;
   }
   ```

3. Implement password hashing with Argon2:
   ```rust
   use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::SaltString};
   
   fn hash_password(password: &str) -> Result<String, AppError> {
       let salt = SaltString::generate(&mut rand::thread_rng());
       Ok(Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string())
   }
   ```

4. Implement JWT token generation:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct Claims {
       pub sub: String,  // user_id
       pub exp: usize,   // 15 minutes for access token
       pub iat: usize,
   }
   
   pub struct TokenPair {
       pub access_token: String,
       pub refresh_token: String,
       pub expires_in: i64,
   }
   ```
   - Access token: 15 minute expiry, HS256 algorithm
   - Refresh token: 7 day expiry, stored hashed in DB

5. Create auth middleware extractor:
   ```rust
   pub struct AuthUser(pub Uuid);
   
   impl<S> FromRequestParts<S> for AuthUser {
       // Extract and validate Bearer token from Authorization header
       // Decode JWT, validate expiry, return user_id
   }
   ```

6. Create auth handlers (src/api/handlers/auth.rs):
   ```rust
   // POST /api/auth/register - CreateUserRequest { email, password }
   // POST /api/auth/login - LoginRequest { email, password }
   // POST /api/auth/refresh - RefreshRequest { refresh_token }
   // POST /api/auth/logout - requires AuthUser
   ```

7. Implement refresh token rotation:
   - On refresh, invalidate old token and issue new pair
   - Hash refresh tokens before storing (SHA-256)
   - Check revoked flag before accepting refresh

8. Wire routes in router:
   ```rust
   Router::new()
       .route("/api/auth/register", post(register))
       .route("/api/auth/login", post(login))
       .route("/api/auth/refresh", post(refresh))
       .route("/api/auth/logout", post(logout))
   ```

### Test Strategy

1. Test registration creates user with hashed password (not plaintext)
2. Test login returns valid JWT that decodes correctly
3. Test login with wrong password returns 401
4. Test duplicate email registration returns 400
5. Test expired access token returns 401
6. Test refresh token generates new valid access token
7. Test refresh token rotation (old refresh token becomes invalid)
8. Test logout invalidates refresh tokens
9. Test AuthUser extractor rejects invalid/missing tokens
10. Test password meets minimum requirements if implemented

---

## Task 4: Implement Task CRUD operations with user authorization

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2, 3

### Description

Build complete task management API with Create, Read, Update, Delete operations, ensuring users can only access their own tasks.

### Implementation Details

1. Create task repository (src/db/repositories/task_repo.rs):
   ```rust
   pub struct TaskRepository { pool: PgPool }
   
   impl TaskRepository {
       pub async fn create(&self, user_id: Uuid, req: CreateTaskRequest) -> Result<Task, AppError>;
       pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Task>, AppError>;
       pub async fn find_all_by_user(&self, user_id: Uuid, filters: TaskFilters) -> Result<Vec<Task>, AppError>;
       pub async fn update(&self, id: Uuid, user_id: Uuid, req: UpdateTaskRequest) -> Result<Task, AppError>;
       pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError>;
   }
   ```

2. Define request/response DTOs:
   ```rust
   #[derive(Deserialize)]
   pub struct CreateTaskRequest {
       pub title: String,
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,  // defaults to medium
   }
   
   #[derive(Deserialize)]
   pub struct UpdateTaskRequest {
       pub title: Option<String>,
       pub description: Option<String>,
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Deserialize)]
   pub struct TaskFilters {
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
   }
   ```

3. Implement handlers (src/api/handlers/tasks.rs):
   ```rust
   // All handlers require AuthUser extractor
   
   // POST /api/tasks
   async fn create_task(
       State(state): State<Arc<AppState>>,
       AuthUser(user_id): AuthUser,
       Json(req): Json<CreateTaskRequest>,
   ) -> Result<impl IntoResponse, AppError> {
       let task = state.task_repo.create(user_id, req).await?;
       Ok((StatusCode::CREATED, Json(task)))
   }
   
   // GET /api/tasks - list with optional status/priority filters
   // GET /api/tasks/:id
   // PUT /api/tasks/:id
   // DELETE /api/tasks/:id
   ```

4. Implement SQL queries with compile-time checking:
   ```rust
   // Create
   sqlx::query_as!(
       Task,
       r#"INSERT INTO tasks (user_id, title, description, priority)
          VALUES ($1, $2, $3, $4)
          RETURNING id, user_id, title, description, 
                    status as "status: TaskStatus", 
                    priority as "priority: TaskPriority",
                    created_at, updated_at"#,
       user_id, req.title, req.description, 
       req.priority.unwrap_or(TaskPriority::Medium) as TaskPriority
   )
   
   // Update with dynamic fields
   sqlx::query_as!(
       Task,
       r#"UPDATE tasks SET 
          title = COALESCE($3, title),
          description = COALESCE($4, description),
          status = COALESCE($5, status),
          priority = COALESCE($6, priority),
          updated_at = NOW()
          WHERE id = $1 AND user_id = $2
          RETURNING ..."#,
       id, user_id, req.title, req.description, 
       req.status as Option<TaskStatus>, req.priority as Option<TaskPriority>
   )
   ```

5. Wire routes (protected by auth):
   ```rust
   Router::new()
       .route("/api/tasks", post(create_task).get(list_tasks))
       .route("/api/tasks/:id", get(get_task).put(update_task).delete(delete_task))
   ```

6. Ensure authorization: All queries include `user_id` in WHERE clause

### Test Strategy

1. Test create task returns 201 with valid task object
2. Test create task without auth returns 401
3. Test list tasks only returns current user's tasks
4. Test get task by ID returns 404 for other user's task
5. Test update task modifies only specified fields
6. Test update task status transitions (pending -> in_progress -> done)
7. Test delete task returns 204 and task is removed
8. Test delete other user's task returns 404 (not 403, to avoid leaking existence)
9. Test list tasks with status filter
10. Test list tasks with priority filter
11. Test input validation (empty title rejected)

---

## Task 5: Add API validation, documentation, and production hardening

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2, 3, 4

### Description

Implement request validation, add OpenAPI documentation, configure CORS, rate limiting, and prepare the API for production deployment.

### Implementation Details

1. Add validation dependencies:
   ```toml
   validator = { version = "0.18", features = ["derive"] }
   utoipa = { version = "5.0", features = ["axum_extras", "uuid", "chrono"] }
   utoipa-swagger-ui = { version = "8.0", features = ["axum"] }
   ```

2. Add validation to request DTOs:
   ```rust
   use validator::Validate;
   
   #[derive(Deserialize, Validate)]
   pub struct CreateTaskRequest {
       #[validate(length(min = 1, max = 255))]
       pub title: String,
       #[validate(length(max = 5000))]
       pub description: Option<String>,
   }
   
   #[derive(Deserialize, Validate)]
   pub struct RegisterRequest {
       #[validate(email)]
       pub email: String,
       #[validate(length(min = 8, max = 128))]
       pub password: String,
   }
   ```

3. Create validation extractor:
   ```rust
   pub struct ValidatedJson<T>(pub T);
   
   impl<S, T> FromRequest<S> for ValidatedJson<T>
   where T: DeserializeOwned + Validate {
       // Deserialize then call .validate()
       // Return 400 with validation errors if invalid
   }
   ```

4. Add OpenAPI documentation with utoipa:
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(create_task, list_tasks, get_task, update_task, delete_task,
             register, login, refresh, logout),
       components(schemas(Task, CreateTaskRequest, UpdateTaskRequest, ...)),
       tags((name = "tasks"), (name = "auth"))
   )]
   struct ApiDoc;
   
   // Add swagger UI route
   Router::new()
       .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
   ```

5. Configure CORS:
   ```rust
   use tower_http::cors::{CorsLayer, Any};
   
   let cors = CorsLayer::new()
       .allow_origin(Any)  // Configure for production
       .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
       .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
   ```

6. Add rate limiting (optional, using tower):
   ```rust
   use tower::limit::RateLimitLayer;
   // Or use governor crate for more control
   ```

7. Add request tracing and logging:
   ```rust
   use tower_http::trace::TraceLayer;
   
   .layer(TraceLayer::new_for_http()
       .make_span_with(|req| tracing::info_span!("request", method = %req.method(), uri = %req.uri()))
   )
   ```

8. Configure graceful shutdown:
   ```rust
   axum::serve(listener, app)
       .with_graceful_shutdown(shutdown_signal())
       .await
   ```

9. Add health check endpoint:
   ```rust
   // GET /health - returns 200 if DB connection is healthy
   async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
       match sqlx::query("SELECT 1").execute(&state.pool).await {
           Ok(_) => (StatusCode::OK, Json(json!({"status": "healthy"}))),
           Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"status": "unhealthy"}))),
       }
   }
   ```

10. Environment configuration:
    - DATABASE_URL, JWT_SECRET (required)
    - HOST, PORT, LOG_LEVEL (optional with defaults)

### Test Strategy

1. Test validation rejects empty task title
2. Test validation rejects invalid email format
3. Test validation rejects password < 8 characters
4. Test validation error response includes field names and messages
5. Test OpenAPI spec is valid JSON at /api-docs/openapi.json
6. Test Swagger UI loads at /swagger-ui
7. Test CORS headers present in responses
8. Test health endpoint returns healthy with valid DB
9. Test health endpoint returns unhealthy with DB down
10. Test graceful shutdown completes in-flight requests
11. Test request tracing logs include request_id
12. Integration test: full flow from register -> login -> create task -> list tasks

---

