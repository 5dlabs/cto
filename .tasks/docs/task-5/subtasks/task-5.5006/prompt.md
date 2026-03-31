Implement subtask 5006: Implement credit signals pipeline stage

## Objective
Build the HTTP client integration with a credit scoring/bureau API to retrieve credit signals as the fourth vetting pipeline stage.

## Steps
1. Create a `clients/credit.rs` module.
2. Define a `CreditClient` struct wrapping reqwest::Client and API credentials.
3. Implement `get_credit_signals(&self, company_name: &str, registration_number: Option<&str>) -> Result<CreditSignals>` that:
   - Calls the credit bureau API endpoint.
   - Extracts: credit_score (Option<u32>), payment_history_rating (String), outstanding_judgments (bool), bankruptcy_flag (bool), credit_limit_suggestion (Option<f64>).
   - Populates the `CreditSignals` struct.
4. Implement the `VettingStage` trait for `CreditStage`.
5. Handle graceful degradation: if the credit API returns no data, return `CreditSignals` with `available: false` and document the absence.
6. Include mock implementation for testing.

## Validation
Unit tests with mocked HTTP responses verify correct parsing of credit bureau data. Unavailable-data case returns a structured result with `available: false`. The VettingStage trait contract is satisfied.