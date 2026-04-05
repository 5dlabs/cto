Implement subtask 4005: Implement automated payment reminders and AR aging logic

## Objective
Build the accounts receivable aging report generation and automated payment reminder system that identifies overdue invoices, categorizes them into aging buckets, and triggers reminder notifications.

## Steps
1. Create `src/services/ar_service.rs`.
2. Implement AR aging buckets: Current (0-30 days), 30-60 days, 60-90 days, 90+ days. For each unpaid/overdue invoice, calculate days_past_due from due_date and categorize.
3. Implement `generate_aging_report(customer_id: Option<UUID>) -> ARAgingReport`: query all sent/overdue invoices, group by aging bucket, calculate totals per bucket and grand total. Optionally filter by customer.
4. Create `src/services/reminder_service.rs`.
5. Implement reminder logic: query invoices where status is 'sent' and due_date is approaching (configurable: 7 days before, on due date, 7 days after, 30 days after). For each, check if a reminder was already sent for this interval (store in reminders table: id, invoice_id, reminder_type, sent_at).
6. Implement `process_reminders()`: for each qualifying invoice, create a reminder record. For v1, log the reminder event and write to audit_log (actual email/notification integration deferred to notification service).
7. Create background job in `src/jobs/reminders.rs`: run daily via tokio::spawn + tokio::time::interval.
8. Implement automatic overdue marking: if invoice status is 'sent' and due_date has passed, update status to 'overdue', write audit log.
9. Create `src/routes/reports.rs`:
   - GET /api/v1/reports/ar-aging — AR aging report
   - GET /api/v1/reports/ar-aging?customer_id=X — per-customer aging
   - GET /api/v1/invoices/:id/reminders — list reminders for an invoice
10. Create `src/routes/financial_reports.rs`:
    - GET /api/v1/reports/revenue?period=monthly&start=2024-01 — revenue summary
    - GET /api/v1/reports/outstanding — total outstanding receivables

## Validation
Unit tests: verify aging bucket calculation with various due dates (current, 45 days, 75 days, 100 days overdue). Integration tests: create invoices with different due dates, generate aging report, verify correct bucketing and totals. Test reminder job: create invoice due in 7 days, run reminder processor, verify reminder record created. Verify duplicate reminders are not created on re-run. Test auto-overdue: create invoice with past due date, run job, verify status changed to 'overdue' with audit log.