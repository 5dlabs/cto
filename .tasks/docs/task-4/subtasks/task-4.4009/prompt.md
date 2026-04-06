Implement subtask 4009: Implement automated payment reminder scheduling

## Objective
Build a background job that identifies invoices approaching or past their due date and triggers payment reminder notifications.

## Steps
1. Create src/jobs/payment_reminders.rs.
2. Implement a reminder scheduler that runs periodically (e.g., daily via tokio cron-like schedule):
   - Query invoices with status 'sent' where due_date is within N days (configurable, e.g., 7 days before, on due date, 7 days after)
   - Track reminder history to avoid duplicate sends (add payment_reminders table: id, invoice_id, reminder_type enum (upcoming/due_today/overdue), sent_at, status)
   - For each qualifying invoice without a recent reminder of that type, emit a reminder event
3. Create a reminder event/struct containing: invoice_id, client_id, invoice_number, amount_due, due_date, reminder_type.
4. For v1, implement a simple notification mechanism:
   - Log the reminder event with full details (structured log at INFO level)
   - Store reminder record in payment_reminders table
   - Expose GET /v1/invoices/:id/reminders to list sent reminders
   - The actual email/notification delivery will be handled by a future notification service
5. Add migration for payment_reminders table.
6. Ensure the job is idempotent (re-running doesn't create duplicate reminders for the same type+invoice+day).

## Validation
Reminder job identifies invoices due within 7 days and creates reminder records; overdue invoices get overdue-type reminders; already-reminded invoices (same type, same day) are not duplicated; reminder history endpoint returns correct records; job runs without error when no invoices qualify; job handles empty database gracefully.