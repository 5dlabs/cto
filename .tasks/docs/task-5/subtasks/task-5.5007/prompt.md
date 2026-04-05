Implement subtask 5007: Implement REST endpoints for vetting operations

## Objective
Build the Axum route handlers for POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id with request validation and error handling.

## Steps
1. Create `vetting::handlers` module with Axum handlers. 2. POST /api/v1/vetting/run: Accept JSON body with org_id, org_name, jurisdiction, domain (optional). Validate input using serde with custom validation. Invoke the vetting pipeline. Return 202 Accepted with a vetting_run_id if async, or 200 with full VettingResult if synchronous. 3. GET /api/v1/vetting/:org_id: Query PostgreSQL for the latest VettingResult for the given org_id. Return 200 with the result or 404 if no vetting has been performed. 4. GET /api/v1/vetting/credit/:org_id: Query PostgreSQL for the credit-specific portion of the latest VettingResult. Return 200 with CreditResult or 404. 5. Implement consistent error response format: { error: string, code: string, details: Option<Value> }. 6. Add request ID middleware (X-Request-Id header) for traceability. 7. Register all routes on the Axum router under the /api/v1/vetting prefix.

## Validation
Integration tests: POST /vetting/run with valid payload returns vetting result; GET /:org_id returns stored result; GET /credit/:org_id returns credit data; 404 for unknown org_id; 400 for invalid input. Verify request ID propagation.