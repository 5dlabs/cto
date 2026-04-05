Implement subtask 5002: Implement OpenCorporates business verification integration

## Objective
Build the HTTP client module for the OpenCorporates API to perform business entity verification, including company lookup, officer search, and filing status checks.

## Steps
1. Create a `vetting::integrations::opencorporates` module. 2. Define a trait `BusinessVerificationProvider` with async methods: `verify_company(name: &str, jurisdiction: &str) -> Result<BusinessVerification>`, `lookup_officers(company_number: &str) -> Result<Vec<Officer>>`. 3. Implement the trait for OpenCorporates using reqwest with the API key loaded from Kubernetes secrets. 4. Parse API responses into strongly typed structs: CompanyMatch (name, company_number, jurisdiction, status, incorporation_date), Officer (name, role, appointed_date). 5. Implement retry logic with exponential backoff (max 3 retries). 6. Return a BusinessVerificationResult with match_confidence (0.0-1.0), active_status (bool), incorporation_years, and officer_count. 7. Handle API rate limits (respect 429 responses with Retry-After header).

## Validation
Unit tests with mocked HTTP responses covering: successful company match, no match found, API rate limit handling, network timeout retry behavior. Integration test against OpenCorporates sandbox API if available.