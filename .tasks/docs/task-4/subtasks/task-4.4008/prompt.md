Implement subtask 4008: Implement automated payment reminders and AR aging logic

## Objective
Build the background job that detects overdue invoices, updates their status, and triggers payment reminder notifications.

## Steps
1. Create `src/services/reminder_service.rs`. 2. Implement `check_overdue_invoices()` — query all invoices with status 'sent' where due_date < now(). Update status to 'overdue'. Return list of newly overdue invoices. 3. Implement `generate_payment_reminders()` — for overdue invoices, determine reminder schedule: first reminder at due_date + 1 day, second at + 7 days, third at + 14 days, final notice at + 30 days. Track reminders in a `finance.payment_reminders` table (id, invoice_id, reminder_type, sent_at, created_at). Add migration for this table. 4. Create a tokio::spawn background task in main.rs that runs every hour: calls check_overdue_invoices(), then generate_payment_reminders(). Use tokio::time::interval. 5. For v1, reminders are recorded in the database (no email sending). Expose GET /api/v1/invoices/:id/reminders to list reminders for an invoice. 6. Implement AR aging calculation helper that returns the aging bucket for a given invoice based on days_overdue.

## Validation
Create an invoice with a past due_date and verify the background job transitions it to overdue status. Verify payment reminders are created according to the schedule. Verify the reminders endpoint returns correct data. Verify the aging bucket calculation returns correct bucket for various days_overdue values.