Implement subtask 4006: Implement payroll and financial reporting endpoints

## Objective
Build REST endpoints for payroll management and financial reporting including revenue summaries, payment reports, and outstanding invoice tracking.

## Steps
1. Create src/db/payroll.rs: repository functions for create_payroll_record, get_payroll, list_payroll (filter by employee_id, period, status), update_payroll_status, approve_payroll, mark_paid.
2. Create src/routes/payroll.rs with handlers:
   - POST /api/v1/payroll → create payroll record (DRAFT)
   - GET /api/v1/payroll → list payroll records (filters: employee_id, period, status)
   - GET /api/v1/payroll/:id → get payroll detail
   - POST /api/v1/payroll/:id/approve → transition DRAFT→APPROVED
   - POST /api/v1/payroll/:id/pay → transition APPROVED→PAID, record paid_at timestamp
3. Create src/services/reports_service.rs with aggregation queries:
   - Revenue summary: total invoiced, total paid, total outstanding by period (month/quarter/year) and currency.
   - Payment report: payments by status, method, date range.
   - Outstanding invoices: overdue invoices with aging (30/60/90 days).
   - Payroll summary: total payroll by period and status.
4. Create src/routes/reports.rs:
   - GET /api/v1/reports/revenue?period=monthly&start=2024-01&end=2024-12
   - GET /api/v1/reports/payments?start=...&end=...
   - GET /api/v1/reports/outstanding
   - GET /api/v1/reports/payroll?period=...
5. All monetary calculations must use Decimal type to avoid floating-point issues.
6. Multi-currency: reports should group by currency; do NOT convert between currencies for aggregation.

## Validation
Payroll status transitions enforce DRAFT→APPROVED→PAID flow; revenue report correctly sums invoiced/paid/outstanding amounts grouped by period and currency; outstanding report correctly calculates aging buckets; all monetary values use Decimal with no floating-point precision loss; payroll list filters work correctly.