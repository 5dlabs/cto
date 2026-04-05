## Acceptance Criteria

- [ ] 1. Unit test: tax calculation for Canadian GST (5%), Ontario HST (13%), and zero-tax jurisdiction produces correct tax_cents values for known subtotals. 2. Unit test: AR aging report correctly buckets invoices into current/30/60/90+ day categories given test data with various due dates. 3. Integration test: create invoice → send (mock Stripe) → record payment → verify status transitions draft→sent→paid and paid_amount_cents equals total_cents. 4. Integration test: partial payment records correctly, status remains 'sent', paid_amount_cents < total_cents. 5. Integration test: create invoice with CAD currency, verify currency field persists through all operations. 6. Stripe webhook test: POST to /api/v1/webhooks/stripe with valid Stripe signature and payment_intent.succeeded event, verify payment recorded and invoice status updated. 7. Currency rate sync test: mock HTTP response, verify rates stored in DB and cached in Valkey. 8. Overdue detection test: create invoice with due_at in the past and status 'sent', trigger background task, verify status changed to 'overdue'. 9. Performance test: invoice generation (create + persist) completes in < 5 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.