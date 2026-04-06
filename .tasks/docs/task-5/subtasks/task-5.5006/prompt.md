Implement subtask 5006: Implement GREEN/YELLOW/RED scoring algorithm

## Objective
Build the scoring algorithm that combines signals from all four external API sources (OpenCorporates, LinkedIn, Google Reviews, credit) into a composite LeadScore with a GREEN/YELLOW/RED rating.

## Steps
1. Create `src/scoring.rs` module.
2. Define scoring dimensions: financial_score (from credit data), reputation_score (from Google Reviews + LinkedIn), legal_score (from OpenCorporates company status and filings).
3. Implement per-dimension scoring functions:
   - `score_financial(credit: &CreditReport) -> f64` — map credit score ranges to 0.0–1.0.
   - `score_reputation(reviews: &GoogleReviewsData, linkedin: &LinkedInProfile) -> f64` — weighted combination of average rating, review count, employee count, company age.
   - `score_legal(corp: &OpenCorporatesResponse) -> f64` — penalize inactive/dissolved status, recent adverse filings.
4. Implement composite scoring: weighted average of dimensions (configurable weights via env vars or defaults: financial=0.4, reputation=0.3, legal=0.3).
5. Map composite score to rating: GREEN >= 0.7, YELLOW >= 0.4, RED < 0.4.
6. Handle missing data gracefully — if an API returned no data, skip that dimension and re-weight.
7. Return `LeadScore` struct with all dimension scores and composite rating.
8. Add comprehensive unit tests covering edge cases: all green, all red, mixed signals, missing dimensions.

## Validation
Unit tests validate: a company with strong financials/reviews/legal status scores GREEN; a company with poor credit and no reviews scores RED; missing data for one dimension still produces a valid score; boundary cases at 0.4 and 0.7 thresholds are correct.