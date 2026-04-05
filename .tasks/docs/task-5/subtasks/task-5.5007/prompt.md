Implement subtask 5007: Implement vetting pipeline orchestration and composite risk scoring

## Objective
Build the core vetting pipeline that orchestrates all four integration modules (OpenCorporates, LinkedIn, Google Reviews, Credit) in parallel, aggregates their results, computes a composite risk score, derives a lead qualification tier, and persists VettingResult and LeadScore to the database.

## Steps
1. Create src/pipeline/mod.rs and src/pipeline/vetting_pipeline.rs.
2. Define `VettingPipeline` struct that holds references to all four integration trait objects (dyn BusinessVerifier, dyn OnlinePresenceChecker, dyn ReputationChecker, dyn CreditChecker) and a PgPool.
3. Implement `run_vetting(&self, org_id: Uuid, company_name: &str, ...) -> Result<(VettingResult, LeadScore), VettingError>`.
4. Use tokio::join! to run all four integration checks concurrently.
5. Composite risk score formula: business_verification (0.25 weight) + linkedin_presence (0.20) + google_reputation (0.25) + credit_signals (0.30). If business verification fails entirely, cap score at 0.3.
6. Lead qualification tiers: >= 0.75 -> Hot, >= 0.50 -> Warm, >= 0.25 -> Cold, < 0.25 -> Disqualified.
7. Build scoring_factors JSONB from individual scores and metadata.
8. Persist VettingResult and LeadScore to database using sqlx transactions.
9. Handle partial failures gracefully (some APIs may fail; use available data with reduced confidence).

## Validation
Unit tests with all mock integrations verify correct composite scoring for various scenarios: all-green company scores Hot, mixed signals score Warm/Cold, all-negative scores Disqualified; partial API failures are handled gracefully without panicking; database persistence is verified in integration tests.