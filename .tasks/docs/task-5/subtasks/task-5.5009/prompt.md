Implement subtask 5009: Build async pipeline orchestrator with concurrent stage execution

## Objective
Implement the pipeline orchestrator that runs stages 1-4 concurrently via tokio::join!, feeds results into the scoring stage, persists VettingResult to the database, and updates VettingRequest status throughout.

## Steps
1. Create `src/pipeline.rs` module.
2. Implement `run_vetting_pipeline(org_id: Uuid, company_name: String, domain: String, pool: &PgPool, client: &ResilientClient, credit_provider: &dyn CreditProvider, request_id: Uuid)`.
3. Update `vetting_requests` row to status='running', started_at=now().
4. Execute stages 1-4 concurrently using `tokio::join!`:
   ```rust
   let (biz, online, reputation, credit) = tokio::join!(
       run_business_verification(&client, &company_name, &api_key),
       run_online_presence(&client, &company_name, &domain),
       run_reputation(&client, &company_name, &api_key),
       credit_provider.check_credit(&company_name, &domain)
   );
   ```
5. Collect results, build ScoringInput, run scoring algorithm.
6. Aggregate all raw_responses from each stage into a single JSONB value.
7. Insert/upsert VettingResult into `vetting_results` table with all fields populated.
8. Update `vetting_requests` to status='completed', completed_at=now().
9. On any panic or unrecoverable error, catch with proper error handling and set status='failed' with error_message.
10. Use tracing spans for each stage for observability.

## Validation
Integration test with wiremock-rs: mock all 4 external APIs, call run_vetting_pipeline, verify VettingResult is persisted with correct fields. Verify VettingRequest transitions pending→running→completed. Test error path: mock all APIs to fail, verify status='completed' (not failed — stages degrade gracefully), score is RED. Test panic recovery: verify status='failed' with error message on unexpected error.