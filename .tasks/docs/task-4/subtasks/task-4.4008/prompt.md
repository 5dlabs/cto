Implement subtask 4008: Write integration tests for all finance endpoints and Stripe flows

## Objective
Create comprehensive integration tests covering invoice lifecycle, payment processing with Stripe mocks, payroll workflows, currency sync, and financial reports.

## Steps
1. Set up test infrastructure: use testcontainers (via testcontainers-rs or docker-compose) to spin up PostgreSQL and Redis for integration tests.
2. Create a mock Stripe server (using wiremock-rs or similar) that simulates PaymentIntent creation, retrieval, and webhook events.
3. Write invoice lifecycle test: create draft → add line items → verify totals → send → create payment → webhook succeeded → verify PAID status.
4. Write payment idempotency test: create payment with same idempotency key twice → verify only one Stripe PaymentIntent created.
5. Write webhook security test: send webhook with invalid signature → verify 400; send with valid signature → verify 200 and correct state update.
6. Write refund flow test: successful payment → refund webhook → verify payment REFUNDED and invoice reverted to SENT.
7. Write payroll lifecycle test: create → approve → pay → verify timestamps.
8. Write currency sync test: mock external API → trigger sync → verify rates in DB and Redis → verify convert endpoint.
9. Write financial reports test: create multiple invoices/payments across currencies and periods → verify revenue, payment, and outstanding reports return correct aggregations.
10. Run coverage analysis and ensure ≥80% coverage across src/routes, src/services, src/db modules.

## Validation
All integration test suites pass with containerized PostgreSQL and Redis and mocked Stripe; end-to-end invoice-to-payment flow completes correctly; webhook signature verification works; idempotency prevents duplicates; financial reports aggregate correctly across currencies; coverage report shows ≥80%.