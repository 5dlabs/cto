Implement subtask 5008: Build vetting pipeline orchestrator

## Objective
Implement the pipeline orchestrator that sequentially or concurrently runs all vetting stages, collects results, invokes the scoring engine, and persists the final VettingResult.

## Steps
1. Create a `pipeline.rs` module.
2. Define a `VettingPipeline` struct that holds references to all stage implementations and the scoring engine.
3. Implement `async fn run(&self, org_id: Uuid, input: VettingInput, repo: &VettingRepository) -> Result<VettingResult>` that:
   - Creates an initial `VettingResult` record with status 'in_progress' in the database.
   - Runs all four stages concurrently using `tokio::join!` (they are independent external API calls).
   - Collects results, handling individual stage failures gracefully (mark stage as failed but continue).
   - Passes all stage results to the `ScoringEngine` to compute the composite score.
   - Updates the `VettingResult` record with all stage outputs, composite score, and status 'completed' (or 'partial' if some stages failed).
4. Add structured tracing (tracing::info/warn) for each pipeline stage start/completion/failure.
5. Implement timeout for the entire pipeline (30s max).

## Validation
Integration test with mocked external APIs verifies the full pipeline runs, stages execute concurrently, partial failures are handled (pipeline completes with 'partial' status), and the final VettingResult is persisted correctly with composite score.