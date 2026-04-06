Implement subtask 4010: Generate OpenAPI specification and write integration tests

## Objective
Generate an OpenAPI 3.0 spec for all Finance service endpoints using utoipa, and write comprehensive integration tests covering all routes with >80% code coverage.

## Steps
1. Add utoipa and utoipa-swagger-ui dependencies.
2. Annotate all Axum handlers and DTOs with utoipa macros (#[utoipa::path], #[derive(ToSchema)]).
3. Register all paths in an OpenApi struct and serve Swagger UI at /docs and OpenAPI JSON at /v1/openapi.json.
4. Verify the generated spec includes all endpoints, request/response schemas, error responses, and authentication requirements.
5. Write integration tests using axum::test helpers or reqwest against a test server:
   - Invoice lifecycle: create draft → update → send → pay via Stripe webhook → verify paid status
   - Payment recording: manual payment, Stripe payment, refund
   - AR aging report accuracy with various invoice states and due dates
   - Revenue report with date range and grouping
   - Payroll lifecycle: create → approve → pay
   - Currency sync and conversion accuracy
   - Tax calculation for each jurisdiction
   - Payment reminder job behavior
6. Use testcontainers-rs or a docker-compose setup for PostgreSQL and Redis in tests.
7. Run cargo tarpaulin or llvm-cov to measure coverage and verify >80% threshold.
8. Ensure the OpenAPI spec is Effect-compatible (standard JSON request/response, clear error schemas) for frontend consumption.

## Validation
OpenAPI spec is valid (passes swagger-cli validate); all endpoints are documented in the spec; Swagger UI loads at /docs; integration tests cover the full invoice lifecycle, Stripe payment flow, all report types, payroll lifecycle, currency operations, tax calculations, and reminders; cargo tarpaulin reports >80% line coverage; tests pass in CI with containerized dependencies.