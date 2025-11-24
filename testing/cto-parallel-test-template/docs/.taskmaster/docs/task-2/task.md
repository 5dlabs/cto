# Task 2: API Endpoints

## Overview
Establish the REST API routing infrastructure using Actix-web, creating the HTTP layer that connects clients to business logic.

## Context
This is a **Level 1 task** that depends on Task 1 (Database Schema). It sets up the API server and routing structure that will be populated by subsequent tasks with actual endpoint implementations.

## Objectives
1. Set up Actix-web HTTP server
2. Configure route structure with scopes
3. Implement health check endpoint
4. Create placeholder route configurations for future endpoints
5. Establish error handling patterns
6. Configure CORS and middleware (if needed)

## Dependencies
- **Task 1:** Database Schema Setup (requires schema.rs to be available)

## Architecture Context
Refer to `.taskmaster/docs/architecture.md` sections:
- **API Endpoints** (lines 373-395): Complete endpoint listing
- **Backend Architecture** (lines 73-105): Module structure
- **High-Level Architecture** (lines 32-71): System overview

## Implementation Plan

### Step 1: Add Actix-web Dependencies
Update `Cargo.toml`:
```toml
[dependencies]
actix-web = "4.3.1"
actix-rt = "2.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Validation:** Run `cargo check`

### Step 2: Create API Module Structure
Create `src/api/mod.rs`:
```rust
pub mod routes;
pub mod errors;

pub use routes::configure_routes;
pub use errors::ApiError;
```

### Step 3: Implement Error Handling
Create `src/api/errors.rs`:
```rust
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    Unauthorized(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": msg
            })),
            ApiError::BadRequest(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "bad_request",
                "message": msg
            })),
            ApiError::InternalError(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": msg
            })),
            ApiError::Unauthorized(msg) => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": msg
            })),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }
}
```

### Step 4: Create Route Configuration
Create `src/api/routes.rs`:
```rust
use actix_web::{web, HttpResponse, Responder};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .service(web::scope("/auth").configure(auth_routes))
            .service(web::scope("/users").configure(user_routes))
            .service(web::scope("/products").configure(product_routes))
            .service(web::scope("/cart").configure(cart_routes))
    );
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Placeholder configurations - to be implemented in later tasks
fn auth_routes(cfg: &mut web::ServiceConfig) {
    // Task 3: Authentication endpoints
    cfg.route("/register", web::post().to(not_implemented))
       .route("/login", web::post().to(not_implemented));
}

fn user_routes(cfg: &mut web::ServiceConfig) {
    // Task 3: User management endpoints
    cfg.route("", web::get().to(not_implemented));
}

fn product_routes(cfg: &mut web::ServiceConfig) {
    // Task 4: Product catalog endpoints
    cfg.route("", web::get().to(not_implemented))
       .route("/{id}", web::get().to(not_implemented));
}

fn cart_routes(cfg: &mut web::ServiceConfig) {
    // Task 5: Shopping cart endpoints
    cfg.route("", web::get().to(not_implemented))
       .route("/add", web::post().to(not_implemented))
       .route("/remove/{product_id}", web::delete().to(not_implemented))
       .route("/clear", web::post().to(not_implemented));
}

async fn not_implemented() -> impl Responder {
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "not_implemented",
        "message": "This endpoint will be implemented in a later task"
    }))
}
```

### Step 5: Set Up Main Application
Update `src/main.rs`:
```rust
use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;

mod api;
mod config;
mod models;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    println!("🚀 Starting API server on http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(api::configure_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Step 6: Add Logging Support
Update `Cargo.toml`:
```toml
[dependencies]
env_logger = "0.10"
log = "0.4"
```

## Testing Strategy

### Manual Testing
```bash
# Start the server
cargo run

# Test health check
curl http://localhost:8080/api/health

# Expected response:
# {"status":"ok","version":"0.1.0"}

# Test not-implemented endpoints
curl http://localhost:8080/api/products
# Expected: 501 Not Implemented

# Test 404 handling
curl http://localhost:8080/api/nonexistent
# Expected: 404 Not Found
```

### Integration Tests
Create `tests/api_routes_test.rs`:
```rust
#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().configure(crate::api::configure_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/api/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

## Risks and Considerations
- **Port Binding:** Ensure port 8080 is available
- **Module Dependencies:** Task 1 must be complete before this task
- **Placeholder Routes:** Return 501 (Not Implemented) to clearly indicate future work
- **Error Handling:** Establish patterns early for consistency
- **CORS:** May need CORS middleware for frontend integration (Task 6)

## Success Criteria
- [ ] Actix-web dependencies added to Cargo.toml
- [ ] `src/api/mod.rs` exports routes and errors
- [ ] `src/api/errors.rs` defines ApiError enum with ResponseError trait
- [ ] `src/api/routes.rs` configures all route scopes
- [ ] Health check endpoint returns 200 OK with JSON
- [ ] Placeholder routes return 501 Not Implemented
- [ ] Server starts without errors on port 8080
- [ ] `cargo check` and `cargo build` succeed
- [ ] Logger middleware configured
- [ ] Route structure matches architecture document

## Files Modified/Created
- `Cargo.toml` - Add actix-web and logging dependencies
- `src/api/mod.rs` - API module exports
- `src/api/routes.rs` - Route configuration
- `src/api/errors.rs` - Error handling
- `src/main.rs` - HTTP server setup
- `tests/api_routes_test.rs` - Integration tests

## Next Steps
This routing infrastructure will be populated by:
- **Task 3:** Authentication endpoints (/api/auth/*)
- **Task 4:** Product catalog endpoints (/api/products/*)
- **Task 5:** Shopping cart endpoints (/api/cart/*)
- **Task 6:** Frontend will consume these APIs
