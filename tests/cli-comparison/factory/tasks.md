# FACTORY Task Generation Results

**Model:** claude-opus-4-5-20251101
**Duration:** 109.44s
**Tasks Generated:** 5
**Theme Coverage:** 100%
**Themes Covered:** docker, database, api, error, project, task, jwt, test, auth

---

## Task 1: Setup project foundation with Axum and PostgreSQL

**Status:** pending | **Priority:** high

### Description

Initialize the Rust project with Axum web framework, SQLx for PostgreSQL database access, and essential dependencies. Establish project structure following Rust conventions.

### Implementation Details

1. Create new Rust binary crate with `cargo new task-manager-api`
2. Configure Cargo.toml with dependencies:
   - axum = "0.8"
   - tokio = { version = "1.40", features = ["full"] }
   - sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
   - serde = { version = "1.0", features = ["derive"] }
   - tower-http = { version = "0.5", features = ["trace", "cors"] }
   - tracing = "0.1"
   - tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   - dotenvy = "0.15"
   - uuid = { version = "1.10", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
3. Create project structure:
   src/
   ├── main.rs           # Entry point, server setup
   ├── config.rs         # Configuration from environment
   ├── db.rs             # Database connection pool
   ├── routes/
   │   └── mod.rs        # Route aggregation
   ├── handlers/         # Request handlers
   ├── models/           # Database models
   └── error.rs          # Custom error types
4. Setup PostgreSQL connection with SQLx:
   ```rust
   use sqlx::postgres::PgPoolOptions;
   pub async fn create_pool(database_url: &str) -> sqlx::PgPool {
       PgPoolOptions::new()
           .max_connections(10)
           .connect(database_url)
           .await
           .expect("Failed to create pool")
   }
   ```
5. Create basic Axum server with health check endpoint
6. Add .env file for DATABASE_URL and other config
7. Setup docker-compose.yml for local PostgreSQL instance

### Test Strategy

1. Verify `cargo build` completes without errors
2. Verify `cargo clippy` passes with no warnings
3. Test health check endpoint returns 200 OK
4. Verify database connection succeeds with valid credentials
5. Test graceful error handling for invalid database URL

---

## Task 2: Implement database schema and migrations

**Status:** pending | **Priority:** high

**Dependencies:** 1

### Description

Design and implement PostgreSQL database schema for users and tasks tables using SQLx migrations. Create corresponding Rust model structs.

### Implementation Details

1. Install sqlx-cli: `cargo install sqlx-cli --features postgres`
2. Create migrations directory and initial migration:
   ```sql
   -- migrations/001_initial_schema.sql
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

   CREATE TABLE users (
       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
       email VARCHAR(255) UNIQUE NOT NULL,
       password_hash VARCHAR(255) NOT NULL,
       created_at TIMESTAMPTZ DEFAULT NOW(),
       updated_at TIMESTAMPTZ DEFAULT NOW()
   );

   CREATE TYPE task_status AS ENUM ('pending', 'in_progress', 'done');
   CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');

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
3. Create Rust models in src/models/:
   ```rust
   // src/models/user.rs
   #[derive(Debug, sqlx::FromRow, Serialize)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       #[serde(skip_serializing)]
       pub password_hash: String,
       pub created_at: DateTime<Utc>,
   }

   // src/models/task.rs
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_status", rename_all = "snake_case")]
   pub enum TaskStatus { Pending, InProgress, Done }

   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
   #[sqlx(type_name = "task_priority", rename_all = "lowercase")]
   pub enum TaskPriority { Low, Medium, High }
   ```
4. Run migrations: `sqlx migrate run`
5. Generate SQLx offline mode files: `cargo sqlx prepare`

### Test Strategy

1. Verify migrations run successfully: `sqlx migrate run`
2. Test rollback capability: `sqlx migrate revert`
3. Verify all model structs compile and derive traits work
4. Test enum serialization/deserialization for status and priority
5. Verify foreign key constraints work correctly
6. Test unique constraint on user email

---

## Task 3: Implement JWT authentication system

**Status:** pending | **Priority:** high

**Dependencies:** 2

### Description

Build complete JWT authentication with login, logout, and token refresh endpoints. Implement secure password hashing and JWT middleware for protected routes.

### Implementation Details

1. Add authentication dependencies to Cargo.toml:
   - jsonwebtoken = "9"
   - argon2 = "0.5"  # Password hashing
   - validator = { version = "0.18", features = ["derive"] }
2. Create JWT configuration in src/auth/jwt.rs:
   ```rust
   use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Claims {
       pub sub: String,  // user_id
       pub exp: usize,   // expiration timestamp
       pub iat: usize,   // issued at
       pub token_type: String,  // "access" or "refresh"
   }
   
   pub fn create_access_token(user_id: &Uuid, secret: &[u8]) -> Result<String, Error> {
       let exp = Utc::now() + Duration::minutes(15);
       let claims = Claims { sub: user_id.to_string(), exp: exp.timestamp() as usize, ... };
       encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
   }
   
   pub fn create_refresh_token(user_id: &Uuid, secret: &[u8]) -> Result<String, Error> {
       let exp = Utc::now() + Duration::days(7);
       // Similar to access token but with longer expiry
   }
   ```
3. Create auth middleware using Axum extractors:
   ```rust
   pub async fn auth_middleware<B>(
       State(state): State<AppState>,
       mut req: Request<B>,
       next: Next<B>,
   ) -> Result<Response, AppError> {
       let token = req.headers().get(AUTHORIZATION)
           .and_then(|v| v.to_str().ok())
           .and_then(|v| v.strip_prefix("Bearer "));
       // Validate token, extract claims, inject user into request extensions
   }
   ```
4. Implement auth handlers in src/handlers/auth.rs:
   - POST /api/auth/register - Create new user with hashed password
   - POST /api/auth/login - Validate credentials, return access + refresh tokens
   - POST /api/auth/refresh - Exchange refresh token for new access token
   - POST /api/auth/logout - Invalidate refresh token (store in blacklist or use short-lived tokens)
5. Password hashing with Argon2:
   ```rust
   use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
   pub fn hash_password(password: &str) -> Result<String, Error> {
       let salt = SaltString::generate(&mut OsRng);
       let argon2 = Argon2::default();
       Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
   }
   ```
6. Store JWT_SECRET in environment, use at least 256-bit secret

### Test Strategy

1. Test user registration with valid/invalid email formats
2. Test login with correct and incorrect credentials
3. Verify access token expires after 15 minutes
4. Verify refresh token generates new access token
5. Test protected endpoint rejects requests without token
6. Test protected endpoint rejects expired tokens
7. Verify password is properly hashed (not stored plain)
8. Test concurrent login from multiple devices
9. Security test: Verify tokens are invalidated properly

---

## Task 4: Implement Task CRUD API endpoints

**Status:** pending | **Priority:** high

**Dependencies:** 3

### Description

Build RESTful endpoints for task management including create, read, update, and delete operations with proper authorization ensuring users can only access their own tasks.

### Implementation Details

1. Create request/response DTOs in src/models/dto.rs:
   ```rust
   #[derive(Debug, Deserialize, Validate)]
   pub struct CreateTaskRequest {
       #[validate(length(min = 1, max = 255))]
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
   pub struct TaskQuery {
       pub status: Option<TaskStatus>,
       pub priority: Option<TaskPriority>,
       pub page: Option<u32>,
       pub per_page: Option<u32>,
   }
   ```
2. Implement task handlers in src/handlers/tasks.rs:
   ```rust
   // GET /api/tasks - List user's tasks with optional filtering
   pub async fn list_tasks(
       State(pool): State<PgPool>,
       Extension(user): Extension<User>,
       Query(params): Query<TaskQuery>,
   ) -> Result<Json<Vec<Task>>, AppError> {
       let tasks = sqlx::query_as!(Task,
           r#"SELECT id, title, description, status as "status: TaskStatus", 
              priority as "priority: TaskPriority", created_at, updated_at
           FROM tasks WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
           user.id, per_page, offset
       ).fetch_all(&pool).await?;
       Ok(Json(tasks))
   }

   // POST /api/tasks - Create new task
   // GET /api/tasks/:id - Get single task
   // PUT /api/tasks/:id - Update task
   // DELETE /api/tasks/:id - Delete task
   ```
3. Add authorization check for task ownership:
   ```rust
   async fn verify_task_ownership(pool: &PgPool, task_id: Uuid, user_id: Uuid) -> Result<Task, AppError> {
       sqlx::query_as!(Task, "SELECT * FROM tasks WHERE id = $1 AND user_id = $2", task_id, user_id)
           .fetch_optional(pool).await?
           .ok_or(AppError::NotFound("Task not found".into()))
   }
   ```
4. Configure routes with auth middleware:
   ```rust
   let task_routes = Router::new()
       .route("/", get(list_tasks).post(create_task))
       .route("/:id", get(get_task).put(update_task).delete(delete_task))
       .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
   ```
5. Implement proper error responses with appropriate HTTP status codes

### Test Strategy

1. Test create task with valid data returns 201 Created
2. Test create task with invalid data returns 400 Bad Request
3. Test list tasks only returns authenticated user's tasks
4. Test filtering by status and priority works correctly
5. Test pagination returns correct page size and offset
6. Test get task by ID returns correct task
7. Test get non-existent task returns 404
8. Test get another user's task returns 404 (not 403)
9. Test update task changes only provided fields
10. Test delete task removes task from database
11. Test all endpoints require authentication (401 without token)

---

## Task 5: Add API documentation, validation, and integration tests

**Status:** pending | **Priority:** medium

**Dependencies:** 4

### Description

Implement comprehensive input validation, OpenAPI documentation, structured error handling, and integration tests to ensure API reliability and developer experience.

### Implementation Details

1. Add documentation dependencies:
   - utoipa = { version = "5", features = ["axum_extras"] }
   - utoipa-swagger-ui = { version = "8", features = ["axum"] }
2. Add OpenAPI documentation to handlers:
   ```rust
   #[utoipa::path(
       post,
       path = "/api/tasks",
       request_body = CreateTaskRequest,
       responses(
           (status = 201, description = "Task created", body = Task),
           (status = 400, description = "Invalid input"),
           (status = 401, description = "Unauthorized"),
       ),
       security(("bearer_auth" = []))
   )]
   pub async fn create_task(...) { ... }
   ```
3. Implement structured error handling:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum AppError {
       #[error("Not found: {0}")]
       NotFound(String),
       #[error("Unauthorized")]
       Unauthorized,
       #[error("Validation error: {0}")]
       Validation(String),
       #[error("Database error")]
       Database(#[from] sqlx::Error),
   }

   impl IntoResponse for AppError {
       fn into_response(self) -> Response {
           let (status, message) = match &self {
               AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
               AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
               // ...
           };
           (status, Json(json!({ "error": message }))).into_response()
       }
   }
   ```
