Implement subtask 4005: Implement Stripe webhook handler for asynchronous payment events

## Objective
Build a secure Stripe webhook endpoint that processes payment lifecycle events (succeeded, failed, refunded) and updates payment/invoice records accordingly.

## Steps
1. Create src/routes/webhooks.rs with POST /api/v1/webhooks/stripe endpoint.
2. Implement Stripe webhook signature verification: read the raw request body, verify against STRIPE_WEBHOOK_SECRET using Stripe's signature scheme (v1 HMAC-SHA256). Reject requests with invalid signatures (return 400).
3. Parse the webhook event type and dispatch to handlers:
   - payment_intent.succeeded: find Payment by stripe_payment_intent_id, update status to SUCCEEDED, update invoice status to PAID if fully paid.
   - payment_intent.payment_failed: find Payment, update status to FAILED, log failure reason.
   - charge.refunded: find Payment by stripe_charge_id, update status to REFUNDED, recalculate invoice payment status (revert to SENT if no successful payments remain).
4. Implement idempotent webhook processing: store processed event IDs in Redis with TTL to prevent duplicate processing.
5. Return 200 immediately for recognized event types to acknowledge receipt.
6. Return 200 for unrecognized event types (don't fail on events we don't handle).
7. Log all webhook events with tracing for audit trail.

## Validation
Unit tests verify webhook signature validation rejects invalid signatures and accepts valid ones; payment_intent.succeeded correctly updates payment to SUCCEEDED and invoice to PAID; payment_intent.payment_failed updates payment to FAILED; charge.refunded reverts invoice status; duplicate event IDs are skipped; unrecognized event types return 200 without error.