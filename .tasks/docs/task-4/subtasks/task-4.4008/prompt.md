Implement subtask 4008: Implement financial reporting endpoints

## Objective
Build reporting endpoints for revenue summaries, outstanding invoices, payment history, and payroll cost reports with date range filtering.

## Steps
1. In src/routes/reports.rs: GET /api/v1/reports/revenue?start_date=&end_date=&currency= — aggregate paid invoice totals by month within date range, optionally convert to target currency using latest rates. 2. GET /api/v1/reports/outstanding — list all invoices with status 'sent' or 'overdue', with total outstanding amount. 3. GET /api/v1/reports/payments?start_date=&end_date= — aggregate payments by payment_method and status within date range. 4. GET /api/v1/reports/payroll?start_date=&end_date= — aggregate payroll costs by currency and status within date range, show approved vs paid totals. 5. All report endpoints return JSON with summary totals and optional breakdown arrays. 6. Use SQL aggregate queries (SUM, GROUP BY) for performance rather than loading all records into memory. 7. Add date validation: start_date must be before end_date, dates must be valid ISO 8601.

## Validation
Seed database with known invoices, payments, and payroll entries across multiple months and currencies; verify revenue report totals match expected sums; outstanding report only includes unpaid invoices; payroll report correctly separates by status; currency conversion in revenue report produces correct results; empty date ranges return zero totals, not errors.