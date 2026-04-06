Implement subtask 2005: Generate OpenAPI specification and write integration tests

## Objective
Document all Equipment Catalog API endpoints with an OpenAPI 3.0 specification and write comprehensive integration tests covering all endpoints, filtering, pagination, error cases, and rate limiting.

## Steps
1. Add the `utoipa` crate (or equivalent) to auto-generate OpenAPI specs from Axum handler annotations.
2. Annotate all endpoint handlers with OpenAPI metadata: path, method, query params, request/response bodies, status codes, descriptions.
3. Serve the OpenAPI JSON at `GET /api/v1/catalog/openapi.json`.
4. Create an `tests/` directory with integration test files.
5. Write integration tests using reqwest or axum::test:
   - test_categories_list: verify categories are returned
   - test_products_list_pagination: verify page, per_page, total_pages
   - test_products_filter_by_category: verify filtering works
   - test_products_search: verify ILIKE search
   - test_product_detail: verify single product with image URLs
   - test_product_not_found: verify 404
   - test_availability_query: verify date range filtering
   - test_equipment_api_catalog: verify simplified agent format
   - test_equipment_api_checkout_success: verify reservation creation
   - test_equipment_api_checkout_conflict: verify 409 on double booking
   - test_rate_limiting: verify 429 after threshold
   - test_health_endpoints: verify live/ready/metrics
6. Ensure tests use a test database (or transactions that roll back).
7. Add a CI-ready test command in Cargo.toml or Makefile.

## Validation
All integration tests pass (`cargo test`); OpenAPI spec at /api/v1/catalog/openapi.json is valid (passes openapi-spec-validator); spec documents all 8+ endpoints with correct schemas, parameters, and response codes; test coverage includes happy paths, error cases, pagination edge cases, and rate limiting.