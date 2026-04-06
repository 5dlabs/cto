Implement subtask 4005: Implement Stripe invoice creation and webhook endpoint with signature validation

## Objective
On POST /api/v1/invoices/:id/send when stripe_invoice_id is null, create a Stripe Invoice via the Stripe API and store the returned ID. Implement POST /api/v1/webhooks/stripe to validate Stripe-Signature and process invoice.paid and payment_intent.succeeded events.

## Steps
Create src/integrations/stripe.rs. Configure Stripe client using STRIPE_SECRET_KEY from environment. On /send: if invoice.stripe_invoice_id IS NULL, call Stripe CreateInvoice API with customer email and total_cents, store returned stripe_invoice_id on the invoice row. POST /api/v1/webhooks/stripe: read raw request body as bytes before any deserialization (required for HMAC validation). Validate Stripe-Signature header using STRIPE_WEBHOOK_SECRET via the stripe-rust verify_webhook or manual HMAC-SHA256 computation with reqwest. On validation failure return 400. Parse event type: invoice.paid → call invoice_payment service to mark invoice paid and insert payment row with method=stripe. payment_intent.succeeded → log and update stripe_payment_id on the matching payment row. Return 200 to Stripe on all successfully processed events.

## Validation
POST /api/v1/webhooks/stripe with valid Stripe-Signature and invoice.paid body updates invoice status to paid (verify via GET /api/v1/invoices/:id). POST with invalid signature returns 400. POST with payment_intent.succeeded updates stripe_payment_id on payment row. Invoice send with null stripe_invoice_id results in non-null stripe_invoice_id after call.