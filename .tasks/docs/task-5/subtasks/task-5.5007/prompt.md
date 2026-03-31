Implement subtask 5007: Implement composite scoring engine (GREEN/YELLOW/RED)

## Objective
Build the composite scoring engine that aggregates results from all four pipeline stages into a final GREEN/YELLOW/RED lead score with detailed breakdown.

## Steps
1. Create a `scoring.rs` module.
2. Define a `ScoringEngine` struct that takes stage results and produces a `LeadScore`.
3. Implement scoring logic:
   - Assign weighted scores to each stage: business_verification (30%), online_presence (20%), reputation (25%), credit_signals (25%).
   - Each stage produces a normalized score 0.0-1.0.
   - Composite score = weighted sum of stage scores.
   - Thresholds: >= 0.7 → GREEN, >= 0.4 → YELLOW, < 0.4 → RED.
   - Hard fail rules: if business not verified OR bankruptcy flag → automatic RED regardless of composite.
4. Return `LeadScore { grade: ScoreGrade, composite_score: f64, stage_scores: HashMap<String, f64>, flags: Vec<String>, recommendation: String }`.
5. The recommendation field provides a human-readable summary (e.g., "Strong business profile with good credit history").
6. Make weights and thresholds configurable via `AppConfig` for future tuning.

## Validation
Unit tests cover: all-positive inputs yield GREEN, mixed inputs yield YELLOW, poor inputs yield RED. Hard fail rules are tested (unverified business → RED, bankruptcy → RED). Edge cases at threshold boundaries are tested. Weights sum to 1.0.