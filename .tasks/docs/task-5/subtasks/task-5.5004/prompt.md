Implement subtask 5004: Implement Google Reviews integration for reputation analysis

## Objective
Build the Google Reviews client module to fetch review data, average ratings, and review counts for an organization, and compute a reputation summary.

## Steps
1. Create a `sources/google_reviews.rs` module implementing the `VettingSource` trait. 2. Implement the Google Places API client (or scraping fallback, pending dp-14 decision). For the API path: use Google Places Text Search to find the business, then Place Details to get reviews. 3. Extract: average_rating, total_review_count, individual reviews (text, rating, date). 4. Compute a basic sentiment summary: count of 1-2 star reviews, 3 star reviews, 4-5 star reviews, and a simple sentiment_score. 5. Parse into a `ReputationAnalysis` struct with avg_rating, review_count, sentiment_score, sentiment_summary, recent_reviews (last 5). 6. Handle API key authentication, quota limits, and businesses with no reviews (return neutral score). 7. Write unit tests with mocked responses.

## Validation
Unit tests pass for: successful review retrieval, business with no reviews (neutral score returned), API quota exceeded handling, correct sentiment calculation from mock review data. ReputationAnalysis struct populated correctly.