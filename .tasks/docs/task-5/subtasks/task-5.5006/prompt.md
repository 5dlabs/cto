Implement subtask 5006: Implement credit API integration for credit signal retrieval

## Objective
Build the HTTP client integration for the selected credit scoring API to retrieve business credit signals and financial health indicators.

## Steps
1. In `integrations/credit_api.rs`, create a CreditApiClient struct with reqwest::Client and API credentials. 2. Implement methods: `get_credit_report(org_id: &str, org_name: &str) -> Result<CreditReport>`, `get_credit_score(org_id: &str) -> Result<CreditSignals>`. 3. Define DTOs for the credit API response format (design to be adaptable since the specific provider is a decision point). 4. Map into `CreditSignals` domain struct with fields: credit_score (Option<u32>), payment_history_rating, outstanding_judgments (u32), bankruptcy_flag (bool), years_in_business (Option<u32>), credit_risk_level (LOW/MEDIUM/HIGH). 5. Handle cases where credit data is unavailable (new businesses). 6. Use trait `CreditChecker` for mockability.

## Validation
Unit tests with mocked credit API responses verify correct parsing of credit signals; missing data scenarios (new business, no credit history) return sensible defaults; credit_risk_level mapping is validated against known inputs.