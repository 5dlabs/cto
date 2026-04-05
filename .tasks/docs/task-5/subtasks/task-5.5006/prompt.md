Implement subtask 5006: Implement Stage 3: Reputation Check via Google Reviews/Places API

## Objective
Build the reputation stage that queries Google Business Profile or Google Places API to extract review rating and review count for the company.

## Steps
1. Create `src/stages/reputation.rs` module.
2. Define `ReputationResult` struct: google_reviews_rating (Option<f64>), google_reviews_count (i32), raw_responses (serde_json::Value).
3. Implement Google Places API integration: call `https://maps.googleapis.com/maps/api/place/findplacefromtext/json?input={company_name}&inputtype=textquery&fields=place_id,name,rating,user_ratings_total&key={api_key}`.
4. Parse response: extract `candidates[0].rating` as google_reviews_rating and `candidates[0].user_ratings_total` as google_reviews_count.
5. API key from environment variable `GOOGLE_PLACES_API_KEY` (sourced from `sigma1-external-apis` secret).
6. Use `ResilientClient` with `google_places` circuit breaker.
7. On circuit open or failure, return rating=None, count=0 with Unavailable note.
8. Capture raw API response JSON for audit storage.

## Validation
Unit tests with wiremock-rs: mock Google Places findplacefromtext endpoint returning a valid candidate with rating 4.5 and 150 reviews. Verify parsed ReputationResult. Test with no candidates found (empty results). Test with missing rating field. Verify raw response captured. Test circuit breaker integration.