## Acceptance Criteria

- [ ] 1. Unit test: scoring algorithm produces GREEN (>= 70) for fully verified company with good reviews, YELLOW (40-69) for partially verified, RED (< 40) for unverified with no online presence — at least 5 test vectors. 2. Unit test: risk_flags correctly populated (e.g., 'business_not_verified', 'no_linkedin_presence', 'low_review_rating'). 3. Integration test: POST /api/v1/vetting/run with mock external APIs returning positive data → poll GET /api/v1/vetting/:org_id until completed → verify GREEN score and all fields populated. 4. Integration test: all external APIs return errors/timeouts → verify pipeline completes with RED score and appropriate risk_flags. 5. Cache test: second GET for same org_id returns cached result without external API calls (verify via mock call counts). 6. GDPR test: DELETE /api/v1/vetting/:org_id removes all records, subsequent GET returns 404. 7. Circuit breaker test: simulate 5 consecutive OpenCorporates failures, verify 6th call is short-circuited without network request.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.