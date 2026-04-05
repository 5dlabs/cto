Implement subtask 5004: Implement Google Reviews API integration module

## Objective
Build a Rust module for fetching Google Reviews data (ratings, review count, sentiment) for a business using the Google Places API.

## Steps
1. Create `src/integrations/google_reviews.rs` module.
2. Define types for Google Places API responses: PlaceSearchResult, PlaceDetails, Review.
3. Implement a `GoogleReviewsClient` struct with reqwest::Client and API key.
4. Implement methods: `find_place(business_name: &str, location: Option<&str>) -> Result<Option<PlaceId>>`, `get_place_details(place_id: &PlaceId) -> Result<PlaceDetails>`, `get_reviews_summary(place_id: &PlaceId) -> Result<ReputationData>`.
5. Map API responses to internal ReputationData struct: { average_rating: f64, total_reviews: u32, recent_reviews: Vec<ReviewSummary>, sentiment_indicator: SentimentLevel }.
6. Handle Google API error codes, quota limits, and places not found.
7. Add tracing spans for each API call.

## Validation
Write unit tests with mocked Google Places API responses. Verify correct parsing of place search results, details extraction, and review aggregation. Test error paths for no results, quota exceeded, and malformed responses.