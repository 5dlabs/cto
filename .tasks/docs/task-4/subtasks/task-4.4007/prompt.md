Implement subtask 4007: Implement Stripe integration for invoice creation and webhook handling

## Objective
Integrate with the Stripe API to create Stripe Invoices when sending invoices, and implement the webhook endpoint to handle `invoice.paid` and `payment_intent.succeeded` events for automatic status updates.

## Steps
1. Create `services/rust/finance/src/stripe/mod.rs` and `services/rust/finance/src/stripe/client.rs`.
2. Define a `StripeClient` struct wrapping `reqwest::Client` with base URL and API key.
   - Read Stripe API key from environment variable `STRIPE_SECRET_KEY` (sourced from `sigma1-stripe-credentials` secret).
   - Read webhook signing secret from `STRIPE_WEBHOOK_SECRET`.
3. Implement `create_invoice(&self, invoice: &Invoice) -> Result<StripeInvoiceId>`:
   - POST to Stripe `/v1/invoices` with customer email, line items, currency, amounts.
   - Parse response, extract Stripe invoice ID.
   - Optionally finalize and send via Stripe.
4. Modify `POST /api/v1/invoices/:id/send`:
   - If Stripe is configured (API key present), call `create_invoice` and store `stripe_invoice_id` on the invoice.
   - If Stripe is not configured, just transition status locally.
5. Implement `POST /api/v1/webhooks/stripe`:
   - Parse raw request body.
   - Verify webhook signature using Stripe signing secret and `stripe-signature` header.
   - Parse event type from JSON payload.
   - For `invoice.paid`: look up invoice by `stripe_invoice_id`, record payment, transition to Paid.
   - For `payment_intent.succeeded`: similar lookup and update.
   - Return 200 OK to Stripe.
   - Log unhandled event types at debug level.
6. Make Stripe integration optional — service should work fully without Stripe configured (feature flag or runtime check on env var presence).
7. Add utoipa annotations for webhook endpoint.

## Validation
Unit test: mock Stripe API responses for invoice creation (success and failure cases). Integration test: mock Stripe webhook payload for `invoice.paid` with valid signature, send to webhook endpoint, verify invoice status updated to Paid and payment recorded. Test invalid webhook signature returns 401. Test with Stripe unconfigured: send invoice works without Stripe call. Test unhandled event types return 200 without side effects.