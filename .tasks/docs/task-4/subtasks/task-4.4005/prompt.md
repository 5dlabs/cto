Implement subtask 4005: Implement invoice CRUD endpoints with tax calculation

## Objective
Build the invoice REST endpoints: POST create (with line items and auto tax calculation), GET list (with filtering), GET by id, under `/api/v1/invoices`. Includes invoice number auto-generation.

## Steps
1. Create `services/rust/finance/src/routes/invoices.rs` and `services/rust/finance/src/db/invoices.rs`.
2. Define request/response models with serde + utoipa:
   - `CreateInvoiceRequest`: project_id, org_id, customer_name, customer_email, jurisdiction, currency, line_items (vec of {description, quantity, unit_price_cents}), due_at.
   - `InvoiceResponse`: all invoice fields plus line_items array.
   - `InvoiceListResponse`: vec of invoices with pagination (offset/limit).
3. `POST /api/v1/invoices`:
   - Validate required fields.
   - Generate invoice_number using a pattern like `INV-{org_short}-{sequential}` (use a DB sequence or counter).
   - Calculate subtotal_cents by summing line item amounts (quantity * unit_price_cents).
   - Call TaxCalculator to get tax_cents based on jurisdiction.
   - Compute total_cents = subtotal_cents + tax_cents.
   - Insert invoice and line items in a transaction.
   - Return 201 with full invoice response.
4. `GET /api/v1/invoices`:
   - Query params: org_id (required), status (optional), project_id (optional), offset, limit.
   - Return paginated list with total count.
5. `GET /api/v1/invoices/:id`:
   - Return full invoice with line items.
   - 404 if not found.
6. Wire routes into Axum router with shared auth middleware.
7. Add utoipa annotations for OpenAPI generation.

## Validation
Integration tests (sqlx::test): Create invoice with 3 line items in CA-ON jurisdiction, verify subtotal_cents = sum of line items, tax_cents = 13% of subtotal, total_cents = subtotal + tax. Verify invoice_number is generated and unique. GET list with org_id filter returns created invoice. GET by id returns full invoice with line items. Verify 404 for non-existent id. Verify 400 for missing required fields. Test pagination with 15 invoices, limit=10.