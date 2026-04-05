Implement subtask 4008: Implement financial reporting endpoints

## Objective
Build the four financial report endpoints: revenue by period, aging buckets, cashflow, and project profitability under `/api/v1/finance/reports/`.

## Steps
1. Create `services/rust/finance/src/routes/reports.rs` and `services/rust/finance/src/db/reports.rs`.
2. `GET /api/v1/finance/reports/revenue`:
   - Query params: org_id, period (month/quarter/year), start_date, end_date.
   - SQL: aggregate `total_cents` from invoices where `status = 'paid'` grouped by period.
   - Return array of {period_label, total_revenue_cents, currency, invoice_count}.
3. `GET /api/v1/finance/reports/aging`:
   - Query params: org_id.
   - SQL: select unpaid invoices (status in sent, viewed, overdue), compute days overdue from `due_at`.
   - Bucket into: current (not yet due), 0-30, 31-60, 61-90, 90+ days overdue.
   - Return {buckets: [{range, count, total_cents}], total_outstanding_cents}.
4. `GET /api/v1/finance/reports/cashflow`:
   - Query params: org_id, period, start_date, end_date.
   - SQL: sum payments received (from `payments` table) as inflow, sum payroll_entries as outflow, grouped by period.
   - Return array of {period_label, inflow_cents, outflow_cents, net_cents}.
5. `GET /api/v1/finance/reports/profitability`:
   - Query params: org_id, project_id (optional).
   - SQL: revenue (paid invoices) minus costs (payroll entries) per project_id.
   - Return array of {project_id, revenue_cents, cost_cents, profit_cents, margin_percent}.
6. All reports require org_id for multi-tenancy isolation.
7. Add utoipa annotations for all report endpoints.
8. Wire into Axum router.

## Validation
Integration tests: (1) Revenue: seed 5 paid invoices across 3 months, verify monthly aggregation returns correct totals. (2) Aging: seed invoices with due dates at -10, -40, -70, -100 days and one future due date, verify correct bucketing (current, 0-30, 31-60, 61-90, 90+). (3) Cashflow: seed payments and payroll entries across 2 months, verify inflow/outflow/net per month. (4) Profitability: seed invoices and payroll for 2 projects, verify per-project profit and margin calculation. (5) Verify all reports filter by org_id (create data for 2 orgs, query for 1).