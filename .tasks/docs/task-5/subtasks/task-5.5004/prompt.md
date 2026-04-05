Implement subtask 5004: Implement LinkedInClient with graceful degradation

## Objective
Build the LinkedIn company lookup client module with retry logic, 10-second timeouts, circuit breaker, and graceful degradation when the API is unavailable or unconfigured.

## Steps
1. Create `src/clients/linkedin.rs`.
2. Define `LinkedInClient` struct with: reqwest::Client (10s timeout), base_url, api_key (Option<String>), circuit breaker state.
3. Implement `pub async fn lookup_company(&self, org_name: &str, org_domain: Option<&str>) -> Result<LinkedInResult, VettingError>`:
   - If api_key is None, log warning and return `LinkedInResult { exists: false, followers: 0, degraded: true }`.
   - Check circuit breaker; if Open, return degraded result with warning log.
   - Make API call (exact endpoint depends on chosen provider — use a generic trait-based design).
   - Parse response for: company page exists, follower_count.
   - Retry with exponential backoff (3 attempts).
4. Define `LinkedInResult` struct: exists (bool), followers (i32), degraded (bool).
5. Implement the shared `CircuitBreaker` trait/struct (extract from opencorporates if not already shared into `src/clients/circuit_breaker.rs`).
6. Add unit tests: configured + successful → exists=true with followers, unconfigured → degraded result, API errors → circuit breaker trips → degraded results.

## Validation
Unit test: mock returns company with 500 followers → LinkedInResult { exists: true, followers: 500, degraded: false }. Unit test: no API key configured → returns degraded result without making network call. Unit test: 5 failures → circuit breaker opens → degraded result returned.