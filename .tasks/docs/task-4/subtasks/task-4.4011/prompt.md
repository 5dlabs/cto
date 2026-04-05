Implement subtask 4011: Implement automated overdue invoice detection background task

## Objective
Build the hourly background task that detects invoices past their due date and transitions them from sent/viewed to overdue status.

## Steps
1. Create `src/background/overdue_detection.rs`.
2. Implement `detect_and_mark_overdue(pool) -> Result<u64>`:
   - Query: UPDATE finance.invoices SET status = 'overdue', updated_at = now() WHERE due_at < now() AND status IN ('sent', 'viewed') RETURNING id.
   - Return the count of updated invoices.
   - Log each transitioned invoice ID at info level.
3. Implement `spawn_overdue_detection_task(pool)`:
   - Spawns a tokio task that loops: detect_and_mark_overdue → log count → tokio::time::sleep(1 hour).
   - On error, log at error level and continue (do not crash the task).
4. Call spawn_overdue_detection_task from main.rs.
5. Ensure the UPDATE query is idempotent and safe for concurrent execution (if multiple replicas run this, the WHERE clause prevents double-updates since status won't match after first update).

## Validation
Integration test: create an invoice with status 'sent' and due_at 2 days in the past. Call detect_and_mark_overdue directly. Verify status changed to 'overdue'. Integration test: create an invoice with status 'sent' and due_at in the future. Call detect_and_mark_overdue. Verify status remains 'sent'. Integration test: create an invoice with status 'paid' and due_at in the past. Verify it is NOT transitioned to overdue. Integration test: run detect_and_mark_overdue twice in a row on the same data, verify it returns 0 on the second run (idempotency).