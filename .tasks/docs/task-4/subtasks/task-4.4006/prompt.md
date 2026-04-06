Implement subtask 4006: Implement financial reporting endpoints and AR aging logic

## Objective
Build endpoints for accounts receivable aging reports, revenue reports, and payroll summary reports with configurable date ranges and grouping.

## Steps
1. Create src/routes/reports.rs with Axum handlers.
2. GET /v1/reports/ar-aging:
   - Query invoices with status 'sent' or 'overdue'
   - Group by aging buckets: current (0-30 days), 31-60, 61-90, 91+ days past due
   - Return per-client and aggregate totals with currency
   - Support filtering by client_id
3. GET /v1/reports/revenue:
   - Query paid invoices within a date range
   - Group by month, client, or project (configurable via query param)
   - Return total revenue, payment count, average invoice value
   - Support multi-currency: convert to a base currency using currency service or show per-currency
4. GET /v1/reports/payroll:
   - Query payroll records within a period
   - Return per-crew-member summary: total hours, gross pay, deductions, net pay
   - Return aggregate totals
5. Implement a background check/cron that transitions invoices from 'sent' to 'overdue' when due_date has passed.
6. Wire all report routes into the router.

## Validation
AR aging report correctly buckets invoices by days past due; invoices paid before due date don't appear in AR aging; revenue report totals match sum of paid invoices for the period; payroll report correctly aggregates per-crew-member; overdue transition job changes invoice status when due_date passes; date range filtering works correctly on all reports.