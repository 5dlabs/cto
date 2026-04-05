Implement subtask 5005: Implement Google Reviews reputation integration module

## Objective
Build a standalone Rust module that integrates with a Google Reviews data source (Google Places API or commercial alternative) to assess a company's reputation based on review count, average rating, and sentiment signals.

## Steps
1. Create src/integrations/google_reviews.rs.
2. Define a trait `ReputationChecker` with async method `check_reputation(company_name: &str, location: Option<&str>) -> Result<Reputation, VettingError>`.
3. Reputation struct: found (bool), place_id (Option<String>), average_rating (Option<f64>), total_reviews (Option<i32>), recent_review_count (Option<i32>), reputation_score (f64 0.0-1.0).
4. Implement `GoogleReviewsClient` struct with reqwest::Client and API key.
5. Use Google Places API: Text Search to find the business, then Place Details for reviews.
6. Calculate reputation_score: weighted combination of average_rating normalized to 0-1 (weight 0.5), log-scaled review count (weight 0.3), and recency of reviews (weight 0.2).
7. Handle API quota limits, no-results, and parsing errors.
8. Implement `MockReputationChecker` for testing.

## Validation
Unit tests with mocked Google Places API responses verify correct reputation scoring; businesses with high ratings and many reviews score higher; missing businesses return found=false with score 0.0; mock implementation works as expected.