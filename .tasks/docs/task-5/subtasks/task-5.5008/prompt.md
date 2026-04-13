<identity>
You are rex working on subtask 5008 of task 5.
</identity>

<context>
<scope>
Write the full cargo test suite covering the scoring algorithm unit tests, async pipeline integration tests with mocked HTTP, cache hit/miss behavior, GDPR deletion flow, and async 202 polling convergence.
</scope>
</context>

<implementation_plan>
Use wiremock-rs or httpmock for HTTP mocking. Test cases required: (1) Scoring unit tests as defined in test_strategy — all three score bands. (2) POST /api/v1/vetting/run returns 202 within 1s. (3) Poll GET /api/v1/vetting/:org_id every 2s up to 30s until status=completed; assert final_score is GREEN/YELLOW/RED and vetted_at is set. (4) OpenCorporates 404 mock → risk_flags contains 'business_not_found'. (5) Cache hit: second POST for same org_id returns immediately with cached=true and no new vetting_request row. (6) GDPR delete → subsequent GET returns 404. (7) All four pipeline steps timing out → final_score=YELLOW, risk_flags contains all four timeout flags. Use tokio::test for async tests. Measure coverage with cargo-llvm-cov; fail CI if < 80%.
</implementation_plan>

<validation>
`cargo test` exits 0 with all tests passing. `cargo llvm-cov --summary-only` reports >= 80% line coverage. CI pipeline runs `cargo test` and coverage check as a required gate.
</validation>