Implement subtask 4006: Implement invoice send and payment recording endpoints

## Objective
Build the invoice action endpoints: POST send (mark as sent), POST paid (record payment with partial/full tracking and status transitions), and payment CRUD endpoints under `/api/v1/payments`.

## Steps
1. `POST /api/v1/invoices/:id/send`:
   - Validate invoice exists and status allows transition to Sent (use state machine).
   - Set `issued_at = now()` if not already set.
   - Transition status to Sent.
   - Return updated invoice.
   - (Stripe integration will be added in a separate subtask — this endpoint just does the local state transition for now.)
2. `POST /api/v1/invoices/:id/paid`:
   - Accept `PaymentRequest`: amount_cents, currency, method.
   - Validate invoice exists and is in a payable state (Sent, Viewed, Overdue).
   - Insert payment record in `payments` table.
   - Update invoice `paid_amount_cents += amount_cents`.
   - If `paid_amount_cents >= total_cents`, transition status to Paid.
   - All in a single DB transaction with row-level locking (`SELECT ... FOR UPDATE`).
   - Return updated invoice with payment details.
3. `POST /api/v1/payments`:
   - Direct payment recording (same logic as above but via payments endpoint).
4. `GET /api/v1/payments`:
   - Query params: invoice_id (optional), org_id, offset, limit.
5. `GET /api/v1/payments/invoice/:invoice_id`:
   - List all payments for a specific invoice.
6. Handle partial payments: if amount_cents < remaining, keep invoice in current status but update paid_amount_cents.
7. Handle overpayment: reject payments where amount_cents would cause paid_amount_cents > total_cents.
8. Add utoipa annotations.

## Validation
Integration tests: (1) Create invoice, send it, verify status=Sent and issued_at set. (2) Record partial payment (50% of total), verify paid_amount_cents updated, status unchanged. (3) Record remaining payment, verify status transitions to Paid. (4) Attempt to pay a Draft invoice, verify 400 error. (5) Attempt overpayment, verify rejection. (6) GET payments by invoice_id returns both payments. (7) Verify DB transaction atomicity: concurrent payments don't cause race conditions (test with row locking).