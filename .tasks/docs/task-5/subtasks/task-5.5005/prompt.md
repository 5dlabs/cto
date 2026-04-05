Implement subtask 5005: Implement credit signal API integration

## Objective
Build the integration module for credit signal checking, implementing the provider trait against the selected credit API to retrieve commercial credit scores and payment history.

## Steps
1. Create a `vetting::integrations::credit` module. 2. Define a trait `CreditProvider` with methods: `check_credit(org_name: &str, registration_number: Option<&str>) -> Result<CreditResult>`, `get_payment_history(org_id: &str) -> Result<PaymentHistory>`. 3. Implement the trait for the chosen credit provider (or a stub if dp-16 is unresolved). Load API credentials from Kubernetes secrets. 4. Define CreditResult struct: credit_score (Option<i32>), credit_limit (Option<f64>), payment_performance (enum: Excellent/Good/Fair/Poor/Unknown), years_in_business, has_ccjs_or_defaults (bool). 5. Compute a normalized credit_signal_score (0.0-1.0) from the raw data: credit_score normalized (weight 0.5), payment_performance mapped to 0-1 (weight 0.3), no defaults bonus (weight 0.2). 6. Implement a mock/stub provider that returns configurable test data for development and testing. 7. Handle credit API errors gracefully: if credit check fails, return CreditResult with Unknown status and 0.5 neutral score.

## Validation
Unit tests using the mock provider for: excellent credit, poor credit, unknown/unavailable credit data, API failure fallback. Verify normalized score calculations match expected outputs.