Implement subtask 5008: Implement API endpoints for vetting operations

## Objective
Build the three Axum HTTP endpoints: POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, GET /api/v1/vetting/credit/:org_id with request validation and proper error responses.

## Steps
1. In `handlers/vetting.rs`, implement: a) `POST /api/v1/vetting/run` — accepts JSON body with org_id and org_name, triggers the vetting pipeline, returns 202 Accepted with a vetting_result_id (or 200 with full result if synchronous). b) `GET /api/v1/vetting/:org_id` — retrieves the latest VettingResult for the given org_id from PostgreSQL, returns 200 with full result or 404 if not found. c) `GET /api/v1/vetting/credit/:org_id` — retrieves credit-specific signals for the org_id, returns 200 with CreditSignals or 404. 2. Define request/response DTOs with serde. 3. Add input validation (valid UUID for org_id, non-empty org_name). 4. Return proper HTTP error codes: 400 for validation errors, 404 for not found, 500 for internal errors. 5. Wire all routes into the Axum router in main.rs with shared state (PgPool, pipeline service).

## Validation
Integration tests: POST /vetting/run with valid payload returns 200/202 and stores result in DB; GET /vetting/:org_id returns stored result; GET /vetting/credit/:org_id returns credit data; invalid org_id returns 400; non-existent org_id returns 404.