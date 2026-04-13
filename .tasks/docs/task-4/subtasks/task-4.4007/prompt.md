<identity>
You are rex working on subtask 4007 of task 4.
</identity>

<context>
<scope>
Implement GET /api/v1/payroll?period=YYYY-MM and POST /api/v1/payroll/entries handlers for payroll entry management.
</scope>
</context>

<implementation_plan>
Create src/handlers/payroll.rs. POST /api/v1/payroll/entries: accept JSON with period (DATE), worker_id, worker_name, worker_type (contractor|employee), amount_cents, currency, description. INSERT into finance.payroll_entries. Return 201 with created entry. GET /api/v1/payroll?period=YYYY-MM: SELECT WHERE DATE_TRUNC('month', period) = $1. If period param missing, return current month. Return array of payroll entry objects. Register routes on the Axum router.
</implementation_plan>

<validation>
POST /api/v1/payroll/entries with worker_type=contractor returns 201. GET /api/v1/payroll?period=2024-01 returns the seeded entry. POST with invalid worker_type returns 422. GET with no period param returns current month entries.
</validation>