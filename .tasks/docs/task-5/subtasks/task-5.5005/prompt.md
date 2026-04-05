Implement subtask 5005: Implement credit API integration module

## Objective
Build a Rust module for fetching business credit signals (credit score, payment history, risk indicators) from the selected credit data provider.

## Steps
1. Create `src/integrations/credit.rs` module.
2. Define a `CreditProvider` trait with async methods: `get_credit_report(org_identifier: &str) -> Result<CreditReport>`, `get_credit_score(org_identifier: &str) -> Result<CreditScore>`.
3. Implement the trait for the chosen credit API provider (pending dp-5-1 resolution). Design the trait so it can be re-implemented for a different provider.
4. Define domain types: CreditReport { score: Option<u32>, risk_level: RiskLevel, payment_history_rating: Option<PaymentRating>, outstanding_judgments: u32, liens: u32, report_date: DateTime }, CreditScore { score: u32, range_min: u32, range_max: u32 }.
5. Implement HTTP calls with appropriate authentication (API key or OAuth depending on provider).
6. Handle provider-specific error codes and map to internal error types.
7. Add tracing and logging for audit trail of credit checks.
8. Consider implementing a stub/mock provider for development and testing.

## Validation
Write unit tests using a mock credit provider implementation. Verify correct deserialization, error handling for unavailable reports, and correct mapping to domain types. Verify the stub provider returns sensible test data.