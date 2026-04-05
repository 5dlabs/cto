Implement subtask 4005: Implement payment recording endpoints with partial payment support

## Objective
Build payment CRUD endpoints including standalone payment recording, invoice-linked payments with automatic status transitions on full payment, and partial payment tracking.

## Steps
1. Define structs in `src/models/payment.rs`: Payment, RecordPaymentRequest (invoice_id, amount_cents, currency, method, received_at, stripe_payment_id optional), PaymentListFilters.
2. Implement `src/db/payments.rs`:
   - `record_payment(pool, req)` — within a transaction: (a) INSERT payment row; (b) UPDATE invoices SET paid_amount_cents = paid_amount_cents + amount_cents, updated_at = now() WHERE id = invoice_id; (c) if paid_amount_cents >= total_cents, transition invoice status to 'paid' using the state machine; (d) COMMIT.
   - `list_payments(pool, filters)` — paginated, filterable by invoice_id, method, date range.
   - `get_payments_for_invoice(pool, invoice_id)` — all payments for a given invoice.
3. Implement Axum handlers in `src/routes/payments.rs`:
   - `POST /api/v1/invoices/:id/paid` → accepts payment details, calls record_payment, returns 200 with payment + updated invoice status.
   - `POST /api/v1/payments` → record standalone payment (must include invoice_id in body).
   - `GET /api/v1/payments` → list payments with filters.
   - `GET /api/v1/payments/invoice/:id` → payments for specific invoice.
4. Validation: payment currency must match invoice currency (or convert using currency_rates if different — for v1, reject mismatched currencies). Amount must be positive. Invoice must be in payable state (sent, viewed, overdue).
5. Idempotency for Stripe payments: if stripe_payment_id is provided, check for existing payment with same stripe_payment_id before inserting (return existing payment if found).
6. Register routes on the router.

## Validation
Integration test: create invoice (total_cents=10000) → send → record payment of 5000 → verify paid_amount_cents=5000, status remains 'sent'. Record another payment of 5000 → verify paid_amount_cents=10000, status transitions to 'paid'. Integration test: attempt payment on draft invoice returns 400. Integration test: payment with duplicate stripe_payment_id returns existing payment (idempotent). Integration test: GET /api/v1/payments/invoice/:id returns all payments for that invoice.