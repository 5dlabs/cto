Implement subtask 4006: Implement financial reporting endpoints

## Objective
Build the /api/v1/finance/reports/* endpoints for revenue, expense, and summary financial reports.

## Steps
1. Create `src/handlers/reports.rs` with Axum handlers for report endpoints. 2. GET /api/v1/finance/reports/revenue: query paid invoices within a date range, aggregate by period (daily, weekly, monthly), return totals and breakdowns. Accept query params: start_date, end_date, group_by, currency. 3. GET /api/v1/finance/reports/expenses: query payroll records within a date range, aggregate by period, return totals. 4. GET /api/v1/finance/reports/profit-loss: combine revenue (paid invoices) and expenses (paid payroll) for a period, calculate net profit/loss. 5. GET /api/v1/finance/reports/outstanding: list unpaid/overdue invoices with aging buckets (0-30, 31-60, 61-90, 90+ days). 6. Create `src/repository/report_repo.rs`: implement optimized SQL queries with GROUP BY, date_trunc, and window functions for aggregations. 7. Implement currency normalization: if multi-currency, convert all amounts to a base currency using rates from `finance.currency_rates`. 8. Wire routes into main router.

## Validation
Integration tests: seed invoices and payroll records across multiple periods; revenue report returns correct aggregations by period; profit-loss report correctly subtracts expenses from revenue; outstanding report places invoices in correct aging buckets; currency conversion produces expected normalized values; reports return within acceptable response times (<3 seconds).