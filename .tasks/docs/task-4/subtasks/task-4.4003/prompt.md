Implement subtask 4003: Implement invoice and payment CRUD endpoints

## Objective
Build Axum route handlers for creating, reading, updating, listing, and deleting invoices and payments, with proper validation, error handling, and JSON serialization.

## Steps
1. Create src/routes/invoices.rs with Axum handlers:
   - POST /v1/invoices: create invoice with line items, calculate subtotal/tax/total, validate required fields
   - GET /v1/invoices/:id: return invoice by ID with 404 if not found
   - GET /v1/invoices: list invoices with pagination (limit/offset), filtering by status, client_id, date range
   - PUT /v1/invoices/:id: update invoice (only if status is draft), recalculate totals
   - DELETE /v1/invoices/:id: soft delete or cancel invoice
   - POST /v1/invoices/:id/send: transition status from draft to sent, record issued_date
2. Create src/routes/payments.rs with handlers:
   - POST /v1/payments: record a payment against an invoice (manual, non-Stripe)
   - GET /v1/payments/:id: return payment details
   - GET /v1/invoices/:id/payments: list payments for an invoice
3. Add request/response DTOs with serde Serialize/Deserialize.
4. Implement input validation (required fields, valid enums, positive amounts).
5. Map repository errors to appropriate HTTP status codes (400, 404, 409, 500).
6. Wire routes into the main Axum router with shared AppState.

## Validation
POST /v1/invoices creates an invoice and returns 201 with calculated totals; GET returns the invoice; list endpoint supports pagination and filtering; updating a sent invoice returns 409; payment recording links to invoice correctly; invalid inputs return 400 with descriptive error messages; all endpoints return proper JSON.