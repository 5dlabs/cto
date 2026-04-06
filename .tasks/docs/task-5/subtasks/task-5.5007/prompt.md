Implement subtask 5007: Wire up API endpoints and orchestrate vetting pipeline

## Objective
Implement the three REST endpoints (/api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id) that orchestrate the full vetting pipeline, persist results, and serve stored vetting data.

## Steps
1. Implement POST `/api/v1/vetting/run` handler:
   - Accept JSON body with org_id, company_name, optional jurisdiction/location.
   - Call all four integration clients concurrently using tokio::join! or futures::join_all.
   - Pass collected data through the scoring algorithm.
   - Persist VettingResult and LeadScore to PostgreSQL via sqlx.
   - Return the VettingResult with score in the response.
2. Implement GET `/api/v1/vetting/:org_id` handler:
   - Query PostgreSQL for the latest VettingResult for the given org_id.
   - Return 404 if not found.
3. Implement GET `/api/v1/vetting/credit/:org_id` handler:
   - Query PostgreSQL for the credit-specific portion of the vetting result.
   - Return structured credit data and score.
4. Add input validation on all endpoints (validate org_id format, required fields).
5. Add proper error handling with structured JSON error responses.
6. Register all routes in the Axum router with appropriate middleware (tracing, CORS).

## Validation
Integration tests: POST /vetting/run with valid data returns 200 with VettingResult containing a valid GREEN/YELLOW/RED score; GET /vetting/:org_id returns persisted result; GET /vetting/credit/:org_id returns credit data; invalid input returns 400; non-existent org returns 404. End-to-end vetting completes within 10 seconds with mocked external APIs.