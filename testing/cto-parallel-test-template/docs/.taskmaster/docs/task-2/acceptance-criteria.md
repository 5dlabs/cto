# Acceptance Criteria: API Endpoints Setup

## Core Requirements

### Dependencies
- [ ] `actix-web = "4.3.1"` added to Cargo.toml
- [ ] `serde` with derive features added
- [ ] `serde_json` added
- [ ] `env_logger` and `log` added
- [ ] `cargo check` passes without errors

### File Structure
- [ ] `src/api/mod.rs` exists and exports routes and errors
- [ ] `src/api/routes.rs` exists with route configuration
- [ ] `src/api/errors.rs` exists with ApiError enum
- [ ] `src/main.rs` updated with HTTP server setup

### Route Configuration
- [ ] `configure_routes()` function defined in routes.rs
- [ ] `/api` scope created as root for all endpoints
- [ ] `/api/health` route implemented and functional
- [ ] `/api/auth` scope configured with placeholder routes
- [ ] `/api/users` scope configured
- [ ] `/api/products` scope configured
- [ ] `/api/cart` scope configured

### Health Check Endpoint
- [ ] Returns HTTP 200 OK
- [ ] Returns JSON response with `"status": "ok"`
- [ ] Includes version information
- [ ] Responds to GET requests

### Placeholder Routes
- [ ] All unimplemented routes return HTTP 501 Not Implemented
- [ ] Placeholder responses include JSON error format
- [ ] Error message indicates future implementation
- [ ] Auth routes: `/auth/register` (POST), `/auth/login` (POST)
- [ ] Product routes: `/products` (GET), `/products/{id}` (GET)
- [ ] Cart routes: `/cart` (GET), `/cart/add` (POST), `/cart/remove/{id}` (DELETE), `/cart/clear` (POST)

### Error Handling
- [ ] `ApiError` enum defined with variants: NotFound, BadRequest, InternalError, Unauthorized
- [ ] `ApiError` implements `Display` trait
- [ ] `ApiError` implements `ResponseError` trait
- [ ] Error responses return appropriate HTTP status codes
- [ ] Error responses return JSON with consistent format: `{"error": "type", "message": "details"}`

### HTTP Server
- [ ] Server binds to 127.0.0.1:8080
- [ ] Server starts without errors
- [ ] `#[actix_web::main]` macro used in main.rs
- [ ] Logger middleware configured
- [ ] Routes registered with App configuration

### Logging
- [ ] `env_logger` initialized
- [ ] Logger middleware wraps the App
- [ ] Request logs printed to console
- [ ] Log level configurable via RUST_LOG environment variable

## Functional Tests

### Manual Testing
```bash
# Start server
cargo run
# Expected: Server starts, prints "Starting API server on http://127.0.0.1:8080"

# Test health check
curl http://localhost:8080/api/health
# Expected: {"status":"ok","version":"0.1.0"}

# Test placeholder endpoint
curl http://localhost:8080/api/products
# Expected: 501 Not Implemented with JSON error

# Test 404
curl http://localhost:8080/api/nonexistent
# Expected: 404 Not Found
```

### Automated Testing
- [ ] Integration test file created: `tests/api_routes_test.rs`
- [ ] Test for health check endpoint passes
- [ ] Test for placeholder endpoints returns 501
- [ ] Test for 404 handling (invalid routes)
- [ ] `cargo test` runs all tests successfully

## Code Quality

### Structure
- [ ] Module organization follows Rust conventions
- [ ] Public API clearly defined via mod.rs exports
- [ ] Route handlers use async/await properly
- [ ] No compiler warnings

### Error Handling
- [ ] Errors use Result types where appropriate
- [ ] `.unwrap()` avoided in production code
- [ ] Error messages are descriptive

### Documentation
- [ ] Route structure documented (comments or README)
- [ ] Error types documented
- [ ] Startup message indicates server address

## Integration Points

### Task 1 Integration
- [ ] Imports `schema` module successfully
- [ ] Imports `models` module successfully
- [ ] Imports `config` module successfully
- [ ] No compilation errors related to Task 1 dependencies

### Future Task Compatibility
- [ ] Route scopes align with Task 3 (auth), Task 4 (products), Task 5 (cart)
- [ ] Placeholder routes match expected future implementations
- [ ] Error handling ready for business logic integration

## Non-Functional Requirements

### Performance
- [ ] Server starts in under 5 seconds
- [ ] Health check responds in under 100ms
- [ ] No memory leaks (basic check with running server)

### Security
- [ ] Error messages don't expose sensitive information
- [ ] Server binds to localhost (not 0.0.0.0) for development
- [ ] Ready for CORS middleware addition (Task 6 integration)

### Maintainability
- [ ] Clear separation of concerns (routes, errors, main)
- [ ] Consistent code style
- [ ] Easy to add new routes
- [ ] Error handling patterns established

## Validation Commands

```bash
# Compilation check
cargo check

# Build project
cargo build

# Run tests
cargo test

# Start server
cargo run

# Health check (in separate terminal)
curl -i http://localhost:8080/api/health

# Test placeholder
curl -i http://localhost:8080/api/products

# Test 404
curl -i http://localhost:8080/api/invalid

# Check logs
RUST_LOG=debug cargo run
```

## Success Checklist

### Must Have
- [x] Server starts successfully
- [x] Health check returns 200 OK
- [x] All placeholder routes return 501
- [x] Error handling implemented
- [x] Tests pass
- [x] No compilation errors/warnings

### Should Have
- [x] Logging configured
- [x] Consistent error format
- [x] Integration tests
- [x] Clear route structure

### Nice to Have
- [ ] API documentation (OpenAPI/Swagger) - Future enhancement
- [ ] Request validation middleware - Added in later tasks
- [ ] Rate limiting - Production feature

## Definition of Done
- All "Must Have" criteria met
- `cargo check`, `cargo build`, and `cargo test` succeed
- Server runs without errors for 5+ minutes
- Health check accessible via curl
- Ready for Task 3, 4, 5 integration
- Code reviewed (or self-reviewed if solo)
- Committed to version control
