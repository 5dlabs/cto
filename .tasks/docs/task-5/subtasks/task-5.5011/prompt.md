Implement subtask 5011: Implement API endpoints (POST run, GET result, GET credit, DELETE GDPR)

## Objective
Build all Axum 0.7 route handlers: POST /api/v1/vetting/run (202 Accepted with async pipeline), GET /api/v1/vetting/:org_id, GET /api/v1/vetting/credit/:org_id, DELETE /api/v1/vetting/:org_id for GDPR.

## Steps
1. Create `src/routes/vetting.rs`.
2. `POST /api/v1/vetting/run`:
   - Parse JSON body: `{ org_id: UUID, org_name: String, org_domain: Option<String> }`.
   - Validate required fields.
   - Insert vetting_request row with status='pending'.
   - `tokio::spawn` the pipeline with request_id, org_id, org_name, org_domain.
   - Return 202 Accepted with `{ request_id: UUID, status: "pending" }`.
3. `GET /api/v1/vetting/:org_id`:
   - Check Valkey cache first.
   - If miss, query DB for latest vetting_result by org_id.
   - If found, return 200 with serialized VettingResult.
   - If not found, return 404 with error body.
4. `GET /api/v1/vetting/credit/:org_id`:
   - Query latest vetting_result for org_id.
   - Return subset: `{ credit_score, credit_data_unavailable: bool }`.
   - 404 if no result exists.
5. `DELETE /api/v1/vetting/:org_id`:
   - Delete all rows from vetting_results and vetting_requests for org_id.
   - Invalidate Valkey cache.
   - Return 204 No Content.
6. Register all routes on Axum Router with shared AppState.
7. Apply shared-auth middleware to all endpoints.
8. Add request/response models with serde Serialize/Deserialize.
9. Add `GET /metrics` endpoint exposing Prometheus metrics from shared-observability.

## Validation
Integration test: POST /vetting/run returns 202 with request_id. GET /vetting/:org_id after pipeline completes returns 200 with full result. GET /vetting/credit/:org_id returns credit subset. DELETE /vetting/:org_id returns 204, subsequent GET returns 404. Test: invalid POST body returns 400. Test: GET for non-existent org returns 404.