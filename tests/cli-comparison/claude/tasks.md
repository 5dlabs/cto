# CLAUDE Task Generation Results

**Model:** claude-opus-4-5-20251101
**Duration:** 210.48s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** jwt, project, api, error, task, auth, database

---

## Task 1: Setup project foundation with database

**Status:** pending | **Priority:** high

### Description

Initialize the Rust project structure with Axum web framework, PostgreSQL database connectivity via sqlx, and essential dependencies. This task establishes the foundation for all subsequent features.

### Implementation Details

1. Create new crate structure following workspace pattern:
   ```
   crates/task-api/
   ├── Cargo.toml
   ├── src/
   │   ├── main.rs
   │   ├── lib.rs
   │   ├── config.rs
   │   └── db.rs
   └── migrations/
   ```

2. Add dependencies to Cargo.toml:
   ```toml
   [dependencies]
   axum = { workspace = true }  # 0.8.4
   tokio = { workspace = true }  # 1.40
   tower = { workspace = true }  # 0.5
   tower-http = { workspace = true }
   sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "migrate"] }
   serde = { workspace = true }
   serde_json = { workspace = true }
   anyhow = { workspace = true }
   thiserror = { workspace = true }
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true }
   uuid = { workspace = true }
   chrono = { workspace = true }
   ```

3. Create config.rs for environment configuration:
   ```rust
   pub struct Config {
       pub database_url: String,
       pub jwt_secret: String,
       pub server_port: u16,
   }
   
   impl Config {
       pub fn from_env() -> Result<Self, anyhow::Error> {
           Ok(Self {
               database_url: std::env::var("DATABASE_URL")?,
               jwt_secret: std::env::var("JWT_SECRET")?,
               server_port: std::env::var("PORT").unwrap_or("3000".into()).parse()?,
           })
       }
   }
   ```

4. Create db.rs with connection pool:
   ```rust
   use sqlx::postgres::{PgPool, PgPoolOptions};
   
   pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
       PgPoolOptions::new()
           .max_connections(5)
           .connect(database_url)
           .await
   }
   ```

