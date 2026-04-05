Implement subtask 5004: Implement Stage 1: Business Verification via OpenCorporates API

## Objective
Build the OpenCorporates integration stage that searches for a company by name, verifies existence and incorporation status, extracts directors, and returns structured results with raw response capture.

## Steps
1. Create `src/stages/business_verification.rs` module.
2. Define `BusinessVerificationResult` struct: company_found (bool), incorporation_status (Option<String>), directors (Vec<String>), opencorporates_data (serde_json::Value).
3. Implement `run_business_verification(client: &ResilientClient, company_name: &str, api_key: &str) -> StageResult<BusinessVerificationResult>`.
4. Call OpenCorporates API: `GET https://api.opencorporates.com/v0.4/companies/search?q={company_name}&api_token={api_key}`.
5. Parse response JSON: extract `results.companies[0].company` — check `inactive` field for incorporation status, extract `officers` endpoint for directors if available.
6. API key sourced from `sigma1-external-apis` Kubernetes secret, passed via environment variable `OPENCORPORATES_API_KEY`.
7. Use `ResilientClient.execute_with_retry` with the `opencorporates` circuit breaker.
8. On circuit open or all retries exhausted, return `StageResult::Unavailable` with reason string.
9. Return raw API response JSON alongside parsed result for audit storage.

## Validation
Unit tests with wiremock-rs: mock OpenCorporates search endpoint returning a valid company response, verify BusinessVerificationResult has company_found=true, correct incorporation status, and extracted directors. Test with empty results (company not found). Test with malformed JSON response (graceful error handling). Verify raw_response is captured.