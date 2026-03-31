Implement subtask 5005: Implement Google Reviews reputation analysis pipeline stage

## Objective
Build the HTTP client integration with the Google Places/Reviews API to analyze business reputation as the third vetting pipeline stage.

## Steps
1. Create a `clients/google_reviews.rs` module.
2. Define a `GoogleReviewsClient` struct wrapping reqwest::Client and API key.
3. Implement `analyze_reputation(&self, company_name: &str, location: Option<&str>) -> Result<ReputationAnalysis>` that:
   - Calls Google Places API to find the business.
   - Retrieves reviews and ratings.
   - Extracts: average_rating (f64), review_count (u32), sentiment_summary (positive/mixed/negative), notable flags (e.g., recent negative trend).
   - Populates the `ReputationAnalysis` struct.
4. Implement the `VettingStage` trait for `GoogleReviewsStage`.
5. Handle cases where the business is not found on Google (return `reputation_found: false` with neutral score).
6. Respect Google API rate limits and handle quota errors.
7. Include mock implementation for testing.

## Validation
Unit tests with mocked HTTP responses verify correct parsing of Google Places/Reviews data. Business-not-found case returns a neutral result. Rate limit errors are handled gracefully. The VettingStage trait contract is satisfied.