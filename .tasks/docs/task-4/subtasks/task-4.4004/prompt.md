Implement subtask 4004: Implement invoice and payment CRUD endpoints

## Objective
Build Axum route handlers for invoice creation, retrieval, listing, status updates, and payment recording endpoints with proper validation and error handling.

## Steps
1. In src/routes/invoices.rs: POST /api/v1/invoices (create invoice with line items, validate currency code, calculate totals), GET /api/v1/invoices/:id (return invoice with line items), GET /api/v1/invoices (list with query params: status, page, per_page), PATCH /api/v1/invoices/:id (update status, e.g., send, void), DELETE /api/v1/invoices/:id (soft-delete/void only if draft). 2. In src/routes/payments.rs: POST /api/v1/payments (record a payment against an invoice, validate amount doesn't exceed remaining balance), GET /api/v1/invoices/:id/payments (list payments for an invoice). 3. Add input validation using a validator crate or custom extractors. 4. Implement proper error responses: 400 for validation errors, 404 for not found, 409 for invalid state transitions. 5. When a payment is recorded and total payments >= invoice total, automatically update invoice status to 'paid'. 6. Register all routes on the Axum router in main.rs.

## Validation
Integration tests: create invoice → verify GET returns it → record partial payment → verify invoice still 'sent' → record remaining payment → verify invoice auto-transitions to 'paid'; attempt to void a paid invoice returns 409; validation errors return 400 with descriptive messages; >80% coverage.