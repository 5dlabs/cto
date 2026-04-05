Implement subtask 5003: Implement OpenCorporatesClient with retry and circuit breaker

## Objective
Build the OpenCorporates external API client module with reqwest, exponential backoff retry via tokio-retry, 10-second timeouts, and a circuit breaker that trips after 5 consecutive failures.

## Steps
1. Create `src/clients/opencorporates.rs`.
2. Define `OpenCorporatesClient` struct with: reqwest::Client (configured with 10s timeout), base_url (String), api_key (Option<String>), and a circuit breaker state (Arc<Mutex<CircuitBreakerState>>).
3. Implement `CircuitBreakerState` (or use a crate): track consecutive_failures, state (Closed/Open/HalfOpen), last_failure_time. Open after 5 failures, half-open after 30s cooldown.
4. Implement `pub async fn search_company(&self, org_name: &str) -> Result<Option<OpenCorporatesResult>, VettingError>`:
   - Check circuit breaker state; if Open and not past cooldown, return Err immediately.
   - Build GET request to `{base_url}/companies/search?q={org_name}&api_token={api_key}`.
   - Wrap in tokio-retry with ExponentialBackoff (3 attempts, starting 500ms).
   - Parse JSON response into `OpenCorporatesSearchResponse` struct.
   - Extract: company_name, jurisdiction_code, company_status (good_standing), officers.
   - On success, reset circuit breaker. On failure, increment.
5. Define response structs: `OpenCorporatesSearchResponse`, `OpenCorporatesCompany`, `OpenCorporatesResult` (domain model).
6. Constructor `new(base_url, api_key)` with defaults.
7. Add `#[cfg(test)]` module with unit tests using mockito or wiremock for: successful search, 404 not found, timeout, circuit breaker tripping after 5 failures.

## Validation
Unit test: mock returns valid company JSON → parsed correctly with business_verified=true. Unit test: mock returns empty results → returns None. Unit test: mock returns 500 five times → circuit breaker opens, 6th call returns error without network request. Unit test: 10s timeout is configured on the reqwest client.