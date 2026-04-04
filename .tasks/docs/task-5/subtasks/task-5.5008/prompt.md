Implement subtask 5008: Implement Stage 5: Final scoring algorithm with weighted point system

## Objective
Build the scoring algorithm that takes results from all 5 stages, applies the weighted 100-point system (30/20/20/20/10), and produces a GREEN/YELLOW/RED classification.

## Steps
1. Create `src/stages/scoring.rs` module.
2. Define `ScoringInput` struct aggregating results from all stages: BusinessVerificationResult, OnlinePresenceResult, ReputationResult, CreditResult, plus a risk_flags Vec<String>.
3. Implement `calculate_score(input: &ScoringInput) -> (i32, String, Vec<String>)` returning (total_points, final_score_label, breakdown).
4. Scoring logic:
   - business_verified == true → +30 points
   - linkedin_exists == true AND linkedin_followers > 50 → +20 points
   - google_reviews_rating >= 4.0 AND google_reviews_count >= 10 → +20 points
   - credit_score.is_some() AND credit_score > 650 → +20 points; credit_available == false → +10 points (benefit of doubt)
   - risk_flags.is_empty() → +10 points
5. Classification: total >= 70 → GREEN, 40..=69 → YELLOW, < 40 → RED.
6. Return a breakdown vector of strings explaining each component's contribution for transparency.
7. Edge cases: handle all stages unavailable (should score ~10 if no risk flags, resulting in RED). Handle partial availability gracefully.

## Validation
Parameterized unit tests covering: (1) All stages pass with excellent data → expect 100 points, GREEN. (2) All stages fail/unavailable → expect 10 points (only no risk flags), RED. (3) Business verified + no LinkedIn + good reviews + no credit + no flags → 30+0+20+10+10=70, GREEN boundary. (4) Business verified + LinkedIn + bad reviews + no credit + flags → 30+20+0+10+0=60, YELLOW. (5) Credit unavailable gives 10 instead of 20. (6) Exactly 40 points → YELLOW boundary. (7) Exactly 39 points → RED boundary. Verify breakdown strings are populated.