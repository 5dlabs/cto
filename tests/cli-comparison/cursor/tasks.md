# CURSOR Task Generation Results

**Model:** opus-4.5-thinking
**Duration:** 123.47s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** task, error, api, database, project, auth, jwt

---

## Task 1: Setup project foundation with database layer

**Status:** pending | **Priority:** high

### Description

Initialize the Task Manager API crate with required dependencies, database schema, and SQLx migrations. This establishes the core data layer that all other features depend on.

### Implementation Details

1. Create new crate `crates/taskapi/` with Cargo.toml:
   - axum = { workspace = true }
   - tokio = { workspace = true }
   - sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "migrate"] }
   - serde = { workspace = true }
   - thiserror = { workspace = true }
   - tracing = { workspace = true }
   - uuid = { workspace = true }
   - chrono = { workspace = true }

2. Create database schema in `migrations/001_initial.sql`:
   ```sql
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   
   CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'done');
   CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');
   
   CREATE TABLE users (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     email VARCHAR(255) UNIQUE NOT NULL,
     password_hash VARCHAR(255) NOT NULL,
     created_at TIMESTAMPTZ DEFAULT NOW(),
     updated_at TIMESTAMPTZ DEFAULT NOW()
   );
   
   CREATE TABLE tasks (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
     title VARCHAR(255) NOT NULL,
     description TEXT,
     status task_status DEFAULT 'pending',
     priority task_priority DEFAULT 'medium',
     created_at TIMESTAMPTZ DEFAULT NOW(),
     updated_at TIMESTAMPTZ DEFAULT NOW()
   );
   
   CREATE INDEX idx_tasks_user_id ON tasks(user_id);
   CREATE INDEX idx_tasks_status ON tasks(status);
   ```

3. Create models in `src/models/mod.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       #[serde(skip_serializing)]
       pub password_hash: String,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_status", rename_all = "snake_case")]
   pub enum TaskStatus { Pending, InProgress, Done }
   
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_priority", rename_all = "snake_case")]
   pub enum TaskPriority { Low, Medium, High }
   ```

4. Create database connection pool in `src/db.rs` using `PgPoolOptions::new().max_connections(5)`

5. Follow existing codebase pattern: use thiserror for error types in `src/error.rs`

### Test Strategy

1. Verify crate compiles: `cargo build -p taskapi`
2. Run SQLx migrations against test database: `DATABASE_URL=postgres://... sqlx migrate run`
3. Write unit test to verify database connection pool initialization
4. Test model serialization/deserialization with serde_json::to_string and from_str
5. Verify SQLx compile-time query checking works by running `cargo sqlx prepare`

---

## Task 2: Implement JWT authentication system

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Build the authentication layer with JWT token generation, validation, refresh capability, and Axum extractors. This provides the security foundation for protected task endpoints.

### Implementation Details

1. Add auth dependencies to Cargo.toml:
   - jsonwebtoken = "9.3"
   - argon2 = "0.5" (for password hashing, modern and secure)
   - tower-http = { workspace = true }

2. Create auth configuration in `src/auth/config.rs`:
   ```rust
   pub struct AuthConfig {
       pub jwt_secret: String,
       pub access_token_expiry: Duration,  // 15 minutes
       pub refresh_token_expiry: Duration, // 7 days
   }
   ```

