Implement subtask 5002: Implement OpenCorporates API integration module

## Objective
Build a dedicated Rust module for querying the OpenCorporates API to retrieve business registration, incorporation status, officers, and filing data for a given organization.

## Steps
1. Create `src/integrations/opencorporates.rs` module.
2. Define request/response types for OpenCorporates API endpoints (company search, company details, officers, filings).
3. Implement an `OpenCorporatesClient` struct with an `reqwest::Client` and API key.
4. Implement methods: `search_company(name: &str, jurisdiction: Option<&str>) -> Result<Vec<CompanyMatch>>`, `get_company(company_number: &str, jurisdiction: &str) -> Result<CompanyDetails>`, `get_officers(company_number: &str, jurisdiction: &str) -> Result<Vec<Officer>>`.
5. Handle API rate limits with exponential backoff using tokio::time::sleep.
6. Map API responses into internal domain types (BusinessVerificationData struct).
7. Handle error cases: company not found, API unavailable, rate limited, malformed responses.
8. Add structured logging with tracing for all API calls.

## Validation
Write unit tests with mock HTTP responses (using wiremock or similar) verifying correct parsing of OpenCorporates responses, error handling for 404/429/500 status codes, and correct mapping to domain types.