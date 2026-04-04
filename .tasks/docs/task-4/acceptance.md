## Acceptance Criteria

- [ ] 1. Unit tests for tax calculation: verify GST/HST computation for each Canadian province and US placeholder — parameterized tests with expected cents values. 2. Unit tests for invoice status machine: draft→sent→viewed→paid, draft→sent→overdue, with rejection of invalid transitions. 3. Integration tests (sqlx::test): create invoice from project, record partial payment, verify paid_amount_cents updated; record remaining payment, verify status transitions to paid. 4. Aging report test: seed invoices with various due dates, verify correct bucketing (0-30, 31-60, 61-90, 90+). 5. Stripe webhook test: mock Stripe webhook payload for `invoice.paid`, send to webhook endpoint, verify invoice status updated to paid. 6. Currency rate sync test: mock exchange rate API response, verify rates stored in DB and cached in Valkey. 7. Invoice generation benchmark: verify < 5 seconds for invoice creation with 50 line items. 8. GDPR test: create invoice + payments, call DELETE, verify customer fields anonymized but financial totals preserved. 9. OpenAPI spec validates. 10. Minimum 80% coverage.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.