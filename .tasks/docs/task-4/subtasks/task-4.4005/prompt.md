Implement subtask 4005: Integrate Stripe API for payment processing and webhook handling

## Objective
Implement Stripe PaymentIntent creation for invoices, payment confirmation flow, and Stripe webhook endpoint to handle asynchronous payment events.

## Steps
1. Add stripe-rust crate as a dependency. 2. Create src/services/stripe.rs: Initialize Stripe client with STRIPE_SECRET_KEY from config. 3. Implement create_payment_intent: given an invoice, create a Stripe PaymentIntent with amount (in smallest currency unit), currency, metadata (invoice_id). Return client_secret for frontend and store stripe_payment_intent_id on the invoice. 4. Implement POST /api/v1/invoices/:id/pay endpoint that creates a PaymentIntent and returns the client_secret. 5. Implement POST /api/v1/webhooks/stripe endpoint: verify webhook signature using STRIPE_WEBHOOK_SECRET, parse event type. Handle 'payment_intent.succeeded': look up invoice by payment_intent metadata, create Payment record with status 'succeeded', update invoice status if fully paid. Handle 'payment_intent.payment_failed': create Payment record with status 'failed', log details. 6. Make webhook handler idempotent: check if a payment with the same stripe_payment_intent_id already exists before creating a duplicate. 7. Add error handling for Stripe API failures (network errors, invalid requests).

## Validation
Unit tests with mocked Stripe client verify PaymentIntent creation with correct parameters; webhook handler correctly parses and verifies signatures; idempotency test: sending the same webhook event twice creates only one payment record; integration test with Stripe test mode: create PaymentIntent, simulate success webhook, verify invoice marked as paid.