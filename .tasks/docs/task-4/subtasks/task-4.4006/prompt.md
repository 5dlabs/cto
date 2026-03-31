Implement subtask 4006: Implement payroll endpoints

## Objective
Build the /api/v1/payroll endpoints for creating, listing, and managing payroll records.

## Steps
1. Create `src/handlers/payroll.rs`. 2. POST /api/v1/payroll — accepts CreatePayrollRequest with employee_id, period_start, period_end, gross_amount, deductions, currency, frequency. Calculates net_amount = gross_amount - deductions. Validate that period_start < period_end. Check for overlapping payroll periods for the same employee. 3. GET /api/v1/payroll — list with filters: employee_id, frequency, status, date range (period overlapping), pagination. 4. GET /api/v1/payroll/:id — single payroll record. 5. PUT /api/v1/payroll/:id — update mutable fields for pending payroll records only. 6. POST /api/v1/payroll/:id/process — transition from pending to processing/completed. Set paid_at timestamp. 7. GET /api/v1/payroll/summary — aggregate endpoint returning total payroll by period, by frequency, by currency. 8. Create `src/services/payroll_service.rs`. 9. Register routes under `/api/v1/payroll`.

## Validation
Create a payroll record and verify net_amount is correctly calculated. Attempt to create overlapping payroll period for same employee and verify rejection. Process a payroll record and verify status transition. List payroll with filters and verify correct pagination. Payroll summary returns correct aggregates.