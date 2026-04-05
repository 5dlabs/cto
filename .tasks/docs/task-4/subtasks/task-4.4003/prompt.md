Implement subtask 4003: Implement invoice CRUD endpoints and business logic

## Objective
Build Axum REST endpoints for creating, reading, updating, listing invoices and invoice line items, including status transitions and invoice number generation.

## Steps
1. Create src/db/invoices.rs: repository functions using sqlx for create_invoice, get_invoice_by_id, list_invoices (with filters: status, customer_id, date range; pagination), update_invoice, soft_delete_invoice, create_line_item, list_line_items_by_invoice.
2. Create src/services/invoice_service.rs: business logic layer. Generate sequential invoice numbers (e.g., INV-2024-00001). Validate status transitions: DRAFT→SENT→PAID, DRAFT→CANCELLED, SENT→OVERDUE, SENT→VOID. Calculate subtotal/tax/total from line items.
3. Create src/routes/invoices.rs with Axum handlers:
   - POST /api/v1/invoices → create invoice (draft)
   - GET /api/v1/invoices → list invoices (query params for filters/pagination)
   - GET /api/v1/invoices/:id → get invoice detail with line items
   - PUT /api/v1/invoices/:id → update invoice (only DRAFT status)
   - POST /api/v1/invoices/:id/send → transition to SENT
   - POST /api/v1/invoices/:id/void → transition to VOID
   - DELETE /api/v1/invoices/:id → soft delete (only DRAFT)
   - POST /api/v1/invoices/:id/line-items → add line item
4. Use Axum extractors for JSON body validation with serde.
5. Recalculate totals whenever line items change.
6. Return proper HTTP status codes: 201 for create, 200 for reads, 409 for invalid state transitions.

## Validation
Unit tests for invoice number generation and status transition validation; integration tests verify full CRUD cycle: create draft → add line items → verify totals → send → mark paid; invalid transitions return 409; soft delete only works on DRAFT invoices; list endpoint pagination and filtering work correctly.