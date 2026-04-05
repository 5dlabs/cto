Implement subtask 5006: Implement GREEN/YELLOW/RED scoring algorithm with aggregation logic

## Objective
Build the scoring engine that aggregates data from all four external sources (OpenCorporates, LinkedIn, Google Reviews, credit) and computes a composite vetting score classified as GREEN, YELLOW, or RED.

## Steps
1. Create `src/scoring.rs` module.
2. Define scoring dimensions with weights: business_verification (e.g., 30%), online_presence (e.g., 20%), reputation (e.g., 25%), credit (e.g., 25%). Make weights configurable.
3. Implement per-dimension scoring functions:
   - `score_business_verification(data: &BusinessVerificationData) -> DimensionScore` — considers registration status, age, officer count, filing recency.
   - `score_online_presence(data: &LinkedInProfile) -> DimensionScore` — considers employee count, follower count, description completeness.
   - `score_reputation(data: &ReputationData) -> DimensionScore` — considers average rating, review count, sentiment.
   - `score_credit(data: &CreditReport) -> DimensionScore` — considers credit score, risk level, payment history, judgments.
4. Each DimensionScore contains a normalized 0.0-1.0 score and a confidence level.
5. Implement `compute_composite_score(dimensions: &[DimensionScore]) -> CompositeScore` that applies weights and produces a final 0.0-1.0 score.
6. Implement classification: GREEN (>= 0.7), YELLOW (0.4-0.69), RED (< 0.4). Thresholds should be configurable.
7. Handle missing dimensions gracefully — redistribute weight proportionally among available dimensions.
8. Return a LeadScore struct with all dimension scores, composite score, and classification.

## Validation
Write comprehensive unit tests covering: all-green scenario, all-red scenario, mixed scenario, missing dimensions, edge cases at threshold boundaries. Verify weight redistribution works correctly when one or more data sources are unavailable.