Implement subtask 4013: Generate OpenAPI spec and configure utoipa documentation

## Objective
Configure utoipa to generate and serve the complete OpenAPI specification for the Finance Service at `/api/v1/finance/openapi.json`, covering all endpoints, request/response models, and error types.

## Steps
1. Create `services/rust/finance/src/openapi.rs`.
2. Define the `#[derive(OpenApi)]` struct with all endpoint paths:
   - `/api/v1/invoices` (POST, GET)
   - `/api/v1/invoices/{id}` (GET)
   - `/api/v1/invoices/{id}/send` (POST)
   - `/api/v1/invoices/{id}/paid` (POST)
   - `/api/v1/invoices/overdue` (GET)
   - `/api/v1/payments` (POST, GET)
   - `/api/v1/payments/invoice/{invoice_id}` (GET)
   - `/api/v1/finance/reports/revenue` (GET)
   - `/api/v1/finance/reports/aging` (GET)
   - `/api/v1/finance/reports/cashflow` (GET)
   - `/api/v1/finance/reports/profitability` (GET)
   - `/api/v1/payroll` (POST, GET)
   - `/api/v1/currency/rates` (GET)
   - `/api/v1/webhooks/stripe` (POST)
   - `/api/v1/gdpr/customer/{id}` (DELETE)
3. Register all request/response schemas in the `components(schemas(...))` section.
4. Add `GET /api/v1/finance/openapi.json` route that serves the generated JSON.
5. Include API info metadata: title "Sigma1 Finance Service", version, description.
6. Add security scheme for API key authentication.

## Validation
Verify GET /api/v1/finance/openapi.json returns valid JSON with 200 status. Validate the returned JSON against OpenAPI 3.0 spec (use a validator). Verify all 15+ endpoint paths are present. Verify all request/response schemas are defined. Verify security scheme is documented.