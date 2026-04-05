Implement subtask 4003: Implement invoice CRUD endpoints with line items and invoice number generation

## Objective
Build the invoice domain model, repository layer, and Axum handlers for creating, listing, and retrieving invoices including line items and sequential per-org invoice number generation.

## Steps
1. Define Rust structs in `src/models/invoice.rs`: Invoice, InvoiceLine, CreateInvoiceRequest (with Vec<CreateLineItem>), InvoiceListFilters (status, date range, overdue flag, pagination), InvoiceResponse (with line items and payment summary).
2. All monetary fields use `rust_decimal::Decimal` in the application layer, converting to/from i64 cents at the DB boundary. Create helper functions `cents_to_decimal(i64) -> Decimal` and `decimal_to_cents(Decimal) -> i64` in a `src/models/money.rs` module.
3. Implement `src/db/invoices.rs` with:
   - `create_invoice(pool, org_id, req)` — within a transaction: (a) SELECT FOR UPDATE on invoice_number_counters to get and increment the next number for (org_id, year), INSERT if not exists; (b) format invoice_number as `{ORG_PREFIX}-{YEAR}-{PADDED_NUMBER}`; (c) compute line item subtotals (quantity * unit_price_cents); (d) sum line subtotals for invoice subtotal_cents; (e) INSERT invoice row; (f) INSERT all line items; (g) COMMIT and return the full invoice.
   - `list_invoices(pool, org_id, filters)` — paginated query with optional WHERE clauses for status, date range, overdue (due_at < now() AND status IN ('sent','viewed')). Return Vec<Invoice> with total count for pagination.
   - `get_invoice(pool, id)` — fetch invoice with LEFT JOIN on line items and a subquery for payment summary (count, total paid).
4. Implement Axum handlers in `src/routes/invoices.rs`:
   - `POST /api/v1/invoices` → calls create_invoice, returns 201 with invoice JSON.
   - `GET /api/v1/invoices` → parses query params into InvoiceListFilters, calls list_invoices, returns paginated response.
   - `GET /api/v1/invoices/:id` → calls get_invoice, returns 200 or 404.
5. Register routes on the router. Apply shared-auth middleware for org_id extraction from JWT.
6. Validate inputs: currency must be one of supported currencies, due_at must be in the future, at least one line item required.
7. Use shared-error for consistent error responses (400, 404, 500).

## Validation
Unit test: money conversion helpers round-trip correctly (cents→decimal→cents). Unit test: invoice number formatting produces expected pattern. Integration test: POST /api/v1/invoices with valid payload returns 201, response has correct invoice_number format, subtotal_cents equals sum of line items. Integration test: GET /api/v1/invoices returns paginated list, filters by status correctly. Integration test: GET /api/v1/invoices/:id returns full invoice with line items. Integration test: concurrent invoice creation for same org produces sequential, non-duplicate invoice numbers.