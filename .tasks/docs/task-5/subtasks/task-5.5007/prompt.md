Implement subtask 5007: Implement vetting pipeline orchestration and scoring algorithm

## Objective
Build the vetting pipeline that orchestrates all four verification stages (business verification, online presence, reputation, credit) and combines their results into a final GREEN/YELLOW/RED lead score.

## Steps
1. In `services/vetting_pipeline.rs`, create a VettingPipeline struct that holds references to all four integration clients (via trait objects). 2. Implement `run_vetting(org_id: UUID, org_name: &str) -> Result<VettingResult>` that: a) Executes all four stages (consider parallel execution with tokio::join! or sequential based on dp-5-2 decision), b) Collects results from each stage, c) Passes all results to the scoring algorithm. 3. In `services/scoring.rs`, implement the scoring algorithm: a) Assign weights to each component (e.g., business_verification: 30%, online_presence: 15%, reputation: 25%, credit: 30%), b) Normalize each component score to 0-100, c) Calculate weighted total, d) Map to classification: GREEN (>=70), YELLOW (40-69), RED (<40). 4. Store the VettingResult and LeadScore in PostgreSQL using sqlx. 5. Handle partial failures (e.g., one API is down) gracefully — score with available data and flag incomplete components.

## Validation
Unit tests verify scoring algorithm with known inputs produce correct GREEN/YELLOW/RED classifications; pipeline handles partial failures (one integration returns error) and still produces a result with degraded flag; weights sum to 100%; boundary values (69.5, 39.5) are tested.