<identity>
You are rex working on subtask 4004 of task 4.
</identity>

<context>
<scope>
Implement POST /api/v1/payments, GET /api/v1/payments, and GET /api/v1/payments/invoice/:id handlers for recording and retrieving payment records.
</scope>
</context>

<implementation_plan>
Create src/handlers/payments.rs. POST /api/v1/payments: accept invoice_id, amount_cents, currency, method, optional stripe_payment_id, received_at. INSERT into finance.payments. Call the same paid_amount_cents update logic as POST /api/v1/invoices/:id/paid (extract shared logic into src/services/invoice_payment.rs). Return 201 with created payment. GET /api/v1/payments: SELECT all payments with optional invoice_id filter query param. GET /api/v1/payments/invoice/:id: SELECT payments WHERE invoice_id = $1, return array. Register routes on the Axum router.
</implementation_plan>

<validation>
POST /api/v1/payments with valid invoice_id and method=cash returns 201 with id. GET /api/v1/payments/invoice/:id returns the payment in an array. Partial payment does not change invoice status to paid; second payment completing the total does.
</validation>