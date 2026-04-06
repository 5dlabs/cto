Implement subtask 5004: Implement Google Reviews API integration client

## Objective
Build an async HTTP client module for querying Google Places/Reviews API to retrieve business reviews, ratings, and reputation signals.

## Steps
1. Create `src/integrations/google_reviews.rs` module.
2. Define request/response types for Google Places API (Place Search, Place Details with reviews).
3. Use reqwest with API key authentication. Read GOOGLE_PLACES_API_KEY from Kubernetes secrets.
4. Implement `fetch_business_reviews(business_name: &str, location: Option<&str>) -> Result<GoogleReviewsData, VettingError>` returning average rating, review count, recent review summaries.
5. Handle pagination if needed, rate limiting, and error responses.
6. Parse and structure review data for downstream scoring.
7. Add unit tests with mocked responses.

## Validation
Unit tests pass with mocked Google Places responses covering: successful reviews fetch, no results found, API key invalid, rate limit exceeded. Parsed data correctly extracts average rating and review count.