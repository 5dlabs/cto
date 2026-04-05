Implement subtask 4004: Implement Stripe payment processing integration

## Objective
Build payment endpoints that integrate with the Stripe API for creating payment intents, processing payments against invoices, and recording payment outcomes.

## Steps
1. Add stripe-rust crate (or implement raw HTTP client to Stripe API using reqwest if stripe-rust doesn't support 0.7 Axum well).
2. Create src/services/stripe_service.rs: initialize Stripe client with STRIPE_SECRET_KEY from config. Implement: create_payment_intent(amount, currency, invoice_id metadata), retrieve_payment_intent, confirm_payment_intent.
3. Create src/db/payments.rs: repository functions for create_payment, get_payment, list_payments_by_invoice, update_payment_status.
4. Create src/routes/payments.rs with Axum handlers:
   - POST /api/v1/invoices/:id/payments → create payment: validate invoice is SENT or OVERDUE, create Stripe PaymentIntent, record Payment with PENDING status, return client_secret for frontend.
   - GET /api/v1/invoices/:id/payments → list payments for invoice.
   - GET /api/v1/payments/:id → get payment detail.
5. Implement idempotency: use idempotency_key on payment creation to prevent duplicate charges. Check if payment with same key exists before creating Stripe PaymentIntent.
6. On successful payment recording, update invoice status to PAID if total payments >= invoice total.
7. Handle Stripe API errors gracefully: map to appropriate HTTP error responses with user-friendly messages.

## Validation
Unit tests with mocked Stripe client verify PaymentIntent creation with correct amount/currency/metadata; idempotency key prevents duplicate payment creation; payment recording updates invoice status to PAID when fully paid; Stripe API errors are mapped to appropriate HTTP 4xx/5xx responses; integration test with Stripe test mode (if keys available) creates and retrieves a PaymentIntent.