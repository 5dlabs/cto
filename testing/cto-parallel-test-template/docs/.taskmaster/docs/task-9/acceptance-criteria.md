# Acceptance Criteria: Integration - Level 1

## Server Integration
- [ ] `cargo run` starts server without errors
- [ ] Server binds to port 8080
- [ ] All routes registered correctly
- [ ] Services initialized properly

## API Endpoint Validation
- [ ] Health check returns 200 OK
- [ ] Product endpoints accessible
- [ ] Cart endpoints require JWT
- [ ] Auth endpoints (placeholders) respond

## Cart Integration
- [ ] Cart endpoints validate JWT tokens
- [ ] Cart uses ProductService for validation
- [ ] Inventory checked before adding items
- [ ] Cart isolated per user

## Service Integration
- [ ] CartService integrates with ProductService
- [ ] JWT validation works in cart routes
- [ ] No service initialization errors

## Test Validation
- [ ] `cargo test` passes all tests
- [ ] Integration tests pass
- [ ] API tests pass
- [ ] No test failures

## Route Validation
- [ ] No route conflicts
- [ ] All scopes (/api/cart, /api/products) work
- [ ] Correct HTTP methods mapped
- [ ] Error responses consistent

## Definition of Done
Level 1 tasks fully integrated with Level 0. System ready for Level 2 (Task 7).
