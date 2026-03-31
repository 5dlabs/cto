Implement subtask 5003: Implement OpenCorporates business verification pipeline stage

## Objective
Build the HTTP client integration with the OpenCorporates API to verify business registration, legal status, and company details as the first stage of the vetting pipeline.

## Steps
1. Create a `clients/opencorporates.rs` module.
2. Define an `OpenCorporatesClient` struct wrapping a `reqwest::Client` and API key.
3. Implement `verify_business(&self, company_name: &str, jurisdiction: Option<&str>) -> Result<BusinessVerification>` that:
   - Calls the OpenCorporates company search endpoint.
   - Parses the response to extract: company number, jurisdiction, incorporation date, current status, registered address.
   - Maps API response to `BusinessVerification` struct with a `verified: bool` field and `confidence_score: f64`.
4. Define a `VettingStage` trait: `async fn execute(&self, input: &VettingInput) -> Result<StageOutput>` for pluggable pipeline stages.
5. Implement `VettingStage` for `OpenCorporatesStage`.
6. Handle API errors, timeouts (10s), and rate limiting gracefully with retries (max 2).
7. Provide a mock implementation behind a `#[cfg(test)]` flag or trait object for testing.

## Validation
Unit tests with mocked HTTP responses verify correct parsing of OpenCorporates API data. Error cases (API timeout, 404, rate limit) are handled without panics. The VettingStage trait contract is satisfied.