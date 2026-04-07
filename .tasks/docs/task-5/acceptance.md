## Acceptance Criteria

- [ ] 1. POST /api/v1/vetting/run returns 202 with request_id within 1 second (pipeline runs async). 2. After async pipeline completes (poll GET /api/v1/vetting/:org_id up to 30s), response contains final_score of GREEN, YELLOW, or RED and vetted_at timestamp. 3. Scoring unit test: business_verified=true, linkedin_followers=500, google_reviews_rating=4.2, google_reviews_count=20, credit_score=700 should produce GREEN. 4. If OpenCorporates returns 404 for org, risk_flags contains business_not_found and final_score is RED or YELLOW. 5. Second call to POST /api/v1/vetting/run for same org_id within 24h returns cached result (verify by checking Valkey key exists via redis-cli GET vetting:org:{id}). 6. DELETE /internal/gdpr/delete/:org_id returns 200; subsequent GET /api/v1/vetting/:org_id returns 404. 7. GET /health/ready returns 200. 8. cargo test passes with >= 80% coverage.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.