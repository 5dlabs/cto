Implement subtask 5004: Implement Google Reviews reputation scoring integration

## Objective
Build the integration module for Google Places/Reviews API to assess a company's reputation based on review ratings, volume, and recency.

## Steps
1. Create a `vetting::integrations::google_reviews` module. 2. Define a trait `ReputationProvider` with method: `assess_reputation(company_name: &str, location: Option<&str>) -> Result<ReputationResult>`. 3. Implement Google Places API integration: first use Place Search to find the business, then use Place Details to fetch reviews. Use the Google API key from Kubernetes secrets. 4. Extract: average_rating, total_reviews, recent_reviews (last 6 months), review_sentiment_summary. 5. Compute a reputation_score (0.0-1.0) based on: average_rating normalized to 0-1 (weight 0.4), log(review_count) normalized (weight 0.3), recency of reviews (weight 0.3). 6. Define ReputationResult struct with score, average_rating, total_reviews, recent_review_count, most_recent_review_date. 7. Handle cases where business is not found (return neutral score with flag).

## Validation
Unit tests with mocked Google API responses: business with many positive reviews, business with few/no reviews, business not found. Verify scoring formula produces expected values for boundary conditions (0 reviews, 5.0 rating, 1.0 rating).