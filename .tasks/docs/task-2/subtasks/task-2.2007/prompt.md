Implement subtask 2007: Write comprehensive integration and unit tests for all endpoints

## Objective
Create a full test suite covering all catalog and equipment-api endpoints, error cases, pagination, rate limiting, and health probes to achieve at least 80% code coverage.

## Steps
1. Create tests/ directory with integration test files: tests/catalog_test.rs, tests/equipment_api_test.rs, tests/health_test.rs, tests/rate_limit_test.rs.
2. Use sqlx::test macro with a test database that runs migrations automatically.
3. Write a test helper module (tests/common/mod.rs) that creates an AppState with test database, spawns the Axum app on a random port, and provides an HTTP client.
4. Catalog tests:
   - List categories returns seeded data, respects parent_id filter.
   - List products returns paginated results, search filter works, category_id filter works.
   - Get product by valid ID returns full product with images.
   - Get product by invalid UUID returns 400, by non-existent UUID returns 404.
   - Availability endpoint returns correct computed available quantities.
   - Availability endpoint returns 400 when dates are missing or invalid.
5. Equipment API tests:
   - Catalog endpoint returns denormalized structure with CDN image URLs.
   - Checkout with valid items returns a quote with correct totals.
   - Checkout with insufficient availability returns appropriate error.
   - Checkout with invalid product_id returns 404.
6. Health tests: /health/live returns 200, /health/ready returns 200 with connected infra.
7. Rate limit tests: verify 429 is returned after exceeding limit.
8. Add cargo-tarpaulin to CI for coverage reporting, targeting ≥80%.

## Validation
cargo test --all runs all tests and they pass; cargo tarpaulin reports ≥80% line coverage; each test file covers both happy-path and error scenarios; tests are isolated and can run in parallel without interference.