# Autonomous Agent Prompt: API Endpoints Setup

## Mission
Set up the REST API routing infrastructure using Actix-web framework, creating the HTTP server and route structure for an e-commerce application.

## Goal
Create a working HTTP server with:
- Health check endpoint
- Route scopes for authentication, users, products, and cart
- Error handling infrastructure
- Placeholder endpoints for future implementation
- Logging middleware

## Prerequisites
- Task 1 (Database Schema) must be complete
- `src/schema.rs` must exist
- PostgreSQL running (for full integration)

## Step-by-Step Instructions

### 1. Add Actix-web Dependencies
Update `Cargo.toml` `[dependencies]` section:
```toml
actix-web = "4.3.1"
actix-rt = "2.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
env_logger = "0.10"
log = "0.4"
```

Verify: `cargo check`

### 2. Create API Module Structure
Create `src/api/mod.rs`:
```rust
pub mod routes;
pub mod errors;

pub use routes::configure_routes;
pub use errors::ApiError;
```

### 3. Implement Error Handling
Create `src/api/errors.rs` with an ApiError enum that implements `ResponseError` trait. Include variants for:
- NotFound
- BadRequest
- InternalError
- Unauthorized

Each variant should return appropriate HTTP status codes and JSON error responses.

### 4. Configure Routes
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

fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(not_implemented))
       .route("/login", web::post().to(not_implemented));
}

fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(not_implemented));
}

fn product_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(not_implemented))
       .route("/{id}", web::get().to(not_implemented));
}

fn cart_routes(cfg: &mut web::ServiceConfig) {
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

### 5. Set Up HTTP Server
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

### 6. Test the Server
Start the server:
```bash
cargo run
```

In another terminal, test endpoints:
```bash
# Health check (should return 200)
curl http://localhost:8080/api/health

# Placeholder endpoint (should return 501)
curl http://localhost:8080/api/products

# Non-existent route (should return 404)
curl http://localhost:8080/api/invalid
```

### 7. Create Integration Test
Create `tests/api_routes_test.rs`:
```rust
use actix_web::{test, App};

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new().configure(your_crate::api::configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[actix_web::test]
async fn test_not_implemented_endpoints() {
    let app = test::init_service(
        App::new().configure(your_crate::api::configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/products")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 501); // Not Implemented
}
```

Run tests: `cargo test`

## Success Criteria
You have succeeded when:
- [ ] Server starts without errors
- [ ] Health check returns 200 OK with JSON `{"status":"ok"}`
- [ ] All placeholder endpoints return 501 Not Implemented
- [ ] Non-existent routes return 404
- [ ] Logger middleware prints request logs
- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] Error responses follow consistent JSON format

## Error Handling
- **Port already in use**: Change port or stop conflicting process
- **Module not found**: Ensure Task 1 files exist (schema.rs, models.rs, config/)
- **Compilation errors**: Check Actix-web version (4.3.1)

## Route Structure
```
/api
├── /health (GET) - Health check
├── /auth
│   ├── /register (POST) - User registration (Task 3)
│   └── /login (POST) - User login (Task 3)
├── /users (GET) - User management (Task 3)
├── /products
│   ├── / (GET) - List products (Task 4)
│   └── /{id} (GET) - Get product (Task 4)
└── /cart
    ├── / (GET) - Get cart (Task 5)
    ├── /add (POST) - Add to cart (Task 5)
    ├── /remove/{product_id} (DELETE) - Remove from cart (Task 5)
    └── /clear (POST) - Clear cart (Task 5)
```

## Key Patterns
- Use `web::scope()` for grouping related routes
- Return `impl Responder` for async handlers
- Use `serde_json::json!` macro for JSON responses
- Implement `ResponseError` trait for custom errors
- Use middleware for cross-cutting concerns (logging, CORS)

## Resources
- Actix-web docs: https://actix.rs/docs/
- Architecture: `.taskmaster/docs/architecture.md`

## Time Estimate
50 minutes for experienced Rust/Actix-web developer.

## Deliverables
1. `src/api/mod.rs` - Module exports
2. `src/api/routes.rs` - Route configuration
3. `src/api/errors.rs` - Error handling
4. `src/main.rs` - HTTP server
5. `tests/api_routes_test.rs` - Integration tests
6. Updated `Cargo.toml` - Dependencies
