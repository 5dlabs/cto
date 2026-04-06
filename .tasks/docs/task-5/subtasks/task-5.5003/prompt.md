Implement subtask 5003: Implement OpenCorporates business verification integration

## Objective
Build the HTTP client integration for the OpenCorporates API to perform business entity verification, including company search, registration status, and officer lookups.

## Steps
1. In `integrations/opencorporates.rs`, create an OpenCorporatesClient struct holding a reqwest::Client and API key. 2. Implement methods: `search_company(name: &str, jurisdiction: Option<&str>) -> Result<CompanySearchResult>`, `get_company(company_number: &str, jurisdiction: &str) -> Result<CompanyDetails>`, `verify_registration(org_name: &str) -> Result<BusinessVerification>`. 3. Define response DTOs matching OpenCorporates API JSON structure. 4. Map API responses into a `BusinessVerification` domain struct with fields: is_registered (bool), registration_date, status, jurisdiction, officers, confidence_score. 5. Handle API errors, rate limits (429), and timeouts gracefully with proper error types. 6. Use trait `BusinessVerifier` so the client can be mocked in tests.

## Validation
Unit tests with mocked HTTP responses verify correct parsing of OpenCorporates data; error cases (404, 429, timeout) return appropriate error variants; BusinessVerification struct is correctly populated from mock data.