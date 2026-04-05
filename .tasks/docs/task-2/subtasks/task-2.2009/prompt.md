Implement subtask 2009: Generate OpenAPI specification and integration tests

## Objective
Generate an OpenAPI 3.0 specification for all catalog endpoints and write integration tests verifying endpoint behavior against the spec.

## Steps
1. Add utoipa crate to Cargo.toml for OpenAPI generation from Rust code. 2. Annotate all handler functions and request/response structs with #[utoipa::path(...)] and #[derive(ToSchema)] macros. 3. Create an OpenAPI doc builder in main.rs that aggregates all paths and serves at GET /api/v1/openapi.json. 4. Optionally serve Swagger UI at /api/v1/docs using utoipa-swagger-ui. 5. Write integration tests in tests/integration/: test_categories_listing, test_products_pagination, test_product_detail, test_product_not_found, test_availability_check, test_equipment_api_catalog, test_equipment_api_checkout, test_checkout_idempotency, test_rate_limiting. 6. Use sqlx test fixtures or a test database with seed data. 7. Each test should assert response status codes, JSON structure, and key field values.

## Validation
GET /api/v1/openapi.json returns valid OpenAPI 3.0 JSON that passes validation (e.g., via swagger-cli validate); all integration tests pass with 'cargo test'; tests cover happy paths, error cases, and edge cases for each endpoint.