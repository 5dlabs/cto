Implement subtask 5009: Implement REST endpoints for vetting operations

## Objective
Create the three REST API endpoints: POST /api/v1/vetting/run, GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id with proper request validation and response formatting.

## Steps
1. Create a `routes/vetting.rs` module.
2. Implement `POST /api/v1/vetting/run`:
   - Accept JSON body: `{ "org_id": "uuid", "company_name": "string", "jurisdiction": "optional string", "website": "optional string" }`.
   - Validate input (org_id required, company_name non-empty).
   - Call `VettingPipeline::run` and return the `VettingResult` as JSON with HTTP 201.
   - Return HTTP 400 for validation errors, 500 for internal errors.
3. Implement `GET /api/v1/vetting/:org_id`:
   - Look up the latest VettingResult for the given org_id.
   - Return HTTP 200 with the full VettingResult or HTTP 404 if not found.
4. Implement `GET /api/v1/vetting/credit/:org_id`:
   - Look up the latest VettingResult and return only the credit_signals portion.
   - Return HTTP 200 or HTTP 404.
5. Register all routes on the Axum router with shared AppState.
6. Use `axum::extract::Json`, `Path`, and `State` extractors consistently.

## Validation
POST /api/v1/vetting/run with valid input returns HTTP 201 with a complete VettingResult. GET /api/v1/vetting/:org_id returns HTTP 200 with stored data. GET /api/v1/vetting/credit/:org_id returns only credit signals. Invalid inputs return HTTP 400. Non-existent org_id returns HTTP 404.