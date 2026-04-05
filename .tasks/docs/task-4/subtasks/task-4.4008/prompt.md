Implement subtask 4008: Implement financial reporting endpoints (revenue, AR aging, cashflow, profitability)

## Objective
Build the four financial reporting endpoints: revenue aggregation by period, accounts receivable aging buckets, cash flow by period, and per-project profitability.

## Steps
1. Create `src/services/reports.rs` and `src/routes/reports.rs`.
2. `GET /api/v1/finance/reports/revenue?period=monthly|quarterly|yearly&start=&end=`:
   - Query: aggregate total_cents from paid invoices grouped by time period using date_trunc.
   - Return array of { period: string, revenue_cents: i64, currency: string, invoice_count: i64 }.
   - Filter by org_id from auth context.
3. `GET /api/v1/finance/reports/aging`:
   - Query: SELECT invoices WHERE status IN ('sent','viewed','overdue'), compute days_outstanding = now() - due_at.
   - Bucket into: current (not yet due), 1-30 days, 31-60 days, 61-90 days, 90+ days.
   - Return { buckets: [{ label, count, total_cents }], total_outstanding_cents }.
   - Use rust_decimal for all arithmetic.
4. `GET /api/v1/finance/reports/cashflow?period=monthly&start=&end=`:
   - Inflows: SUM(amount_cents) from payments grouped by period.
   - Outflows: SUM(total_cents) from payroll_entries WHERE status='paid' grouped by period.
   - Return array of { period, inflow_cents, outflow_cents, net_cents }.
5. `GET /api/v1/finance/reports/profitability?project_id=`:
   - Revenue: SUM(paid_amount_cents) from invoices for project_id.
   - Costs: (for v1, costs come from payroll entries linked to the project — add optional project_id to payroll_entries or accept it as a limitation).
   - Return { project_id, revenue_cents, cost_cents, profit_cents, margin_pct }.
   - Document that project cost tracking is limited in v1.
6. All queries must be parameterized with org_id for multi-tenancy.
7. Register all four routes under `/api/v1/finance/reports/`.

## Validation
Unit test: AR aging buckets correctly categorize invoices — an invoice due yesterday is in '1-30 days', one due 45 days ago is in '31-60 days', one not yet due is 'current'. Integration test: seed 5 invoices with varying due dates and statuses, call aging endpoint, verify bucket counts and totals match. Integration test: seed paid invoices across 3 months, call revenue endpoint with period=monthly, verify each month's total is correct. Integration test: cashflow endpoint returns correct inflows from payments and outflows from payroll entries.