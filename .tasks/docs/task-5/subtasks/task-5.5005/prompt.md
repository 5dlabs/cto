Implement subtask 5005: Implement GooglePlacesClient with retry and circuit breaker

## Objective
Build the Google Places API client module to search for a business and extract reviews rating and count, with retry logic, 10-second timeouts, and circuit breaker.

## Steps
1. Create `src/clients/google_places.rs`.
2. Define `GooglePlacesClient` struct with: reqwest::Client (10s timeout), api_key (String), circuit breaker state.
3. Implement `pub async fn search_business(&self, org_name: &str, org_domain: Option<&str>) -> Result<GooglePlacesResult, VettingError>`:
   - Check circuit breaker.
   - Call Google Places Text Search endpoint: `https://maps.googleapis.com/maps/api/place/textsearch/json?query={org_name}&key={api_key}`.
   - Parse first result's place_id.
   - Call Place Details endpoint for rating and user_ratings_total.
   - Retry with exponential backoff (3 attempts).
   - Return `GooglePlacesResult { rating: Option<f32>, review_count: i32 }`.
4. Handle: no results found (rating=None, count=0), API errors, invalid responses.
5. Define response structs matching Google Places API JSON shape.
6. Unit tests with mocked endpoints: successful search with 4.5 rating and 25 reviews, no results found, API error triggers circuit breaker.

## Validation
Unit test: mock returns place with rating 4.5, 25 reviews → GooglePlacesResult { rating: Some(4.5), review_count: 25 }. Unit test: mock returns empty results → GooglePlacesResult { rating: None, review_count: 0 }. Unit test: circuit breaker trips after 5 failures.