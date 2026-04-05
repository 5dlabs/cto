Implement subtask 4010: Write integration tests for end-to-end finance workflows

## Objective
Create comprehensive integration tests covering the full invoice-to-payment lifecycle, Stripe webhook processing, payroll workflow, and financial reporting accuracy.

## Steps
1. Set up test infrastructure using testcontainers or sqlx::test with a real PostgreSQL instance. Mock Stripe API calls using a test double/mockserver. 2. Test 1 - Invoice-to-Payment E2E: Create invoice with line items → verify totals calculated correctly → initiate Stripe payment (mock PaymentIntent creation) → simulate Stripe webhook 'payment_intent.succeeded' → verify payment recorded → verify invoice status is 'paid'. 3. Test 2 - Partial Payment: Create invoice → record partial payment → verify invoice still 'sent' → record remaining payment → verify invoice 'paid'. 4. Test 3 - Webhook Idempotency: Send same Stripe webhook event twice → verify only one payment record created. 5. Test 4 - Payroll Workflow: Create payroll entry → attempt to pay (should fail, still draft) → approve → pay → verify paid_date set. 6. Test 5 - Financial Reports: Seed invoices and payments across 3 months → verify revenue report aggregates correctly → verify outstanding report excludes paid invoices. 7. Test 6 - Currency: Insert currency rates → create invoice in EUR → verify conversion to USD in revenue report uses correct rate. 8. All tests use isolated transactions or separate databases to prevent interference.

## Validation
All 6 integration test scenarios pass in CI; tests run with real PostgreSQL via testcontainers; Stripe interactions are mocked but webhook signature verification uses test secrets; test coverage report shows >80% on service and handler modules.