3. Implement JWT claims in `src/auth/jwt.rs`:
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Claims {
       pub sub: Uuid,  // user_id
       pub exp: usize, // expiration timestamp
       pub iat: usize, // issued at
       pub token_type: TokenType, // access or refresh
   }
   
   pub fn create_token(user_id: Uuid, config: &AuthConfig, token_type: TokenType) -> Result<String>
   pub fn validate_token(token: &str, secret: &str) -> Result<Claims>
   ```

4. Create Axum extractor in `src/auth/extractor.rs`:
   ```rust
   pub struct AuthUser(pub Uuid);
   
   #[async_trait]
   impl<S> FromRequestParts<S> for AuthUser
   where S: Send + Sync {
       type Rejection = AuthError;
       async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
           // Extract Bearer token from Authorization header
           // Validate JWT and return AuthUser(claims.sub)
       }
   }
   ```

5. Implement password hashing with argon2:
   ```rust
   pub fn hash_password(password: &str) -> Result<String>
   pub fn verify_password(password: &str, hash: &str) -> Result<bool>
   ```

6. Create auth endpoints in `src/auth/handlers.rs`:
   - POST /auth/register - create user with hashed password
   - POST /auth/login - verify credentials, return access + refresh tokens
   - POST /auth/refresh - exchange refresh token for new access token
   - POST /auth/logout - (stateless, client discards token)

7. Follow codebase error pattern: create AuthError enum with thiserror

### Test Strategy

1. Unit test JWT token creation and validation with various expiry scenarios
2. Unit test password hashing: verify hash_password produces valid argon2 hash, verify_password correctly validates
3. Integration test: register user → login → verify token works → access protected endpoint
4. Test token expiration: create expired token, verify rejection
5. Test refresh flow: use refresh token to get new access token
6. Test invalid credentials return 401 Unauthorized
7. Test malformed JWT tokens are rejected

---

## Task 3: Build task CRUD API endpoints

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2

### Description

Implement the core task management REST API with all CRUD operations, filtering, and pagination. Tasks are user-scoped and require authentication.

### Implementation Details

1. Create task repository in `src/tasks/repository.rs`:
   ```rust
   pub struct TaskRepository {
       pool: PgPool,
   }
   
   impl TaskRepository {
       pub async fn create(&self, user_id: Uuid, input: CreateTask) -> Result<Task>
       pub async fn get_by_id(&self, user_id: Uuid, task_id: Uuid) -> Result<Option<Task>>
       pub async fn list(&self, user_id: Uuid, filter: TaskFilter) -> Result<Vec<Task>>
       pub async fn update(&self, user_id: Uuid, task_id: Uuid, input: UpdateTask) -> Result<Task>
       pub async fn delete(&self, user_id: Uuid, task_id: Uuid) -> Result<()>
   }
   ```

2. Define DTOs in `src/tasks/dto.rs`:
   ```rust
   #[derive(Deserialize)]
   pub struct CreateTask {
       pub title: String,
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Deserialize)]
   pub struct UpdateTask {
       pub title: Option<String>,
       pub description: Option<String>,
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Deserialize)]
   pub struct TaskFilter {
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
       pub limit: Option<i64>,
       pub offset: Option<i64>,
   }
   ```

3. Create task handlers in `src/tasks/handlers.rs`:
   ```rust
   pub async fn create_task(
       AuthUser(user_id): AuthUser,
       State(state): State<AppState>,
       Json(input): Json<CreateTask>,
   ) -> Result<Json<Task>, ApiError>
   
   pub async fn get_task(
       AuthUser(user_id): AuthUser,
       State(state): State<AppState>,
       Path(task_id): Path<Uuid>,
   ) -> Result<Json<Task>, ApiError>
   
   pub async fn list_tasks(
       AuthUser(user_id): AuthUser,
       State(state): State<AppState>,
       Query(filter): Query<TaskFilter>,
   ) -> Result<Json<Vec<Task>>, ApiError>
   
   pub async fn update_task(...) -> Result<Json<Task>, ApiError>
   pub async fn delete_task(...) -> Result<StatusCode, ApiError>
   ```

4. Build router following codebase pattern in `src/tasks/router.rs`:
   ```rust
   pub fn task_routes() -> Router<AppState> {
       Router::new()
           .route("/tasks", post(create_task).get(list_tasks))
           .route("/tasks/:id", get(get_task).put(update_task).delete(delete_task))
   }
   ```

5. Use SQLx compile-time checked queries:
   ```rust
   sqlx::query_as!(Task, r#"
       SELECT id, user_id, title, description,
              status as "status: TaskStatus",
              priority as "priority: TaskPriority",
              created_at, updated_at
       FROM tasks WHERE user_id = $1 AND id = $2
   "#, user_id, task_id)
   ```

### Test Strategy

1. Integration tests for each endpoint with authenticated requests
2. Test task isolation: user A cannot access user B's tasks (return 404, not 403)
3. Test create task with all fields, with minimal fields (defaults applied)
4. Test update with partial fields (PATCH-like behavior)
5. Test delete returns 204 No Content
6. Test list with filters: by status, by priority, with pagination
7. Test 404 for non-existent task
8. Test validation errors return 400 Bad Request
9. Verify updated_at timestamp changes on update

---

## Task 4: Configure HTTP server with middleware stack

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2, 3

### Description

Assemble the complete Axum application with all routes, middleware (CORS, tracing, timeouts), shared state, and graceful shutdown following existing codebase patterns.

### Implementation Details

1. Create AppState in `src/state.rs`:
   ```rust
   #[derive(Clone)]
   pub struct AppState {
       pub db: PgPool,
       pub auth_config: Arc<AuthConfig>,
   }
   ```

2. Build main router in `src/router.rs` following codebase pattern:
   ```rust
   pub fn build_router(state: AppState) -> Router {
       Router::new()
           .route("/health", get(health_check))
           .route("/ready", get(readiness_check))
           .nest("/api/v1", api_routes())
           .layer(
               ServiceBuilder::new()
                   .layer(
                       TraceLayer::new_for_http()
                           .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                           .on_request(DefaultOnRequest::new().level(Level::INFO))
                           .on_response(DefaultOnResponse::new().level(Level::INFO)),
                   )
                   .layer(CorsLayer::permissive())  // Configure properly for production
                   .layer(TimeoutLayer::new(Duration::from_secs(30)))
           )
           .with_state(state)
   }
   
   fn api_routes() -> Router<AppState> {
       Router::new()
           .merge(auth_routes())
           .merge(task_routes())
   }
   ```

3. Implement health endpoints:
   ```rust
   async fn health_check() -> Json<Value> {
       Json(json!({ "status": "healthy", "service": "taskapi" }))
   }
   
   async fn readiness_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
       // Check database connectivity
       sqlx::query("SELECT 1").execute(&state.db).await?;
       Ok(Json(json!({ "status": "ready" })))
   }
   ```

4. Create main binary in `src/bin/taskapi.rs`:
   ```rust
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Initialize tracing (match codebase pattern)
       tracing_subscriber::registry()
           .with(tracing_subscriber::EnvFilter::try_from_default_env()
               .unwrap_or_else(|_| "info,taskapi=debug".into()))
           .with(tracing_subscriber::fmt::layer())
           .init();
       
       // Load config from env
       let database_url = std::env::var("DATABASE_URL")?;
       let jwt_secret = std::env::var("JWT_SECRET")?;
       
       // Create connection pool
       let pool = PgPoolOptions::new()
           .max_connections(5)
           .connect(&database_url).await?;
       
       // Run migrations
       sqlx::migrate!().run(&pool).await?;
       
       // Build app
       let state = AppState { db: pool, auth_config: Arc::new(auth_config) };
       let app = build_router(state);
       
       // Start server with graceful shutdown
       let listener = TcpListener::bind("0.0.0.0:8080").await?;
       info!("Task API listening on 0.0.0.0:8080");
       
       axum::serve(listener, app)
           .with_graceful_shutdown(shutdown_signal())
           .await?;
       
       Ok(())
   }
   ```

5. Add binary entry to Cargo.toml:
   ```toml
   [[bin]]
   name = "taskapi"
   path = "src/bin/taskapi.rs"
   ```

### Test Strategy

1. Start server locally and verify /health returns 200 with JSON body
2. Verify /ready checks database connectivity (fails gracefully if DB down)
3. Test CORS headers present in responses
4. Verify request tracing appears in logs
5. Test graceful shutdown: send SIGTERM, verify in-flight requests complete
6. Test timeout middleware: slow handler should return 504 after 30s
7. Integration test: full request flow from auth to task operations
8. Verify Clippy pedantic passes: `cargo clippy -p taskapi -- -D warnings -W clippy::pedantic`

---

## Task 5: Add input validation and comprehensive error handling

**Status:** pending | **Priority:** medium

**Dependencies:** 3, 4

### Description

Implement robust input validation with helpful error messages and standardized API error responses. This polishes the API for production readiness.

### Implementation Details

1. Add validation dependency:
   - validator = { version = "0.18", features = ["derive"] }

2. Add validation to DTOs:
   ```rust
   use validator::Validate;
   
   #[derive(Deserialize, Validate)]
   pub struct CreateTask {
       #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
       pub title: String,
       #[validate(length(max = 5000, message = "Description too long"))]
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Deserialize, Validate)]
   pub struct RegisterUser {
       #[validate(email(message = "Invalid email format"))]
       pub email: String,
       #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
       pub password: String,
   }
   ```

3. Create ValidatedJson extractor:
   ```rust
   pub struct ValidatedJson<T>(pub T);
   
   #[async_trait]
   impl<T, S> FromRequest<S> for ValidatedJson<T>
   where
       T: DeserializeOwned + Validate,
       S: Send + Sync,
   {
       type Rejection = ApiError;
       
       async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
           let Json(value) = Json::<T>::from_request(req, state).await
               .map_err(|e| ApiError::BadRequest(e.to_string()))?;
           value.validate().map_err(|e| ApiError::ValidationError(e))?;
           Ok(Self(value))
       }
   }
   ```

4. Create standardized error response in `src/error.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ApiError {
       #[error("Bad request: {0}")]
       BadRequest(String),
       #[error("Validation failed")]
       ValidationError(validator::ValidationErrors),
       #[error("Unauthorized")]
       Unauthorized,
       #[error("Not found")]
       NotFound,
       #[error("Conflict: {0}")]
       Conflict(String),
       #[error("Internal server error")]
       Internal(#[from] anyhow::Error),
   }
   
   impl IntoResponse for ApiError {
       fn into_response(self) -> Response {
           let (status, body) = match &self {
               ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, json!({"error": msg})),
               ApiError::ValidationError(e) => (StatusCode::BAD_REQUEST, json!({"errors": format_validation_errors(e)})),
               ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, json!({"error": "Unauthorized"})),
               ApiError::NotFound => (StatusCode::NOT_FOUND, json!({"error": "Not found"})),
               ApiError::Conflict(msg) => (StatusCode::CONFLICT, json!({"error": msg})),
               ApiError::Internal(_) => {
                   tracing::error!("Internal error: {:?}", self);
                   (StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "Internal server error"}))
               }
           };
           (status, Json(body)).into_response()
       }
   }
   ```

5. Update handlers to use ValidatedJson:
   ```rust
   pub async fn create_task(
       AuthUser(user_id): AuthUser,
       State(state): State<AppState>,
       ValidatedJson(input): ValidatedJson<CreateTask>,
   ) -> Result<Json<Task>, ApiError>
   ```

6. Add duplicate email check on registration returning 409 Conflict

### Test Strategy

1. Test title validation: empty string, > 255 chars both return 400 with clear message
2. Test email validation: invalid format returns 400
3. Test password validation: < 8 chars returns 400
4. Test description max length validation
5. Verify error response format is consistent JSON structure
6. Test duplicate email registration returns 409 Conflict
7. Test internal errors don't leak stack traces (return generic message)
8. Verify all validation errors are aggregated (not just first error)
9. Run full test suite: `cargo test -p taskapi`
10. Verify linting passes: `cargo clippy -p taskapi -- -D warnings -W clippy::pedantic`

---

