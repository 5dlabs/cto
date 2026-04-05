Implement subtask 5005: Implement credit scoring API integration

## Objective
Build the credit data API client module to retrieve business credit scores and risk assessments for organizations.

## Steps
1. Create a `sources/credit.rs` module implementing the `VettingSource` trait. 2. Implement the credit API client (provider pending dp-12 decision). Design behind a `CreditProvider` trait so the provider can be swapped. 3. For v1, implement against the chosen provider's REST API: authenticate, submit company identifier (DUNS number, company registration number, etc.), retrieve credit report. 4. Parse into a `CreditScore` struct: provider_name, credit_score (numeric), risk_level (low/medium/high), payment_history_summary, credit_limit_recommendation, report_date. 5. Handle: company not found in credit database, expired/stale reports, API authentication failures. 6. Implement the /api/v1/vetting/credit/:org_id endpoint that returns just the credit portion independently. 7. Write unit tests with mocked credit API responses.

## Validation
Unit tests pass for: successful credit report retrieval, company not found in credit DB, authentication failure handling. CreditScore struct correctly populated. The /api/v1/vetting/credit/:org_id endpoint returns credit data for a mocked org.