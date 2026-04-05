Implement subtask 5006: Implement vetting pipeline orchestration and composite scoring algorithm

## Objective
Build the vetting pipeline that orchestrates all four integration modules (business verification, online presence, reputation, credit signals) and computes the composite LeadScore.

## Steps
1. Create a `vetting::pipeline` module that accepts an org_id and org metadata (name, jurisdiction, domain). 2. Execute all four checks concurrently using tokio::join! or futures::join_all for parallel execution. 3. Collect results from: BusinessVerificationProvider, OnlinePresenceProvider, ReputationProvider, CreditProvider. 4. Implement the composite scoring algorithm: composite_score = (business_verification_weight * bv_score) + (online_presence_weight * op_score) + (reputation_weight * rep_score) + (credit_weight * credit_score). Default weights: bv=0.25, op=0.20, rep=0.25, credit=0.30 (make configurable via environment variables). 5. Build a VettingResult containing all individual results plus the composite score and scoring breakdown. 6. Persist the VettingResult and LeadScore to PostgreSQL within a transaction. 7. Handle partial failures: if one integration fails, still compute a partial score with reduced weights and flag which checks failed. 8. Return the completed VettingResult.

## Validation
Integration test with all mock providers: verify pipeline runs all checks concurrently, computes correct composite score, persists to DB. Test partial failure scenarios: one provider fails, two providers fail. Verify weight recalculation on partial failure.