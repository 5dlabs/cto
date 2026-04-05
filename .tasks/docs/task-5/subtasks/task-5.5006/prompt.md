Implement subtask 5006: Implement CreditClient with feature flag gating

## Objective
Build the commercial credit API client module that is feature-flagged and returns None when not configured, with retry and circuit breaker when active.

## Steps
1. Create `src/clients/credit.rs`.
2. Define `CreditClient` struct with: reqwest::Client (10s timeout), base_url (Option<String>), api_key (Option<String>), enabled (bool from config/env var `CREDIT_API_ENABLED`), circuit breaker state.
3. Implement `pub async fn get_credit_score(&self, org_name: &str, org_domain: Option<&str>) -> Result<Option<i32>, VettingError>`:
   - If !enabled or api_key.is_none(), log `tracing::warn!("Credit API not configured, skipping")` and return Ok(None).
   - Check circuit breaker.
   - Make API call to credit provider endpoint.
   - Parse credit score integer from response.
   - Retry with exponential backoff (3 attempts).
   - Return Ok(Some(score)) on success.
4. Define `CreditResult` struct for internal use.
5. Ensure the feature flag is read from environment variable at startup and stored in AppState config.
6. Unit tests: disabled flag → returns None without network call, enabled + success → returns Some(score), enabled + failures → circuit breaker trips.

## Validation
Unit test: CREDIT_API_ENABLED=false → returns Ok(None) with no HTTP calls. Unit test: enabled + mock returns score 720 → returns Ok(Some(720)). Unit test: enabled + 5 failures → circuit breaker opens → returns error.