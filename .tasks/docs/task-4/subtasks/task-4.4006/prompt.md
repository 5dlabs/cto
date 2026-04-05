Implement subtask 4006: Implement payroll endpoints with multi-currency support

## Objective
Build Axum route handlers for payroll entry creation, approval workflow, and payment recording, with amounts stored and displayed in the crew member's configured currency.

## Steps
1. In src/routes/payroll.rs: POST /api/v1/payroll (create payroll entry: crew_member_id, pay_period_start, pay_period_end, hours_worked, hourly_rate, currency; calculate gross_amount = hours * rate), GET /api/v1/payroll (list with filters: crew_member_id, pay_period, status, page/per_page), GET /api/v1/payroll/:id, PATCH /api/v1/payroll/:id/approve (transition from draft→approved, validate caller has admin role placeholder), PATCH /api/v1/payroll/:id/pay (transition from approved→paid, set paid_date). 2. Add validation: pay_period_end must be after pay_period_start, hours_worked > 0, hourly_rate > 0. 3. Implement currency conversion helper in src/services/currency.rs: given an amount and source/target currencies, look up the latest rate from currency_rates table and return converted amount. 4. Add GET /api/v1/payroll/summary endpoint: aggregate total payroll by currency for a given period. 5. Invalid state transitions (e.g., paying a draft entry) return 409.

## Validation
Integration tests: create payroll entry → approve → mark paid; attempt to pay draft entry returns 409; currency conversion returns correct amount based on stored rates; payroll summary aggregates correctly across multiple entries and currencies; >80% coverage.