Implement subtask 4003: Integrate Stripe for payment processing and webhook handling

## Objective
Implement Stripe payment intent creation, confirmation handling, and webhook endpoint for processing asynchronous payment events (succeeded, failed, refunded).

## Steps
1. Add `stripe-rust` crate dependency.
2. Create `src/services/stripe_service.rs`: initialize Stripe client with STRIPE_SECRET_KEY.
3. Implement `create_payment_intent(invoice_id, amount, currency)`: call Stripe API to create a PaymentIntent, store the payment_intent_id in the payments table with status 'pending', return client_secret for frontend.
4. Implement `src/routes/payments.rs`:
   - POST /api/v1/payments/create-intent — create payment intent for an invoice
   - GET /api/v1/payments/:id — get payment details
   - GET /api/v1/invoices/:id/payments — list payments for an invoice
5. Implement `src/routes/webhooks.rs`:
   - POST /api/v1/webhooks/stripe — Stripe webhook endpoint
   - Verify webhook signature using Stripe webhook secret from environment
   - Handle events: `payment_intent.succeeded` → update payment status to 'completed', update invoice status to 'paid', record paid_at timestamp, write audit log
   - Handle `payment_intent.payment_failed` → update payment status to 'failed', write audit log
   - Handle `charge.refunded` → create refund record, update payment status, write audit log
   - Return 200 for all handled events, 400 for signature verification failure
6. Implement idempotency: check if event has already been processed (store stripe_event_id) before applying changes.
7. All payment state changes must be transactional with their corresponding invoice updates.

## Validation
Unit tests with mocked Stripe client: verify payment intent creation stores correct data. Integration tests: simulate webhook payloads with valid signatures, verify payment and invoice statuses update correctly. Test idempotency: send same webhook event twice, verify no duplicate processing. Test invalid signature returns 400. Test payment_failed event marks payment as failed without changing invoice to paid.