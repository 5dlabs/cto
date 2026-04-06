Implement subtask 4014: Write integration tests for Finance service end-to-end flows

## Objective
Write integration tests covering invoice lifecycle, aging bucket correctness, Stripe webhook processing, and GDPR anonymization.

## Steps
Create tests/integration.rs (Rust integration test file). Use sqlx::PgPool pointed at a test PostgreSQL instance (managed via testcontainers-rs or a pre-seeded CI database). Spin up the Axum app in a tokio::test with a bound listener. Test cases: (1) Full invoice lifecycle: create → send → receive Stripe webhook → verify paid status. (2) Aging report: seed invoice issued_at = NOW()-35d with status=sent; GET /aging returns count=1 in 31-60 bucket. (3) Overdue task: seed invoice due_at=NOW()-2d status=sent, call the update function directly, assert status=overdue. (4) GDPR delete: create invoice for org_id X, DELETE /internal/gdpr/delete/X, re-query invoices, org_id is NULL. (5) Currency rates: call sync function with mocked HTTP, assert DB row inserted and Valkey key set. Run with `cargo test --test integration`.

## Validation
cargo test --test integration exits 0 with all 5 scenarios passing. cargo test -- --nocapture shows per-test logs. cargo tarpaulin or cargo llvm-cov reports >= 80% line coverage across src/. No async deadlocks (test timeout = 30s per test).