Implement subtask 5005: Implement Google Reviews pipeline step and weighted scoring algorithm

## Objective
Implement the third pipeline step (Google Places API for reviews) and the weighted scoring function that aggregates all four step results into a GREEN/YELLOW/RED final_score. The scoring logic must be unit-testable independently of HTTP calls.

## Steps
Step 3 — Google Reviews: GET https://maps.googleapis.com/maps/api/place/findplacefromtext/json?input={company_name}&inputtype=textquery&key={GOOGLE_PLACES_API_KEY}. Extract place_id from response. Then GET https://maps.googleapis.com/maps/api/place/details/json?place_id={id}&fields=rating,user_ratings_total&key={key}. Parse rating (FLOAT4) and user_ratings_total (INT). Wrap in 10s timeout. On failure push 'google_reviews_unavailable'. Step 4 — Credit stub: generate a mock credit_score in range 600-750, push 'credit_stub' into risk_flags to signal Phase 2 replacement. Scoring function (pure, no I/O): fn compute_score(business_verified: bool, linkedin_followers: i32, google_rating: f32, google_count: i32, credit_score: i32) -> &'static str. Weights: business_verified → 0 or 40 points; linkedin_score = min(linkedin_followers/1000.0, 1.0)*20; reviews_score = (google_rating/5.0)*min(google_count as f32/50.0, 1.0)*20; credit_component = ((credit_score-300) as f32/550.0).clamp(0.0,1.0)*20. Sum → >=70 GREEN, 40-69 YELLOW, <40 RED.

## Validation
Unit test compute_score: (true, 500, 4.2, 20, 700) → GREEN. (false, 0, 0.0, 0, 0) → RED. (false, 200, 3.0, 10, 600) → YELLOW. Integration test mocking Places API: valid response → google_reviews_rating and google_reviews_count populated in vetting_results. Timeout mock → 'google_reviews_unavailable' in risk_flags.