Implement subtask 2010: Write integration tests for all catalog endpoints

## Objective
Create comprehensive integration tests covering all Equipment Catalog Service endpoints with database seeding, happy paths, and error cases.

## Steps
1. Create a tests/ directory with integration test files.
2. Set up test infrastructure: use sqlx::test or a test database with migrations applied before each test.
3. Create seed data fixtures: sample categories (3+), products (5+), availability records.
4. Write tests for each endpoint:
   - test_list_categories: verify response structure, tree mode.
   - test_list_products: verify pagination, filtering by category, search.
   - test_get_product: verify detail response, 404 for missing.
   - test_get_availability: verify date range, 404 for missing product.
   - test_agent_catalog: verify machine-readable format.
   - test_agent_checkout: verify reservation creation, 409 on double-booking.
   - test_rate_limiting: verify 429 after exceeding limit.
   - test_health_endpoints: verify /healthz, /readyz, /metrics.
5. Aim for >80% code coverage.
6. Add a CI-compatible test command in Cargo.toml or Makefile.

## Validation
All integration tests pass with `cargo test`. Code coverage report shows >80% line coverage. Tests cover happy paths, error cases (404, 409, 429), and edge cases (empty results, invalid parameters). Tests are idempotent and can run in CI.