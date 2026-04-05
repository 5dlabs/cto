Implement subtask 4006: Implement payroll endpoints and financial reporting

## Objective
Build the payroll management endpoints and financial reporting endpoints including revenue, expense, and payroll summary reports.

## Steps
1. Implement payroll routes in /src/routes/payroll.rs: POST /api/v1/payroll (create payroll record), GET /api/v1/payroll (list with filters: employee, period, status), GET /api/v1/payroll/:id, PUT /api/v1/payroll/:id/approve (approve for payment), PUT /api/v1/payroll/:id/process (mark as processed/paid). 2. Payroll record includes: employee_id, period, gross_amount, deductions (tax, benefits), net_amount, currency. 3. Implement financial reporting endpoints in /src/routes/reports.rs: GET /api/v1/finance/reports/revenue (revenue summary by period with currency breakdown), GET /api/v1/finance/reports/expenses (expense summary), GET /api/v1/finance/reports/payroll-summary (total payroll costs by period), GET /api/v1/finance/reports/profit-loss (revenue minus expenses and payroll). 4. Reports should accept query parameters for date range and currency. Use the currency conversion service for multi-currency aggregation. 5. Implement proper pagination on list endpoints.

## Validation
Payroll CRUD operates correctly with proper status transitions; payroll approval workflow enforces valid state changes; revenue report accurately sums paid invoices by period; payroll summary matches sum of processed payroll records; profit-loss calculation is correct; multi-currency reports convert to requested base currency.