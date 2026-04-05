Implement subtask 5002: Implement OpenCorporates API integration for business verification

## Objective
Build the OpenCorporates API client module to look up company registration data, verify incorporation status, and return structured business verification results.

## Steps
1. Create a `sources/opencorporates.rs` module. 2. Define an async trait `VettingSource` with a method `async fn fetch(&self, org_identifier: &str) -> Result<SourceResult, VettingError>` to allow uniform source handling. 3. Implement the OpenCorporates HTTP client using reqwest. 4. Handle authentication via API key from environment/secrets. 5. Parse the OpenCorporates JSON response into a `BusinessVerification` struct containing: company_name, jurisdiction, incorporation_date, company_status (active/inactive/dissolved), registered_address, officers list. 6. Implement error handling for rate limits (HTTP 429), not-found responses, and network failures using typed errors. 7. Add retry logic with exponential backoff for transient failures. 8. Write unit tests with mock HTTP responses (use wiremock or similar).

## Validation
Unit tests pass with mocked OpenCorporates responses for: successful company lookup, company not found (404), rate limit (429 with retry), malformed response handling. The BusinessVerification struct is correctly populated from mock data.