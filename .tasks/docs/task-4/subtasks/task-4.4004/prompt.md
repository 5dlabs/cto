Implement subtask 4004: Integrate Stripe for payment processing and webhook handling

## Objective
Implement Stripe payment intent creation, payment processing, and webhook handling to record Stripe payment events against invoices.

## Steps
1. Add stripe-rust crate dependency.
2. Read STRIPE_SECRET_KEY and STRIPE_WEBHOOK_SECRET from environment/secrets.
3. Create src/stripe_client.rs with a StripeService:
   - create_payment_intent(invoice_id, amount, currency) → creates a Stripe PaymentIntent, returns client_secret and payment_intent_id
   - retrieve_payment_intent(payment_intent_id) → fetches status from Stripe
4. Add endpoint POST /v1/invoices/:id/pay:
   - Create a Stripe PaymentIntent for the invoice amount
   - Store payment_intent_id in a pending payment record
   - Return client_secret to the frontend for Stripe.js confirmation
5. Implement POST /v1/webhooks/stripe:
   - Verify webhook signature using STRIPE_WEBHOOK_SECRET
   - Handle events: payment_intent.succeeded → mark payment as succeeded, update invoice status to paid
   - Handle payment_intent.payment_failed → mark payment as failed
   - Handle charge.refunded → record refund, update invoice status
   - Return 200 to Stripe for all handled events
6. Ensure idempotency: check if payment already recorded before creating duplicate.
7. Create a mock Stripe client for testing.

## Validation
POST /v1/invoices/:id/pay returns a client_secret and creates a pending payment; webhook with valid signature for payment_intent.succeeded marks payment as succeeded and invoice as paid; webhook with invalid signature returns 401; duplicate webhook delivery doesn't create duplicate payments; refund webhook updates payment and invoice status correctly.