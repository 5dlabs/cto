Implement subtask 5008: Implement API endpoints for vetting service

## Objective
Implement the three Axum HTTP endpoints: POST /api/v1/vetting/run (trigger a vetting pipeline run), GET /api/v1/vetting/:org_id (retrieve vetting results for an organization), and GET /api/v1/vetting/credit/:org_id (retrieve credit-specific data for an organization). Wire up the pipeline and database queries to the router.

## Steps
1. Create src/handlers/mod.rs and src/handlers/vetting.rs.
2. POST /api/v1/vetting/run: Accept JSON body with org_id, company_name, jurisdiction, domain. Invoke VettingPipeline::run_vetting. Return 201 with VettingResult and LeadScore. Handle validation errors (400), internal errors (500).
3. GET /api/v1/vetting/:org_id: Query database for the latest VettingResult and LeadScore by org_id. Return 200 with results or 404 if not found.
4. GET /api/v1/vetting/credit/:org_id: Query database for credit-specific fields from VettingResult. Return 200 or 404.
5. Define request/response DTOs with serde Serialize/Deserialize.
6. Add proper error handling with a shared AppError type that maps to HTTP status codes.
7. Wire all routes into the main Axum Router with shared state (PgPool, pipeline dependencies).
8. Add request tracing/logging middleware.

## Validation
Integration tests using axum::test helpers verify: POST /vetting/run returns 201 with valid scores; GET /vetting/:org_id returns stored results; GET /vetting/credit/:org_id returns credit data; invalid requests return 400; missing org returns 404.