Implement subtask 5009: Implement async vetting pipeline orchestrator

## Objective
Build the async pipeline that orchestrates the 5 vetting steps (OpenCorporates, LinkedIn, Google Places, Credit, Scoring), updates request status, stores results, and handles partial failures gracefully.

## Steps
1. Create `src/pipeline/mod.rs`.
2. Define `VettingPipeline` struct holding references to all four clients, PgPool, and Valkey connection.
3. Implement `pub async fn run(&self, request_id: Uuid, org_id: Uuid, org_name: &str, org_domain: Option<&str>) -> Result<(), VettingError>`:
   - Update vetting_requests status to 'in_progress'.
   - Step 1: Call `opencorporates_client.search_company(org_name)`. Handle Ok/Err, store partial result.
   - Step 2: Call `linkedin_client.lookup_company(org_name, org_domain)`. Handle Ok/Err.
   - Step 3: Call `google_places_client.search_business(org_name, org_domain)`. Handle Ok/Err.
   - Step 4: Call `credit_client.get_credit_score(org_name, org_domain)`. Handle Ok/Err.
   - Step 5: Assemble `VettingSignals` from steps 1-4, call `calculate_score`.
   - Build `VettingResult` row and insert via repository.
   - Update vetting_requests status to 'completed', set completed_at.
   - Cache result in Valkey (separate subtask handles caching logic).
   - On any unrecoverable error, update status to 'failed'.
4. Each step catches its own errors and produces default/empty signals so the pipeline always completes (graceful degradation).
5. Log each step's outcome at info level with org_id context via tracing spans.
6. This function is called from a `tokio::spawn` in the route handler (next subtask).

## Validation
Integration test: with all mocked clients returning success, pipeline completes, vetting_requests status is 'completed', vetting_results row exists with correct data. Integration test: all clients return errors, pipeline still completes with status 'completed', result has RED score and all risk_flags populated. Test: request status transitions from pending → in_progress → completed (or failed).