5. Create initial migration (migrations/001_initial.sql):
   ```sql
   CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'done');
   CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');
   
   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       email VARCHAR(255) NOT NULL UNIQUE,
       password_hash VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   
   CREATE TABLE tasks (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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

6. Setup main.rs with basic server:
   ```rust
   #[tokio::main]
   async fn main() -> Result<(), anyhow::Error> {
       tracing_subscriber::fmt::init();
       let config = Config::from_env()?;
       let pool = create_pool(&config.database_url).await?;
       sqlx::migrate!().run(&pool).await?;
       
       let state = AppState { db: pool, config };
       let app = Router::new()
           .route("/health", get(|| async { Json(json!({"status": "healthy"})) }))
           .with_state(state);
       
       let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await?;
       axum::serve(listener, app).await?;
       Ok(())
   }
   ```

### Test Strategy

1. Unit tests: Verify Config::from_env() parses environment variables correctly with valid and missing values
2. Integration tests: Verify database connection pool creation succeeds with valid DATABASE_URL
3. Migration tests: Run sqlx migrate to verify migrations apply without errors
4. Health check test: GET /health returns 200 with {"status": "healthy"}
5. Verify the server starts and binds to configured port

---

## Task 2: Implement JWT authentication system

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Build the authentication layer with JWT token generation, validation, password hashing, and login/logout/refresh endpoints. Uses jsonwebtoken crate for JWT operations and argon2 for secure password hashing.

### Implementation Details

1. Add auth dependencies to Cargo.toml:
   ```toml
   jsonwebtoken = "9.3"
   argon2 = "0.5"
   axum-extra = { version = "0.10", features = ["typed-header"] }
   ```

2. Create src/auth/mod.rs with JWT types:
   ```rust
   use serde::{Deserialize, Serialize};
   use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Claims {
       pub sub: String,  // user_id
       pub exp: i64,
       pub iat: i64,
       pub token_type: TokenType,
   }
   
   #[derive(Debug, Serialize, Deserialize, PartialEq)]
   pub enum TokenType { Access, Refresh }
   
   pub fn generate_tokens(user_id: &str, secret: &str) -> Result<(String, String), Error> {
       let now = chrono::Utc::now();
       let access_claims = Claims {
           sub: user_id.to_string(),
           exp: (now + chrono::Duration::hours(1)).timestamp(),
           iat: now.timestamp(),
           token_type: TokenType::Access,
       };
       let refresh_claims = Claims {
           sub: user_id.to_string(),
           exp: (now + chrono::Duration::days(7)).timestamp(),
           iat: now.timestamp(),
           token_type: TokenType::Refresh,
       };
       let key = EncodingKey::from_secret(secret.as_bytes());
       Ok((
           encode(&Header::default(), &access_claims, &key)?,
           encode(&Header::default(), &refresh_claims, &key)?,
       ))
   }
   ```

3. Create JWT extractor (src/auth/extractor.rs) following Axum 0.8 pattern:
   ```rust
   use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
   use axum_extra::headers::{Authorization, authorization::Bearer};
   use axum_extra::TypedHeader;
   
   pub struct AuthUser {
       pub user_id: uuid::Uuid,
   }
   
   #[async_trait]
   impl<S> FromRequestParts<S> for AuthUser
   where S: Send + Sync {
       type Rejection = (StatusCode, Json<Value>);
       
       async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
           let TypedHeader(Authorization::<Bearer>(bearer)) = 
               TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                   .await
                   .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"error": "Missing token"}))))?;
           
           // Decode and validate token...
       }
   }
   ```

4. Create auth routes (src/auth/routes.rs):
   ```rust
   pub fn auth_routes() -> Router<AppState> {
       Router::new()
           .route("/login", post(login))
           .route("/register", post(register))
           .route("/refresh", post(refresh_token))
           .route("/logout", post(logout))
   }
   
   async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Result<Json<TokenResponse>, ApiError> {
       let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", req.email)
           .fetch_optional(&state.db).await?
           .ok_or(ApiError::Unauthorized)?;
       
       verify_password(&req.password, &user.password_hash)?;
       let (access, refresh) = generate_tokens(&user.id.to_string(), &state.config.jwt_secret)?;
       Ok(Json(TokenResponse { access_token: access, refresh_token: refresh, token_type: "Bearer" }))
   }
   ```

5. Implement password hashing:
   ```rust
   use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
   use argon2::password_hash::SaltString;
   
   pub fn hash_password(password: &str) -> Result<String, Error> {
       let salt = SaltString::generate(&mut rand::thread_rng());
       Ok(Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string())
   }
   
   pub fn verify_password(password: &str, hash: &str) -> Result<(), Error> {
       let parsed = PasswordHash::new(hash)?;
       Argon2::default().verify_password(password.as_bytes(), &parsed)
           .map_err(|_| Error::InvalidCredentials)
   }
   ```

### Test Strategy

1. Unit tests for JWT generation: verify token structure, expiration times, and claims
2. Unit tests for password hashing: verify hash_password produces valid argon2 hash, verify_password accepts correct password and rejects wrong password
3. Integration tests for /auth/register: POST creates user, returns 201, stores hashed password in DB
4. Integration tests for /auth/login: returns tokens for valid credentials, returns 401 for invalid
5. Integration tests for /auth/refresh: returns new access token for valid refresh token, rejects expired tokens
6. Test AuthUser extractor rejects missing/invalid/expired tokens with appropriate error responses

---

## Task 3: Implement task CRUD operations

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2

### Description

Build the core task management API with Create, Read, Update, Delete operations for tasks. Tasks are scoped to authenticated users and support status (pending/in_progress/done) and priority (low/medium/high) fields.

### Implementation Details

1. Create task models (src/tasks/models.rs):
   ```rust
   use serde::{Deserialize, Serialize};
   use sqlx::FromRow;
   
   #[derive(Debug, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_status", rename_all = "snake_case")]
   pub enum TaskStatus { Pending, InProgress, Done }
   
   #[derive(Debug, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_priority", rename_all = "snake_case")]
   pub enum TaskPriority { Low, Medium, High }
   
   #[derive(Debug, Serialize, FromRow)]
   pub struct Task {
       pub id: uuid::Uuid,
       pub user_id: uuid::Uuid,
       pub title: String,
       pub description: Option<String>,
       pub status: TaskStatus,
       pub priority: TaskPriority,
       pub created_at: chrono::DateTime<chrono::Utc>,
       pub updated_at: chrono::DateTime<chrono::Utc>,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct CreateTask {
       pub title: String,
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct UpdateTask {
       pub title: Option<String>,
       pub description: Option<String>,
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
   }
   ```

2. Create task routes (src/tasks/routes.rs):
   ```rust
   pub fn task_routes() -> Router<AppState> {
       Router::new()
           .route("/", get(list_tasks).post(create_task))
           .route("/:id", get(get_task).put(update_task).delete(delete_task))
   }
   ```

3. Implement handlers:
   ```rust
   // Create task
   async fn create_task(
       State(state): State<AppState>,
       auth: AuthUser,
       Json(req): Json<CreateTask>,
   ) -> Result<(StatusCode, Json<Task>), ApiError> {
       let task = sqlx::query_as!(
           Task,
           r#"INSERT INTO tasks (user_id, title, description, priority)
              VALUES ($1, $2, $3, $4)
              RETURNING id, user_id, title, description, 
                        status as "status: TaskStatus",
                        priority as "priority: TaskPriority",
                        created_at, updated_at"#,
           auth.user_id, req.title, req.description,
           req.priority.unwrap_or(TaskPriority::Medium) as TaskPriority
       )
       .fetch_one(&state.db).await?;
       Ok((StatusCode::CREATED, Json(task)))
   }
   
   // List tasks with optional filters
   async fn list_tasks(
       State(state): State<AppState>,
       auth: AuthUser,
       Query(params): Query<ListParams>,
   ) -> Result<Json<Vec<Task>>, ApiError> {
       let tasks = sqlx::query_as!(
           Task,
           r#"SELECT id, user_id, title, description,
                     status as "status: TaskStatus",
                     priority as "priority: TaskPriority",
                     created_at, updated_at
              FROM tasks WHERE user_id = $1
              ORDER BY created_at DESC"#,
           auth.user_id
       )
       .fetch_all(&state.db).await?;
       Ok(Json(tasks))
   }
   
   // Get single task
   async fn get_task(
       State(state): State<AppState>,
       auth: AuthUser,
       Path(task_id): Path<uuid::Uuid>,
   ) -> Result<Json<Task>, ApiError> {
       sqlx::query_as!(...)
           .fetch_optional(&state.db).await?
           .ok_or(ApiError::NotFound)
           .map(Json)
   }
   
   // Update task
   async fn update_task(
       State(state): State<AppState>,
       auth: AuthUser,
       Path(task_id): Path<uuid::Uuid>,
       Json(req): Json<UpdateTask>,
   ) -> Result<Json<Task>, ApiError> {
       let task = sqlx::query_as!(
           Task,
           r#"UPDATE tasks SET
              title = COALESCE($3, title),
              description = COALESCE($4, description),
              status = COALESCE($5, status),
              priority = COALESCE($6, priority),
              updated_at = NOW()
              WHERE id = $1 AND user_id = $2
              RETURNING ..."#,
           task_id, auth.user_id, req.title, req.description,
           req.status as Option<TaskStatus>,
           req.priority as Option<TaskPriority>
       )
       .fetch_optional(&state.db).await?
       .ok_or(ApiError::NotFound)?;
       Ok(Json(task))
   }
   
   // Delete task
   async fn delete_task(
       State(state): State<AppState>,
       auth: AuthUser,
       Path(task_id): Path<uuid::Uuid>,
   ) -> Result<StatusCode, ApiError> {
       let result = sqlx::query!(
           "DELETE FROM tasks WHERE id = $1 AND user_id = $2",
           task_id, auth.user_id
       )
       .execute(&state.db).await?;
       
       if result.rows_affected() == 0 {
           Err(ApiError::NotFound)
       } else {
           Ok(StatusCode::NO_CONTENT)
       }
   }
   ```

4. Wire routes in main.rs:
   ```rust
   let app = Router::new()
       .route("/health", get(health))
       .nest("/auth", auth_routes())
       .nest("/tasks", task_routes())
       .with_state(state);
   ```

### Test Strategy

1. Integration tests for POST /tasks: creates task with valid token, returns 401 without token, validates required fields
2. Integration tests for GET /tasks: returns only tasks belonging to authenticated user, empty array for new user
3. Integration tests for GET /tasks/:id: returns task if owned by user, 404 for non-existent, 404 for other user's task (not 403 to avoid enumeration)
4. Integration tests for PUT /tasks/:id: updates specified fields only, preserves unspecified fields, validates enum values for status/priority
5. Integration tests for DELETE /tasks/:id: returns 204 on success, 404 for non-existent, removes task from database
6. Test status transitions: pending -> in_progress -> done
7. Test priority values: low, medium, high accepted; invalid values rejected with 400

---

## Task 4: Add error handling and validation

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2, 3

### Description

Implement comprehensive error handling with structured JSON error responses, input validation using validator crate, and consistent error formatting across all endpoints.

### Implementation Details

1. Add validation dependency:
   ```toml
   validator = { version = "0.18", features = ["derive"] }
   ```

2. Create error types (src/error.rs):
   ```rust
   use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
   use serde_json::json;
   
   #[derive(Debug, thiserror::Error)]
   pub enum ApiError {
       #[error("Resource not found")]
       NotFound,
       #[error("Unauthorized")]
       Unauthorized,
       #[error("Invalid credentials")]
       InvalidCredentials,
       #[error("Validation error: {0}")]
       Validation(String),
       #[error("Conflict: {0}")]
       Conflict(String),
       #[error("Internal server error")]
       Internal(#[from] anyhow::Error),
       #[error("Database error")]
       Database(#[from] sqlx::Error),
   }
   
   impl IntoResponse for ApiError {
       fn into_response(self) -> Response {
           let (status, error_code, message) = match &self {
               ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
               ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
               ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS", self.to_string()),
               ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
               ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
               ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".into()),
               ApiError::Database(e) => {
                   tracing::error!(error = %e, "Database error");
                   (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error".into())
               }
           };
           
           (status, Json(json!({
               "error": {
                   "code": error_code,
                   "message": message
               }
           }))).into_response()
       }
   }
   ```

3. Add validation to request structs:
   ```rust
   use validator::Validate;
   
   #[derive(Debug, Deserialize, Validate)]
   pub struct CreateTask {
       #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
       pub title: String,
       #[validate(length(max = 10000, message = "Description too long"))]
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Debug, Deserialize, Validate)]
   pub struct LoginRequest {
       #[validate(email(message = "Invalid email format"))]
       pub email: String,
       #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
       pub password: String,
   }
   
   #[derive(Debug, Deserialize, Validate)]
   pub struct RegisterRequest {
       #[validate(email(message = "Invalid email format"))]
       pub email: String,
       #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
       pub password: String,
   }
   ```

4. Create validated JSON extractor:
   ```rust
   use axum::{async_trait, extract::{FromRequest, Request}};
   use validator::Validate;
   
   pub struct ValidatedJson<T>(pub T);
   
   #[async_trait]
   impl<S, T> FromRequest<S> for ValidatedJson<T>
   where
       S: Send + Sync,
       T: DeserializeOwned + Validate,
   {
       type Rejection = ApiError;
       
       async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
           let Json(value) = Json::<T>::from_request(req, state)
               .await
               .map_err(|e| ApiError::Validation(e.to_string()))?;
           
           value.validate()
               .map_err(|e| ApiError::Validation(format_validation_errors(&e)))?;
           
           Ok(ValidatedJson(value))
       }
   }
   
   fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
       errors.field_errors()
           .iter()
           .flat_map(|(field, errs)| errs.iter().map(move |e| 
               format!("{}: {}", field, e.message.as_ref().map(|m| m.to_string()).unwrap_or_default())
           ))
           .collect::<Vec<_>>()
           .join("; ")
   }
   ```

5. Handle database constraint violations:
   ```rust
   impl From<sqlx::Error> for ApiError {
       fn from(e: sqlx::Error) -> Self {
           match &e {
               sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                   ApiError::Conflict("Resource already exists".into())
               }
               _ => ApiError::Database(e)
           }
       }
   }
   ```

### Test Strategy

1. Test validation errors: POST /auth/register with invalid email returns 400 with validation message
2. Test validation errors: POST /tasks with empty title returns 400
3. Test validation errors: POST /tasks with title > 255 chars returns 400
4. Test not found: GET /tasks/{random-uuid} returns 404 with structured error
5. Test conflict: POST /auth/register with existing email returns 409
6. Test unauthorized: Access protected endpoint without token returns 401
7. Test error response format: All errors return {"error": {"code": "...", "message": "..."}}
8. Test internal errors don't leak stack traces to client

---

## Task 5: Add middleware and API polish

**Status:** pending | **Priority:** medium

**Dependencies:** 1, 2, 3, 4

### Description

Add production-ready middleware including request tracing, CORS configuration, rate limiting, and request timeouts. Also add OpenAPI documentation generation and comprehensive health check endpoints.

### Implementation Details

1. Add middleware dependencies:
   ```toml
   tower-http = { version = "0.5", features = ["trace", "cors", "limit", "timeout", "request-id"] }
   utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
   utoipa-swagger-ui = { version = "8", features = ["axum"] }
   ```

2. Configure middleware stack in main.rs:
   ```rust
   use tower::ServiceBuilder;
   use tower_http::{
       trace::TraceLayer,
       cors::{CorsLayer, Any},
       timeout::TimeoutLayer,
       limit::RequestBodyLimitLayer,
       request_id::{SetRequestIdLayer, PropagateRequestIdLayer},
   };
   use std::time::Duration;
   
   let middleware = ServiceBuilder::new()
       .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
       .layer(PropagateRequestIdLayer::x_request_id())
       .layer(TraceLayer::new_for_http()
           .make_span_with(|request: &Request<_>| {
               let request_id = request.headers()
                   .get("x-request-id")
                   .and_then(|v| v.to_str().ok())
                   .unwrap_or("unknown");
               tracing::info_span!("http_request",
                   method = %request.method(),
                   uri = %request.uri(),
                   request_id = %request_id
               )
           }))
       .layer(TimeoutLayer::new(Duration::from_secs(30)))
       .layer(RequestBodyLimitLayer::new(1024 * 1024))  // 1MB
       .layer(CorsLayer::new()
           .allow_origin(Any)
           .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
           .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]));
   
   let app = Router::new()
       // ... routes ...
       .layer(middleware)
       .with_state(state);
   ```

3. Add OpenAPI documentation with utoipa:
   ```rust
   use utoipa::OpenApi;
   use utoipa_swagger_ui::SwaggerUi;
   
   #[derive(OpenApi)]
   #[openapi(
       paths(
           auth::login, auth::register, auth::refresh_token,
           tasks::create_task, tasks::list_tasks, tasks::get_task,
           tasks::update_task, tasks::delete_task
       ),
       components(schemas(
           Task, CreateTask, UpdateTask, TaskStatus, TaskPriority,
           LoginRequest, RegisterRequest, TokenResponse, ApiError
       )),
       tags(
           (name = "auth", description = "Authentication endpoints"),
           (name = "tasks", description = "Task management endpoints")
       )
   )]
   struct ApiDoc;
   
   // Add to router:
   .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
   ```

4. Enhanced health check:
   ```rust
   #[derive(Serialize)]
   struct HealthResponse {
       status: &'static str,
       version: &'static str,
       database: DatabaseHealth,
   }
   
   #[derive(Serialize)]
   struct DatabaseHealth {
       connected: bool,
       latency_ms: Option<u64>,
   }
   
   async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
       let start = std::time::Instant::now();
       let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
       let latency = start.elapsed().as_millis() as u64;
       
       Json(HealthResponse {
           status: if db_ok { "healthy" } else { "degraded" },
           version: env!("CARGO_PKG_VERSION"),
           database: DatabaseHealth {
               connected: db_ok,
               latency_ms: if db_ok { Some(latency) } else { None },
           },
       })
   }
   ```

5. Add graceful shutdown:
   ```rust
   let listener = tokio::net::TcpListener::bind(addr).await?;
   axum::serve(listener, app)
       .with_graceful_shutdown(shutdown_signal())
       .await?;
   
   async fn shutdown_signal() {
       let ctrl_c = async {
           tokio::signal::ctrl_c().await.expect("failed to listen for ctrl+c");
       };
       #[cfg(unix)]
       let terminate = async {
           tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
               .expect("failed to install signal handler")
               .recv()
               .await;
       };
       #[cfg(not(unix))]
       let terminate = std::future::pending::<()>();
       
       tokio::select! {
           _ = ctrl_c => {},
           _ = terminate => {},
       }
       tracing::info!("Shutdown signal received");
   }
   ```

### Test Strategy

1. Test CORS: OPTIONS request returns correct CORS headers
2. Test request timeout: Slow handler returns 408 after timeout
3. Test body limit: Request > 1MB returns 413
4. Test request ID: Response includes x-request-id header
5. Test tracing: Logs include request_id, method, uri for each request
6. Test health endpoint: Returns database connectivity status and latency
7. Test OpenAPI: GET /api-docs/openapi.json returns valid OpenAPI spec
8. Test Swagger UI: GET /swagger-ui loads documentation interface
9. Test graceful shutdown: Server completes in-flight requests before terminating

---

