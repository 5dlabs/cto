Implement subtask 5005: Implement Google Reviews reputation analysis integration

## Objective
Build the HTTP client integration for Google Reviews/Places API to analyze business reputation based on review data.

## Steps
1. In `integrations/google_reviews.rs`, create a GoogleReviewsClient struct with reqwest::Client and API key. 2. Implement methods: `search_place(business_name: &str, location: Option<&str>) -> Result<PlaceSearchResult>`, `get_reviews(place_id: &str) -> Result<Vec<Review>>`, `analyze_reputation(org_name: &str) -> Result<ReputationAnalysis>`. 3. Define DTOs matching Google Places API JSON. 4. Map into `ReputationAnalysis` domain struct with fields: average_rating (f64), total_reviews (u32), recent_review_sentiment (POSITIVE/NEUTRAL/NEGATIVE), review_trend (IMPROVING/STABLE/DECLINING), reputation_score (0-100). 5. Implement simple sentiment trend analysis from review timestamps and ratings. 6. Handle API errors and missing data gracefully. 7. Use trait `ReputationAnalyzer` for mockability.

## Validation
Unit tests with mocked Google Places responses verify correct reputation scoring; edge cases (no reviews, single review, many reviews) produce expected scores; sentiment trend calculation is verified with known datasets.