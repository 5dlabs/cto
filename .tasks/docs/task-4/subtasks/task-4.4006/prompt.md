Implement subtask 4006: Implement Stripe integration module with invoice creation and webhook handler

## Objective
Build the Stripe integration module using stripe-rust crate: create Stripe invoices when an invoice is sent with card/stripe method, and handle payment_intent.succeeded webhooks with idempotency.

## Steps
1. Create `src/stripe/client.rs`:
   - Initialize stripe::Client from STRIPE_SECRET_KEY env var.
   - Add to AppState.
2. Create `src/stripe/invoices.rs`:
   - `create_stripe_invoice(client, invoice, line_items)` — creates a Stripe Invoice with line items, returns stripe_invoice_id.
   - Maps internal line items to Stripe InvoiceItem objects.
   - Stores the returned stripe_invoice_id on the local invoice record.
3. Modify `POST /api/v1/invoices/:id/send` handler (from subtask 4004):
   - After transitioning to 'sent', check if payment method is card/stripe.
   - If so, call create_stripe_invoice, store stripe_invoice_id, and finalize the Stripe invoice (which triggers Stripe to send the invoice).
   - If Stripe call fails, roll back the status transition or mark for retry.
4. Create `src/stripe/webhooks.rs`:
   - `POST /api/v1/webhooks/stripe` handler.
   - Extract raw body bytes using a custom Axum extractor (needed for signature verification).
   - Verify webhook signature using stripe::Webhook::construct_event with STRIPE_WEBHOOK_SECRET.
   - Handle `payment_intent.succeeded` event: extract payment intent ID, find matching invoice by stripe_invoice_id or metadata, call record_payment with stripe_payment_id for idempotency.
   - Handle `invoice.payment_failed` event: log warning, optionally update internal status.
   - Return 200 for handled events, 200 for unhandled events (Stripe expects 2xx).
5. Idempotency: the record_payment function (from 4005) already checks for duplicate stripe_payment_id. The webhook handler relies on this.
6. Add STRIPE_SECRET_KEY and STRIPE_WEBHOOK_SECRET to required env vars. In dev/test, these can be Stripe test-mode keys.
7. Register webhook route WITHOUT auth middleware (Stripe calls this externally; signature verification is the auth).

## Validation
Unit test: webhook signature verification rejects invalid signatures and accepts valid ones (use stripe-rust test helpers). Integration test: mock Stripe API, POST /api/v1/invoices/:id/send for a card-method invoice → verify Stripe invoice creation is called and stripe_invoice_id is stored. Integration test: POST /api/v1/webhooks/stripe with a valid payment_intent.succeeded payload and correct signature → verify payment is recorded and invoice status is updated. Integration test: replay the same webhook event → verify no duplicate payment is created (idempotency). Integration test: webhook with invalid signature returns 400.