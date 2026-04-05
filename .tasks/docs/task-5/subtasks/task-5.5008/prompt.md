Implement subtask 5008: Implement weighted scoring algorithm and risk_flags population

## Objective
Build the scoring module that takes raw vetting data from all pipeline steps and computes the weighted GREEN/YELLOW/RED score with risk_flags array.

## Steps
1. Create `src/scoring/mod.rs`.
2. Define `VettingSignals` input struct:
   - business_verified: bool
   - linkedin_exists: bool
   - linkedin_followers: i32
   - google_reviews_rating: Option<f32>
   - google_reviews_count: i32
   - credit_score: Option<i32>
3. Define `ScoringResult` output struct:
   - total_points: i32
   - final_score: FinalScore enum (GREEN, YELLOW, RED)
   - risk_flags: Vec<String>
4. Implement `pub fn calculate_score(signals: &VettingSignals) -> ScoringResult`:
   - Start with 0 points, empty risk_flags.
   - business_verified → +30 points; else add "business_not_verified" flag.
   - linkedin_exists && followers > 50 → +20 points; else add "no_linkedin_presence" or "low_linkedin_followers" flag.
   - google_reviews_rating >= 4.0 → +20 points; else if rating is Some but < 4.0 add "low_review_rating"; if None add "no_google_reviews".
   - google_reviews_count >= 10 → +10 points; else add "insufficient_review_count".
   - credit_score >= 600 → +20 points; credit_score < 600 → add "low_credit_score"; credit_score None but total of other signals >= 50 → +10 points (partial credit); credit_score None → add "credit_data_unavailable".
   - GREEN: >= 70, YELLOW: 40-69, RED: < 40.
5. Write comprehensive unit tests with at least 7 test vectors:
   - All positive signals → GREEN (100 pts)
   - Verified + LinkedIn + reviews but no credit → GREEN (80+10=90 or 80)
   - Verified only → YELLOW (30 pts? — actually RED, need to be precise)
   - Nothing verified → RED (0 pts)
   - Partial: verified + low linkedin + good reviews + no credit
   - Edge case: exactly 70 pts → GREEN
   - Edge case: exactly 40 pts → YELLOW
   - Edge case: 39 pts → RED

## Validation
At least 7 unit test vectors covering: all-positive (GREEN, 100pts), all-negative (RED, 0pts), business_verified only (RED, 30pts), verified+linkedin+reviews no credit (score depends on partial credit logic), edge at 70 (GREEN), edge at 40 (YELLOW), edge at 39 (RED). Verify risk_flags array contains exactly the expected flags for each scenario.