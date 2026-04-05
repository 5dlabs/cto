Implement subtask 4009: Implement payroll endpoints for entry management and period summaries

## Objective
Build payroll entry creation and period-based summary endpoints including support for employee and contractor types.

## Steps
1. Define structs in `src/models/payroll.rs`: PayrollEntry, CreatePayrollEntryRequest (employee_id, period_start, period_end, type, hours, rate_cents, currency), PayrollSummary.
2. Implement `src/db/payroll.rs`:
   - `create_payroll_entry(pool, req)` — compute total_cents = hours * rate_cents (using rust_decimal for precise multiplication), INSERT and return.
   - `list_payroll_entries(pool, period_start, period_end, org_id)` — entries within a date range.
   - `get_payroll_summary(pool, period_start, period_end)` — aggregate: total entries, total_cents by type (employee vs contractor), headcount.
3. Implement Axum handlers in `src/routes/payroll.rs`:
   - `POST /api/v1/payroll/entries` → create entry, return 201.
   - `GET /api/v1/payroll?period_start=&period_end=` → return payroll summary + list of entries for the period.
4. Validation: period_end must be after period_start, hours must be positive, rate_cents must be positive, type must be 'employee' or 'contractor'.
5. total_cents computation: use rust_decimal to multiply hours (Decimal) by rate_cents (converted to Decimal), then convert result to i64 cents.
6. Register routes on the router.

## Validation
Integration test: POST /api/v1/payroll/entries with valid data returns 201, total_cents is correctly computed (e.g., 40 hours * 5000 cents/hr = 200000 cents). Integration test: GET /api/v1/payroll with period returns summary with correct aggregation. Unit test: total_cents calculation handles fractional hours correctly (e.g., 37.5 hours * 4250 cents = 159375 cents).