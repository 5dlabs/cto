<identity>
You are rex working on subtask 4003 of task 4.
</identity>

<context>
<scope>
Implement POST /api/v1/invoices (create with tax computation), GET /api/v1/invoices (list), GET /api/v1/invoices/:id, POST /api/v1/invoices/:id/send, and POST /api/v1/invoices/:id/paid handlers in Axum.
</scope>
</context>

<implementation_plan>
Create src/handlers/invoices.rs. POST /api/v1/invoices: accept JSON body with project_id, org_id, currency, line_items array. Compute subtotal_cents = sum(quantity * unit_price_cents). Apply tax: if currency == 'CAD' tax_cents = subtotal_cents * 13 / 100 else tax_cents = 0. total_cents = subtotal_cents + tax_cents. Generate invoice_number as 'INV-' + sequential padded number. INSERT invoices and invoice_line_items in a sqlx transaction. Return 201. GET /api/v1/invoices: SELECT with optional status and org_id query params. GET /api/v1/invoices/:id: SELECT with 404 on missing. POST /api/v1/invoices/:id/send: UPDATE status=sent if current status=draft; log 'email would be sent' for now (SMTP deferred). POST /api/v1/invoices/:id/paid: INSERT payment row, UPDATE paid_amount_cents += amount_cents, if paid_amount_cents >= total_cents SET status=paid. Use rust_decimal for all financial arithmetic to avoid floating point. Define InvoiceStatus enum with state machine guard (e.g., cannot send a cancelled invoice).
</implementation_plan>

<validation>
POST /api/v1/invoices with CAD currency and line items: total_cents = subtotal + 13% tax, status=draft, returns 201. POST send → status becomes sent. POST paid with exact total_cents → status becomes paid. POST paid again → returns 400 (already paid). GET list with status=draft filter returns only draft invoices.
</validation>