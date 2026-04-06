Implement subtask 5005: Implement credit check API integration client

## Objective
Build an async HTTP client module for querying a credit/financial data API to retrieve credit scores and financial health indicators for organizations.

## Steps
1. Create `src/integrations/credit.rs` module.
2. Define request/response types for the credit check API (e.g., Dun & Bradstreet, Experian Business, or similar).
3. Use reqwest with appropriate authentication (API key or OAuth). Read CREDIT_API_KEY and CREDIT_API_URL from Kubernetes secrets/ConfigMap.
4. Implement `fetch_credit_report(org_id: &str, company_name: &str) -> Result<CreditReport, VettingError>` returning credit score, payment history indicators, risk rating, outstanding judgments.
5. Handle timeouts (credit APIs can be slow — 10s timeout), retries (1 retry on transient failure), and error mapping.
6. Add unit tests with mocked responses.

## Validation
Unit tests pass with mocked credit API responses covering: successful report, org not found, API timeout, authentication failure. CreditReport struct correctly populated with score and risk indicators.