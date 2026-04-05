Implement subtask 5007: Implement vetting REST endpoints and orchestration pipeline

## Objective
Build the Axum REST endpoints (/api/v1/vetting/run, /api/v1/vetting/:org_id, /api/v1/vetting/credit/:org_id) and the orchestration logic that calls all integration modules, runs scoring, and persists results.

## Steps
1. Create `src/routes/vetting.rs` module.
2. Implement POST `/api/v1/vetting/run` endpoint:
   - Accept JSON body with org_id, org_name, optional jurisdiction and location hints.
   - Orchestrate parallel calls to OpenCorporates, LinkedIn, Google Reviews, and credit modules using tokio::join! or tokio::JoinSet.
   - Pass aggregated data to the scoring engine.
   - Persist VettingResult and LeadScore to PostgreSQL using sqlx.
   - Return the VettingResult with score and classification in the response.
3. Implement GET `/api/v1/vetting/:org_id` endpoint:
   - Query PostgreSQL for the latest VettingResult for the given org_id.
   - Return 404 if no result found.
4. Implement GET `/api/v1/vetting/credit/:org_id` endpoint:
   - Query for credit-specific data from the latest vetting result.
   - Return structured credit data and score.
5. Add request validation with proper error responses (400 for bad input, 404 for not found, 500 for internal errors).
6. Use Axum extractors and State for dependency injection of database pool and API clients.
7. Wire all routes into the main Axum router.

## Validation
Write integration tests using a test database and mocked external APIs. Verify POST /vetting/run creates a record and returns correct score. Verify GET /vetting/:org_id retrieves persisted data. Verify GET /vetting/credit/:org_id returns credit-specific data. Test error cases (invalid input, missing org).