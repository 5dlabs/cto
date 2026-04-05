Implement subtask 4003: Implement Stripe payment integration with webhook handling

## Objective
Integrate with Stripe API for processing payments, recording payment results, and handling Stripe webhooks for asynchronous payment events.

## Steps
1. Add stripe-rust crate dependency. 2. Implement Stripe client initialization in /src/services/stripe.rs using the Stripe secret key from Kubernetes secret (loaded via environment variable). 3. Implement POST /api/v1/payments endpoint: accept invoice_id and payment method, create a Stripe PaymentIntent, return the client_secret for frontend confirmation. 4. Implement POST /api/v1/payments/webhook endpoint: verify Stripe webhook signature using the webhook secret, handle events: payment_intent.succeeded (mark payment as completed, update invoice status to paid), payment_intent.payment_failed (mark payment as failed, log reason), charge.refunded (record refund). 5. Implement GET /api/v1/payments (list payments with filters) and GET /api/v1/payments/:id (get payment details). 6. Ensure idempotency: use Stripe's idempotency keys on PaymentIntent creation. 7. Record all payment state transitions in the payments table with timestamps.

## Validation
PaymentIntent creation returns valid client_secret; webhook handler correctly verifies signatures and rejects invalid ones; successful payment webhook updates invoice status to paid; failed payment webhook records failure reason; duplicate webhook deliveries are handled idempotently; payments table reflects accurate state.