Implement subtask 5002: Implement OpenCorporates API integration client

## Objective
Build an async HTTP client module for querying the OpenCorporates API to retrieve company registration data, officer information, and filings for a given organization.

## Steps
1. Create `src/integrations/opencorporates.rs` module.
2. Define request/response types matching the OpenCorporates REST API (company search, company details endpoints).
3. Use reqwest with async/await for HTTP calls.
4. Read API key from secrets (environment variable OPENCORPORATES_API_KEY sourced from Kubernetes secret).
5. Implement `fetch_company_details(company_name: &str, jurisdiction: Option<&str>) -> Result<OpenCorporatesResponse, VettingError>`.
6. Handle rate limiting (respect Retry-After headers), timeouts (5s default), and error mapping to domain VettingError enum.
7. Parse response into structured data: company status, incorporation date, officers, recent filings.
8. Add unit tests with mock HTTP responses using wiremock or similar.

## Validation
Unit tests pass with mocked OpenCorporates responses covering: successful lookup, company not found, rate-limited response, malformed response. Integration test against real API (gated behind feature flag) returns valid company data.