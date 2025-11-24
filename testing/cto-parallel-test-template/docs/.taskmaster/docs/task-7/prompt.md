# Autonomous Agent Prompt: Integration Tests

## Mission
Create comprehensive integration test suite covering all APIs, authentication, and end-to-end user flows.

## Implementation Steps

1. **Create test infrastructure** in `tests/` directory
2. **Build test utilities** for app initialization
3. **Write API tests** for products, cart, auth endpoints
4. **Write authentication tests** for JWT and passwords
5. **Create integration tests** for complete user flows

## Key Test Scenarios

### API Tests
- Product listing, retrieval, creation
- Cart operations with authentication
- Error responses (404, 401, 400)

### Auth Tests
- JWT token creation and validation
- Password hashing and verification
- Token expiration

### Integration Tests
- Full shopping flow: browse → add to cart → checkout
- User registration → login → authenticated cart access

## Success Criteria
```bash
cargo test
# All tests should pass
```

Use Actix-web test utilities for HTTP testing.
Ensure tests are independent and repeatable.
