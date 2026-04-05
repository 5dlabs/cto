Implement subtask 4008: End-to-end finance workflow tests

## Objective
Write comprehensive integration tests covering the full invoice lifecycle, Stripe payment flow, currency conversion, AR aging, payroll, and cross-cutting audit trail.

## Steps
1. Create `tests/` integration test directory.
2. Test 1 — Full invoice lifecycle: create invoice from opportunity → add line items → verify totals → send invoice (status: sent) → create Stripe payment intent → simulate Stripe webhook (payment_intent.succeeded) → verify invoice marked as paid → verify payment record → verify audit trail.
3. Test 2 — Multi-currency: sync currency rates → create invoice in EUR → convert displayed amount to USD → verify conversion accuracy → create payment in EUR → verify amount matches.
4. Test 3 — AR aging: create invoices with due dates at 0, 35, 65, and 95 days ago → generate aging report → verify each invoice is in the correct bucket → verify totals per bucket.
5. Test 4 — Payment reminders: create invoice due in 5 days → run reminder processor → verify reminder created → run again → verify no duplicate. Create overdue invoice → run overdue marking → verify status changed.
6. Test 5 — Payroll flow: create payroll record → verify calculations → approve → process → mark paid → verify audit trail.
7. Test 6 — Error handling: attempt to pay a cancelled invoice → verify rejection. Send invalid Stripe webhook signature → verify 400. Create payroll with negative hours → verify validation error.
8. Use testcontainers-rs or dedicated test database.

## Validation
All integration tests pass with `cargo test --test integration`. Tests cover happy paths, edge cases, and error conditions. Each test is isolated with its own data. Audit log completeness is verified for every state-changing operation. No test interdependencies.