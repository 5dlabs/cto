Implement subtask 5002: Implement REST endpoint scaffolding and async pipeline dispatch

## Objective
Build the Axum router with POST /api/v1/vetting/run (202 Accepted), GET /api/v1/vetting/:org_id, and GET /api/v1/vetting/credit/:org_id endpoints. Wire up the 202 pattern that spawns the vetting pipeline as a background tokio task and updates vetting_requests status.

## Steps
1. Create `src/main.rs` with Axum app setup: shared state holding DB pool (from shared crate), reqwest::Client, and circuit breaker state.
2. Implement `POST /api/v1/vetting/run` handler: validate JSON body (org_id, company_name, domain). Insert a new row in `vetting_requests` with status='pending'. Spawn a `tokio::spawn` background task that will call the pipeline orchestrator. Return 202 Accepted with the request ID.
3. Implement `GET /api/v1/vetting/:org_id` handler: query `vetting_results` by org_id, return latest result as JSON or 404.
4. Implement `GET /api/v1/vetting/credit/:org_id` handler: query `vetting_results` and return only credit_score and related fields subset.
5. Define the pipeline orchestrator function signature that takes org_id, company_name, domain, db_pool, http_client. It updates vetting_requests status to 'running' at start, 'completed'/'failed' at end.
6. Wire shared crate middleware: health check at `/health`, metrics at `/metrics`, API key auth extractor.
7. Ensure proper error handling with consistent JSON error responses (AppError type with IntoResponse).

## Validation
Unit test: POST /run with valid body returns 202 and a UUID request_id. GET /:org_id with no data returns 404. GET /credit/:org_id with no data returns 404. Integration test: POST /run, wait briefly, GET /:org_id returns result (with mocked pipeline). Verify vetting_requests status transitions from pending→running→completed.