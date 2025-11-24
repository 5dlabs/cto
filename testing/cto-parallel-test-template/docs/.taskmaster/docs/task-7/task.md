# Task 7: Integration Tests

## Overview
Create comprehensive integration tests covering all application components, APIs, and end-to-end user flows.

## Context
**Level 2 task** depending on Tasks 2 (API), 5 (Cart), and 6 (Frontend). Final implementation task before integration validation.

## Objectives
1. Create integration test suite for API endpoints
2. Test authentication flows
3. Test shopping cart operations
4. Verify component integration
5. Create end-to-end user flow tests

## Dependencies
- **Task 2:** API Endpoints
- **Task 5:** Shopping Cart API
- **Task 6:** Frontend Components

## Implementation Plan

### Step 1: Set Up Test Infrastructure
Create `tests/` directory with:
- `integration_tests.rs` - E2E flows
- `api_tests.rs` - API endpoint tests
- `auth_tests.rs` - Authentication tests
- `common/mod.rs` - Test utilities

### Step 2: Implement Test Utilities
```rust
// tests/common/mod.rs
pub mod test_utils {
    use actix_web::{test, web, App};

    pub async fn get_test_app() -> impl actix_web::dev::Service {
        // Initialize services and return test app
    }

    pub fn get_test_services() -> (web::Data<ProductService>, web::Data<CartService>) {
        // Return initialized services
    }
}
```

### Step 3: Create API Tests
Test suites for:
- Product CRUD operations
- Product filtering
- Error handling

### Step 4: Create Auth Tests
Test JWT creation, validation, password hashing

### Step 5: Create Integration Tests
End-to-end tests for:
- Complete shopping flow
- User registration and login
- Cart management

## Testing Strategy
```bash
cargo test                    # Run all tests
cargo test --test integration_tests
cargo test --test api_tests
cargo test --test auth_tests
```

## Success Criteria
- [ ] All test files created in `tests/` directory
- [ ] API endpoint tests cover all routes
- [ ] Auth tests verify JWT and passwords
- [ ] Integration tests cover full user flows
- [ ] `cargo test` passes all tests
- [ ] Tests are independent (no shared state)

## Files Created
- `tests/integration_tests.rs`
- `tests/api_tests.rs`
- `tests/auth_tests.rs`
- `tests/common/mod.rs`
