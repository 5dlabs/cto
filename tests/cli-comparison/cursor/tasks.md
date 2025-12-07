# CURSOR Task Generation Results

**Model:** opus-4.5-thinking
**Duration:** 88.67s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** project, error, auth, database, task, api, jwt

---

## Task 1: Setup project foundation with Axum and PostgreSQL

**Status:** pending | **Priority:** high

### Description

Initialize the Rust project with Axum web framework, SQLx for PostgreSQL database access, and essential dependencies. Establish the project structure, configuration management, and database connection pooling.

### Implementation Details

1. Create new Cargo project with workspace structure:
   ```
   task-manager-api/
   ├── Cargo.toml
   ├── src/
   │   ├── main.rs
   │   ├── config.rs
   │   ├── db.rs
   │   ├── error.rs
   │   └── routes/mod.rs
   └── migrations/
   ```

2. Add dependencies to Cargo.toml:
   - axum = "0.8.4"
   - tokio = { version = "1.40", features = ["full"] }
   - sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
   - tower = "0.5"
   - tower-http = { version = "0.6", features = ["trace", "cors", "timeout"] }
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - tracing = "0.1"
   - tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   - thiserror = "2.0"
   - anyhow = "1.0"
   - uuid = { version = "1.10", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - dotenvy = "0.15"

3. Create config.rs with environment-based configuration:
   ```rust
   pub struct Config {
       pub database_url: String,
       pub jwt_secret: String,
       pub server_port: u16,
   }
   impl Config {
       pub fn from_env() -> Result<Self, anyhow::Error> { ... }
   }
   ```

4. Create db.rs with SQLx connection pool:
   ```rust
   pub type DbPool = sqlx::PgPool;
   pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
       PgPoolOptions::new()
           .max_connections(5)
           .connect(database_url).await
   }
   ```

5. Create error.rs with custom error types using thiserror:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum AppError {
       #[error("Database error: {0}")]
       Database(#[from] sqlx::Error),
       #[error("Not found")]
       NotFound,
       // ...
   }
   impl IntoResponse for AppError { ... }
   ```

6. Set up main.rs with basic Axum server following workspace patterns:
   ```rust
   let app = Router::new()
       .route("/health", get(health_check))
       .layer(TraceLayer::new_for_http())
       .with_state(app_state);
   ```

### Test Strategy

1. Verify `cargo build` succeeds without warnings
2. Run `cargo clippy --all-targets -- -D warnings` passes
3. Test database connection with `sqlx database create` and `sqlx migrate run`
4. Verify health endpoint returns 200 OK with JSON response
5. Test graceful shutdown handling with SIGTERM

---

## Task 2: Implement database schema and migrations for users and tasks

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Create SQLx migrations for users and tasks tables with proper constraints, indexes, and enum types for task status and priority. Implement the domain models with SQLx FromRow derive.

### Implementation Details

1. Create migration for users table (migrations/001_create_users.sql):
   ```sql
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   
   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
       email VARCHAR(255) UNIQUE NOT NULL,
       password_hash VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   CREATE INDEX idx_users_email ON users(email);
   ```

2. Create migration for tasks table (migrations/002_create_tasks.sql):
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

3. Create src/models/mod.rs with domain types:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_status", rename_all = "snake_case")]
   pub enum TaskStatus { Pending, InProgress, Done }
   
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_priority", rename_all = "snake_case")]
   pub enum TaskPriority { Low, Medium, High }
   
   #[derive(Debug, Clone, Serialize, sqlx::FromRow)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       #[serde(skip_serializing)]
       pub password_hash: String,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   #[derive(Debug, Clone, Serialize, sqlx::FromRow)]
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

4. Create src/repositories/ with user and task repository traits and implementations

### Test Strategy

1. Run `sqlx migrate run` successfully against test database
2. Write unit tests for model serialization/deserialization
3. Test enum type mapping with SQLx round-trip queries
4. Verify foreign key constraints work (cascade delete)
5. Test index performance with EXPLAIN ANALYZE on common queries

---

## Task 3: Implement JWT authentication system

**Status:** pending | **Priority:** high

**Dependencies:** 1, 2

### Description

Build JWT-based authentication with login, logout, and token refresh endpoints. Implement password hashing with argon2, JWT token generation/validation with jsonwebtoken crate, and Axum middleware extractor for protected routes.

### Implementation Details

1. Add authentication dependencies:
   - jsonwebtoken = "9.3"
   - argon2 = "0.5"

2. Create src/auth/mod.rs with JWT handling:
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Claims {
       pub sub: Uuid,  // user_id
       pub exp: i64,   // expiration timestamp
       pub iat: i64,   // issued at
   }
   
   pub fn create_token(user_id: Uuid, secret: &str) -> Result<String, AppError> {
       let claims = Claims {
           sub: user_id,
           exp: (Utc::now() + Duration::hours(24)).timestamp(),
           iat: Utc::now().timestamp(),
       };
       encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
   }
   
   pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
       decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
   }
   ```

3. Create Axum extractor for authenticated requests:
   ```rust
   pub struct AuthUser(pub Uuid);
   
   #[async_trait]
   impl<S> FromRequestParts<S> for AuthUser
   where
       S: Send + Sync,
       AppState: FromRef<S>,
   {
       type Rejection = AppError;
       
       async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
           let auth_header = parts.headers.get(AUTHORIZATION)
               .and_then(|h| h.to_str().ok())
               .and_then(|h| h.strip_prefix("Bearer "))
               .ok_or(AppError::Unauthorized)?;
           
           let state = AppState::from_ref(state);
           let claims = verify_token(auth_header, &state.config.jwt_secret)?;
           Ok(AuthUser(claims.sub))
       }
   }
   ```

4. Create auth routes (src/routes/auth.rs):
   ```rust
   pub fn auth_routes() -> Router<AppState> {
       Router::new()
           .route("/register", post(register))
           .route("/login", post(login))
           .route("/refresh", post(refresh_token))
           .route("/logout", post(logout))
   }
   
   async fn register(State(state): State<AppState>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>, AppError> {
       let password_hash = hash_password(&req.password)?;
       // Insert user, return token
   }
   
   async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>, AppError> {
       let user = get_user_by_email(&state.db, &req.email).await?;
       verify_password(&req.password, &user.password_hash)?;
       let token = create_token(user.id, &state.config.jwt_secret)?;
       Ok(Json(AuthResponse { token, user_id: user.id }))
   }
   ```

5. Password hashing with argon2:
   ```rust
   pub fn hash_password(password: &str) -> Result<String, AppError> {
       let salt = SaltString::generate(&mut OsRng);
       let argon2 = Argon2::default();
       Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
   }
   ```

### Test Strategy

1. Unit test JWT token creation and validation
2. Test token expiration handling
3. Test password hashing and verification
4. Integration test login flow: register → login → access protected route
5. Test invalid/expired token rejection
6. Test refresh token flow
7. Verify password is never returned in responses

---

## Task 4: Implement Task CRUD API endpoints

**Status:** pending | **Priority:** high

**Dependencies:** 2, 3

### Description

Create RESTful endpoints for task management including create, read (single and list with filtering), update, and delete operations. All endpoints require authentication and scope tasks to the authenticated user.

### Implementation Details

1. Create task routes (src/routes/tasks.rs):
   ```rust
   pub fn task_routes() -> Router<AppState> {
       Router::new()
           .route("/", get(list_tasks).post(create_task))
           .route("/{id}", get(get_task).put(update_task).delete(delete_task))
   }
   ```

2. Define request/response DTOs:
   ```rust
   #[derive(Debug, Deserialize)]
   pub struct CreateTaskRequest {
       pub title: String,
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct UpdateTaskRequest {
       pub title: Option<String>,
       pub description: Option<String>,
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct ListTasksQuery {
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
       pub limit: Option<i64>,
       pub offset: Option<i64>,
   }
   ```

3. Implement handlers with user scoping:
   ```rust
   async fn create_task(
       State(state): State<AppState>,
       AuthUser(user_id): AuthUser,
       Json(req): Json<CreateTaskRequest>,
   ) -> Result<(StatusCode, Json<Task>), AppError> {
       let task = sqlx::query_as::<_, Task>(
           r#"INSERT INTO tasks (user_id, title, description, priority)
              VALUES ($1, $2, $3, $4)
              RETURNING *"#
       )
       .bind(user_id)
       .bind(&req.title)
       .bind(&req.description)
       .bind(req.priority.unwrap_or(TaskPriority::Medium))
       .fetch_one(&state.db)
       .await?;
       
       Ok((StatusCode::CREATED, Json(task)))
   }
   
   async fn list_tasks(
       State(state): State<AppState>,
       AuthUser(user_id): AuthUser,
       Query(params): Query<ListTasksQuery>,
   ) -> Result<Json<Vec<Task>>, AppError> {
       let mut query = QueryBuilder::new(
           "SELECT * FROM tasks WHERE user_id = "
       );
       query.push_bind(user_id);
       
       if let Some(status) = params.status {
           query.push(" AND status = ").push_bind(status);
       }
       // ... build dynamic query
   }
   
   async fn get_task(
       State(state): State<AppState>,
       AuthUser(user_id): AuthUser,
       Path(task_id): Path<Uuid>,
   ) -> Result<Json<Task>, AppError> {
       sqlx::query_as::<_, Task>(
           "SELECT * FROM tasks WHERE id = $1 AND user_id = $2"
       )
       .bind(task_id)
       .bind(user_id)
       .fetch_optional(&state.db)
       .await?
       .ok_or(AppError::NotFound)
       .map(Json)
   }
   ```

4. Wire routes in main.rs:
   ```rust
   let app = Router::new()
       .route("/health", get(health_check))
       .nest("/api/auth", auth_routes())
       .nest("/api/tasks", task_routes())
       .layer(TraceLayer::new_for_http())
       .with_state(app_state);
   ```

### Test Strategy

1. Test CRUD operations with valid authentication
2. Verify user isolation (user A cannot access user B's tasks)
3. Test filtering by status and priority
4. Test pagination with limit/offset
5. Test 404 for non-existent tasks
6. Test 401 for unauthenticated requests
7. Test partial updates (PATCH semantics via PUT)
8. Verify updated_at timestamp changes on update

---

## Task 5: Add input validation, error handling, and API documentation

**Status:** pending | **Priority:** medium

**Dependencies:** 3, 4

### Description

Implement comprehensive input validation using validator crate, standardize error responses, add request/response logging, and generate OpenAPI documentation with utoipa for API discoverability.

### Implementation Details

1. Add validation dependencies:
   - validator = { version = "0.18", features = ["derive"] }
   - utoipa = { version = "5.3", features = ["axum_extras", "chrono", "uuid"] }
   - utoipa-swagger-ui = { version = "8.1", features = ["axum"] }

2. Add validation to request DTOs:
   ```rust
   use validator::Validate;
   
   #[derive(Debug, Deserialize, Validate)]
   pub struct CreateTaskRequest {
       #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
       pub title: String,
       #[validate(length(max = 10000, message = "Description too long"))]
       pub description: Option<String>,
       pub priority: Option<TaskPriority>,
   }
   
   #[derive(Debug, Deserialize, Validate)]
   pub struct RegisterRequest {
       #[validate(email(message = "Invalid email format"))]
       pub email: String,
       #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
       pub password: String,
   }
   ```

3. Create validation extractor:
   ```rust
   pub struct ValidatedJson<T>(pub T);
   
   #[async_trait]
   impl<T, S> FromRequest<S> for ValidatedJson<T>
   where
       T: DeserializeOwned + Validate,
       S: Send + Sync,
   {
       type Rejection = AppError;
       
       async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
           let Json(value) = Json::<T>::from_request(req, state).await
               .map_err(|e| AppError::BadRequest(e.to_string()))?;
           value.validate().map_err(|e| AppError::Validation(e))?;
           Ok(ValidatedJson(value))
       }
   }
   ```

4. Standardize error responses:
   ```rust
   #[derive(Debug, Serialize)]
   pub struct ErrorResponse {
       pub error: String,
       pub code: String,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub details: Option<serde_json::Value>,
   }
   
   impl IntoResponse for AppError {
       fn into_response(self) -> Response {
           let (status, error_response) = match self {
               AppError::NotFound => (StatusCode::NOT_FOUND, ErrorResponse { ... }),
               AppError::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorResponse { ... }),
               AppError::Validation(e) => (StatusCode::BAD_REQUEST, ErrorResponse {
                   error: "Validation failed".into(),
                   code: "VALIDATION_ERROR".into(),
                   details: Some(serde_json::to_value(e.field_errors()).unwrap()),
               }),
               // ...
           };
           (status, Json(error_response)).into_response()
       }
   }
   ```

5. Add OpenAPI documentation with utoipa:
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(
           routes::auth::register,
           routes::auth::login,
           routes::tasks::create_task,
           routes::tasks::list_tasks,
           // ...
       ),
       components(schemas(Task, CreateTaskRequest, AuthResponse, ErrorResponse)),
       tags((name = "auth", description = "Authentication endpoints")),
       tags((name = "tasks", description = "Task management endpoints")),
   )]
   struct ApiDoc;
   
   // In main.rs:
   .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
   ```

6. Add request ID middleware for tracing:
   ```rust
   async fn add_request_id(mut req: Request, next: Next) -> Response {
       let request_id = Uuid::new_v4().to_string();
       req.extensions_mut().insert(RequestId(request_id.clone()));
       let mut response = next.run(req).await;
       response.headers_mut().insert("x-request-id", request_id.parse().unwrap());
       response
   }
   ```

### Test Strategy

1. Test validation errors return 400 with detailed field errors
2. Test all error types return consistent JSON structure
3. Verify x-request-id header present in all responses
4. Test OpenAPI spec generates valid JSON at /api-docs/openapi.json
5. Verify Swagger UI loads at /swagger-ui
6. Test rate limiting behavior (if implemented)
7. End-to-end test: invalid email format → proper validation message
8. Run `cargo clippy --all-targets -- -D warnings` passes

---

