## Acceptance Criteria

- [ ] 1. POST /api/v1/invoices with valid project_id and line items returns 201 with status=draft and computed total_cents = subtotal_cents + tax_cents. 2. POST /api/v1/invoices/:id/paid with amount equal to total_cents returns invoice status=paid. 3. GET /api/v1/finance/reports/aging returns JSON with buckets 0-30, 31-60, 61-90, 90+ each as numeric counts; overdue invoice (created 35 days ago, unpaid) appears in 31-60 bucket. 4. GET /api/v1/currency/rates returns JSON object with at least USD, CAD, AUD keys and non-zero rate values. 5. Stripe webhook POST /api/v1/webhooks/stripe with valid Stripe-Signature header and invoice.paid payload updates invoice status to paid (verify via GET /api/v1/invoices/:id). 6. Overdue reminder task: seed invoice with due_at = NOW()-2d and status=sent; after background task runs, status becomes overdue. 7. GET /health/ready returns 200. 8. cargo test passes with >= 80% coverage.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.