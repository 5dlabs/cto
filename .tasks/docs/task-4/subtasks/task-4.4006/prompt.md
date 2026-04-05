Implement subtask 4006: Implement payroll endpoints

## Objective
Build the payroll management endpoints for creating, reviewing, and processing payroll records linked to crew members and projects.

## Steps
1. Create `src/models/payroll.rs`: PayrollRecord, CreatePayrollRequest, PayrollListQuery (filter by crew_member, period, status), PayrollResponse.
2. Create `src/repositories/payroll_repo.rs`: create_payroll_record, get_payroll_by_id, list_payroll_records, update_payroll_status, calculate_period_totals.
3. Create `src/services/payroll_service.rs`: validate payroll record (hours >= 0, rate > 0), calculate gross_amount = hours × rate, apply standard deductions (configurable percentage), calculate net_amount. Status flow: draft → approved → processed → paid.
4. Create `src/routes/payroll.rs`:
   - POST /api/v1/payroll — create payroll record
   - GET /api/v1/payroll/:id — get payroll record
   - GET /api/v1/payroll — list payroll records with filters
   - PATCH /api/v1/payroll/:id/status — approve/process/mark paid
   - GET /api/v1/payroll/summary?period_start=X&period_end=Y — payroll summary for period
5. Write audit log entries for all payroll state changes.
6. Ensure all monetary calculations use consistent precision (matching the chosen approach from dp-10).

## Validation
Integration tests: create payroll record, verify gross/net calculations. Test status transitions (valid and invalid). List with filters by crew member and period. Verify payroll summary aggregates correctly across multiple records. Verify audit log entries for each status change.