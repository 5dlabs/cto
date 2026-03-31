Implement subtask 4004: Implement payment CRUD endpoints

## Objective
Build the /api/v1/payments endpoints for recording, listing, and managing payments, including linking payments to invoices and updating invoice status on payment completion.

## Steps
1. Create `src/handlers/payments.rs`. 2. POST /api/v1/payments — accepts CreatePaymentRequest with invoice_id (optional), amount, currency, method. For non-Stripe methods, record the payment directly. For Stripe, delegate to Stripe integration (subtask 4005). Validate that payment amount does not exceed invoice remaining balance if invoice_id is provided. 3. GET /api/v1/payments — list with filters: status, method, invoice_id, date range, pagination. 4. GET /api/v1/payments/:id — single payment detail. 5. PUT /api/v1/payments/:id/status — update payment status (for manual methods like bank_transfer, cash). When status becomes 'completed' and invoice_id is set, update invoice status to 'paid' and set paid_date. Use a sqlx transaction for atomicity. 6. Create `src/services/payment_service.rs` with business logic. 7. Implement partial payment tracking: if payment amount < invoice total, invoice remains 'sent'; if sum of completed payments >= total, mark as 'paid'. 8. Register routes under `/api/v1/payments`.

## Validation
Record a manual payment against an invoice and verify invoice status updates to paid. Attempt overpayment and confirm rejection. List payments with filters and verify correct results. Verify partial payment leaves invoice unpaid until fully covered.