4. Create integration tests in tests/integration/:
   ```rust
   // tests/integration/auth_test.rs
   #[tokio::test]
   async fn test_register_and_login() {
       let app = spawn_app().await;
       let client = reqwest::Client::new();
       
       // Register
       let res = client.post(&format!("{}/api/auth/register", app.address))
           .json(&json!({ "email": "test@example.com", "password": "secure123" }))
           .send().await.unwrap();
       assert_eq!(res.status(), 201);
       
       // Login
       let res = client.post(&format!("{}/api/auth/login", app.address))
           .json(&json!({ "email": "test@example.com", "password": "secure123" }))
           .send().await.unwrap();
       assert_eq!(res.status(), 200);
       let body: Value = res.json().await.unwrap();
       assert!(body["access_token"].is_string());
   }
   ```
5. Add rate limiting with tower middleware
6. Configure CORS for frontend integration
7. Add request logging with tracing

### Test Strategy

1. Verify Swagger UI accessible at /swagger-ui
2. Verify OpenAPI spec downloadable at /api-doc/openapi.json
3. Run full integration test suite: `cargo test --test '*'`
4. Test error responses have consistent JSON structure
5. Test validation errors return helpful messages
6. Test rate limiting kicks in after threshold
7. Verify CORS headers present for allowed origins
8. Test concurrent requests don't cause race conditions
9. Load test with 100 concurrent users
10. Security audit: Test SQL injection prevention
11. Security audit: Test XSS prevention in responses

---

