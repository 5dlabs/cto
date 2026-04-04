Implement subtask 4011: Implement automated overdue invoice detection background task

## Objective
Build the background task that periodically checks for sent invoices past their due date and transitions them to overdue status, exposing overdue invoices for reminder queries.

## Steps
1. Create `services/rust/finance/src/background/overdue_detector.rs`.
2. Define `OverdueDetector` struct with DB pool.
3. Implement `check_overdue(&self) -> Result<u64>`:
   - SQL: `UPDATE invoices SET status = 'overdue', updated_at = now() WHERE status IN ('sent', 'viewed') AND due_at < now() RETURNING id`.
   - Return count of invoices transitioned.
   - Log each transitioned invoice ID at info level.
4. Implement `run_detection_loop(&self)` that runs `check_overdue()` every 15 minutes using `tokio::time::interval`.
5. Spawn in `main.rs` alongside currency sync and Axum server.
6. Add a query endpoint `GET /api/v1/invoices/overdue`:
   - Query params: org_id (required).
   - Return list of overdue invoices with customer info, amount, days overdue.
   - This endpoint is consumed by Morgan (notification service) for sending reminders.
7. Handle the edge case: invoices that receive payment between detection runs should not be marked overdue (the state machine already prevents Paid→Overdue, but verify the SQL only targets sent/viewed).
8. Configurable interval via `OVERDUE_CHECK_INTERVAL_SECS` (default 900).

## Validation
Integration test: (1) Create invoice with due_at in the past and status=Sent, run check_overdue, verify status changed to Overdue. (2) Create invoice with due_at in the future and status=Sent, verify not changed. (3) Create invoice with status=Paid and due_at in past, verify not changed. (4) GET /api/v1/invoices/overdue returns only overdue invoices. (5) Verify return count matches number of transitioned invoices. (6) Run detection twice on same data, verify idempotent (already overdue invoices not re-processed).