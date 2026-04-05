Implement subtask 4002: Implement invoice CRUD endpoints with quote-to-invoice atomic conversion

## Objective
Build the invoice management endpoints including create, read, update, list with filtering, and the atomic quote-to-invoice conversion flow that marks the source opportunity and creates the invoice in a single database transaction.

## Steps
1. Create `src/models/invoice.rs`: Invoice, InvoiceLineItem, CreateInvoiceRequest, UpdateInvoiceRequest, InvoiceResponse, InvoiceListQuery (filters: status, customer_id, date range, pagination).
2. Create `src/repositories/invoice_repo.rs` with SQLx queries: create_invoice (within transaction: insert invoice + line items), get_invoice_by_id (join with line items), list_invoices (with dynamic filtering), update_invoice_status, create_from_opportunity.
3. Implement `create_from_opportunity` as atomic transaction: (a) fetch opportunity data from RMS service or a shared reference, (b) create invoice with line items copied from opportunity, (c) write audit_log entry recording the conversion. Use SQLx transaction API.
4. Create `src/services/invoice_service.rs` with business logic: validate status transitions (draft→sent→paid, draft→cancelled, sent→cancelled, sent→overdue), calculate totals from line items, enforce required fields.
5. Create `src/routes/invoices.rs` with Axum handlers:
   - POST /api/v1/invoices — create invoice
   - GET /api/v1/invoices/:id — get invoice
   - GET /api/v1/invoices — list invoices with query params
   - PATCH /api/v1/invoices/:id/status — update status
   - POST /api/v1/invoices/from-opportunity/:opportunity_id — convert quote to invoice
6. Return proper HTTP status codes: 201 for creation, 404 for not found, 422 for validation errors, 409 for invalid transitions.
7. Write audit log entries for all state changes.

## Validation
Integration tests: create invoice with line items, verify totals calculated correctly. Fetch by ID, verify line items included. List with filters, verify pagination. Test all status transitions (valid and invalid). Test quote-to-invoice conversion atomicity: verify invoice created and audit log written, or neither on failure. Verify 422 returned for invalid data.