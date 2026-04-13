<identity>
You are rex working on subtask 4009 of task 4.
</identity>

<context>
<scope>
Implement a daily tokio background task that queries invoices with status=sent and due_at more than 1 day past, transitions them to status=overdue, and logs a reminder entry.
</scope>
</context>

<implementation_plan>
Create src/tasks/overdue_reminder.rs. Spawn a tokio::task::spawn loop at server startup: sleep until next midnight UTC, then repeat every 24 hours. On each run: UPDATE finance.invoices SET status='overdue' WHERE status='sent' AND due_at < NOW() - INTERVAL '1 day' RETURNING id, org_id, invoice_number. For each returned row: log tracing::info!('Invoice {} for org {} is now overdue — reminder enqueued', invoice_number, org_id). Include a TODO comment referencing Morgan integration for real email dispatch. Ensure the task handles DB errors gracefully (log error, continue loop without crashing).
</implementation_plan>

<validation>
Seed invoice with due_at = NOW() - INTERVAL '2 days' and status=sent. Trigger task manually by setting sleep to 0 in test configuration or calling the update function directly. Assert invoice status is overdue via GET /api/v1/invoices/:id. Verify log output contains the invoice number. DB error during update does not panic the task (test with poisoned DB connection).
</validation>