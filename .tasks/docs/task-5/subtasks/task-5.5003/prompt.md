Implement subtask 5003: Implement OpenCorporates business verification integration module

## Objective
Build a standalone Rust module that integrates with the OpenCorporates API to verify business registration, retrieve incorporation details, officer information, and filing status. Implement proper error handling, response parsing, and a trait-based interface for testability.

## Steps
1. Create src/integrations/mod.rs and src/integrations/opencorporates.rs.
2. Define a trait `BusinessVerifier` with async method `verify_business(company_name: &str, jurisdiction: Option<&str>) -> Result<BusinessVerification, VettingError>`.
3. BusinessVerification struct: company_number, company_name, jurisdiction_code, incorporation_date, company_type, current_status, registered_address, officers (Vec), inactive (bool).
4. Implement `OpenCorporatesClient` struct holding reqwest::Client and API key.
5. Implement the API call to OpenCorporates search endpoint (GET /companies/search) and company details endpoint.
6. Parse JSON responses using serde, handle rate limiting (429), not-found, and network errors gracefully.
7. Normalize the output into the BusinessVerification struct.
8. Implement a `MockBusinessVerifier` for testing.

## Validation
Unit tests with recorded/mocked API responses verify correct parsing of OpenCorporates data; error cases (404, 429, network timeout) are handled and return appropriate VettingError variants; mock implementation satisfies the trait contract.