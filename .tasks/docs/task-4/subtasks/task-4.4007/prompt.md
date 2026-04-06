Implement subtask 4007: Implement payroll processing endpoints

## Objective
Build endpoints for creating, approving, and processing payroll records, with support for hourly rates, deductions, and batch processing.

## Steps
1. Create src/routes/payroll.rs with Axum handlers.
2. POST /v1/payroll: create a payroll record for a crew member and period:
   - Accept crew_member_id, period_start, period_end, hours_worked, rate, deductions
   - Calculate gross_pay = hours_worked * rate
   - Calculate net_pay = gross_pay - sum(deductions)
   - Validate no duplicate payroll for same crew_member + period
   - Return created record with status 'draft'
3. GET /v1/payroll/:id: return payroll record.
4. GET /v1/payroll: list payroll records with filters (crew_member_id, period, status).
5. POST /v1/payroll/:id/approve: transition from draft to approved.
6. POST /v1/payroll/:id/pay: transition from approved to paid, record paid_at timestamp.
7. POST /v1/payroll/batch: create payroll records for multiple crew members at once for a given period.
8. Add validation for state transitions (draft → approved → paid only).

## Validation
POST /v1/payroll creates record with correct gross/net calculations; duplicate payroll for same crew+period returns 409; approval transitions status correctly; paying unapproved payroll returns error; batch creation creates records for all specified crew members; list filtering works correctly by status and period.