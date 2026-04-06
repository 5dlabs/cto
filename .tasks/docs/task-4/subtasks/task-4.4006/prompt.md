Implement subtask 4006: Implement financial report endpoints (revenue, aging, cashflow, profitability)

## Objective
Implement GET /api/v1/finance/reports/revenue, /aging, /cashflow, and /profitability query handlers returning aggregated financial data.

## Steps
Create src/handlers/reports.rs. GET /api/v1/finance/reports/revenue?period=YYYY-MM: SELECT SUM(total_cents) FROM finance.invoices WHERE status='paid' AND DATE_TRUNC('month', issued_at) = $1. GET /api/v1/finance/reports/aging: SELECT COUNT(*), SUM(total_cents - paid_amount_cents) FROM finance.invoices WHERE status IN ('sent','viewed','overdue') GROUP BY CASE WHEN NOW()-issued_at <= 30 THEN '0-30' WHEN NOW()-issued_at <= 60 THEN '31-60' WHEN NOW()-issued_at <= 90 THEN '61-90' ELSE '90+' END. Return JSON with bucket keys and count + amount fields. GET /api/v1/finance/reports/cashflow: monthly in (paid invoices) and out (payroll entries) for the trailing 12 months. GET /api/v1/finance/reports/profitability: per project_id, SUM(paid invoices total_cents) - SUM(payroll_entries amount_cents). All queries use sqlx::query_as with typed result structs.

## Validation
Seed one invoice issued 35 days ago with status=sent; GET /aging shows it in 31-60 bucket with count=1. Seed paid invoices for two months; GET /revenue?period=YYYY-MM returns correct sum for each month. GET /cashflow returns array with at least the current month. GET /profitability returns per-project rows.