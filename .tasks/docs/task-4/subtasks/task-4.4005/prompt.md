Implement subtask 4005: Implement payroll endpoints

## Objective
Build the /api/v1/payroll endpoints for creating, approving, and managing payroll records.

## Steps
1. Create `src/models/payroll.rs`: define PayrollRecord struct, CreatePayrollRequest (employee_id, period_start, period_end, gross_amount, deductions), PayrollStatus enum, PayrollListQuery DTO. Deductions should be a structured JSONB field (e.g., [{type: 'tax', amount: 500}, {type: 'insurance', amount: 200}]). 2. Create `src/repository/payroll_repo.rs`: implement PayrollRepository — create, get_by_id, list (with filtering by employee, period, status), update_status, calculate_net (gross - sum of deductions). 3. Create `src/handlers/payroll.rs`: POST /api/v1/payroll (create draft), GET /api/v1/payroll/:id, GET /api/v1/payroll (list), PUT /api/v1/payroll/:id/approve (transition DRAFT→APPROVED), PUT /api/v1/payroll/:id/pay (transition APPROVED→PAID, set paid_at). 4. Validate: period_start < period_end, no overlapping periods for same employee, gross_amount > 0, net_amount (gross - deductions) > 0. 5. Wire routes into main router.

## Validation
Unit tests for net amount calculation and period overlap detection; integration tests: create payroll record, approve it, pay it; invalid status transitions (e.g., DRAFT→PAID) return 422; overlapping periods for same employee return 409; deductions are correctly stored and retrieved as structured JSON.