Implement subtask 5006: Implement credit signals integration module

## Objective
Build a standalone Rust module that integrates with a credit/financial data API to retrieve credit signals for a business, including credit score, payment history indicators, and financial risk level.

## Steps
1. Create src/integrations/credit.rs.
2. Define a trait `CreditChecker` with async method `check_credit(org_id: &str, company_name: &str, registration_number: Option<&str>) -> Result<CreditSignals, VettingError>`.
3. CreditSignals struct: found (bool), credit_score (Option<f64>), credit_limit (Option<f64>), payment_performance (Option<String>), risk_level (CreditRiskLevel enum: Low/Medium/High/Unknown), days_beyond_terms (Option<i32>), ccjs_count (Option<i32>), credit_signal_score (f64 0.0-1.0).
4. Implement `CreditApiClient` struct with reqwest::Client and API credentials.
5. Make API call to the configured credit data provider, parse the response.
6. Normalize credit_signal_score: map credit score ranges to 0-1, factor in payment performance and CCJs.
7. Handle cases where credit data is unavailable (return Unknown risk level).
8. Implement `MockCreditChecker` for testing.

## Validation
Unit tests with mocked credit API responses verify correct credit signal parsing and score normalization; companies with good credit score high, those with CCJs score low; unavailable data returns Unknown risk level; mock implementation is functional.