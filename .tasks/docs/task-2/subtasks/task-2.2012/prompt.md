Implement subtask 2012: Write integration test suite and verify cargo-tarpaulin coverage >= 80%

## Objective
Write sqlx-based integration tests using test transactions for catalog handlers, availability logic, and GDPR endpoints, then run cargo-tarpaulin to confirm >= 80% line coverage.

## Steps
Create tests/integration_test.rs. Use #[sqlx::test] macro for DB tests (automatically creates and tears down a test database with migrations applied). Test cases: (1) test_create_and_list_products: insert category, insert product, call list handler, assert product appears. (2) test_availability_with_block: insert product with stock_quantity=2, insert availability_block for requested window with quantity_booked=2, call availability handler, assert quantity_available=0. (3) test_checkout_atomicity: two concurrent checkout tasks for the same product with quantity=1 and stock=1 — assert exactly one 201 and one 409. (4) test_gdpr_export_and_delete: insert block with customer_id, export, verify, delete, re-export empty. (5) test_rate_limiter_unit: call rate_limit logic with mocked redis responses 100 times success, 101st returns RateLimitExceeded. Add [dev-dependencies] in Cargo.toml: sqlx = {features = ['test']}, tokio = {features = ['test-util']}. Install cargo-tarpaulin: cargo install cargo-tarpaulin. Run: cargo tarpaulin --out Lcov --output-dir coverage/. Assert coverage >= 80% or CI fails.

## Validation
cargo test runs all integration tests with 0 failures. cargo tarpaulin outputs coverage percentage >= 80% in CI logs. Test (3) for checkout atomicity must demonstrate exactly 1 success and 1 conflict when run 10 times consecutively (no flakiness). Test output shows all 5 named test functions as passed.