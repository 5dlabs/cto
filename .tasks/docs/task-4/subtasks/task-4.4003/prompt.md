Implement subtask 4003: Implement Payment model and CRUD endpoints

## Objective
Define the Payment domain model, repository layer, and implement /api/v1/payments endpoints for recording and querying payments.

## Steps
1. Create `src/models/payment.rs`: define Payment struct, CreatePaymentRequest (invoice_id, amount, currency, method), PaymentStatus enum, PaymentMethod enum, PaymentListQuery DTO. 2. Create `src/repository/payment_repo.rs`: implement PaymentRepository — create(req) → Payment, get_by_id(id, tenant_id) → Option<Payment>, list_by_invoice(invoice_id) → Vec<Payment>, list(query) → PaginatedResult<Payment>. 3. Create `src/handlers/payment.rs`: POST /api/v1/payments (record payment — for non-Stripe methods or manual recording), GET /api/v1/payments/:id, GET /api/v1/payments (list with filters). 4. When a payment is recorded and matches the invoice total, transactionally update the invoice status to PAID and set paid_at. 5. Implement partial payment tracking: if payment amount < invoice total, invoice remains in current status; track cumulative payments. 6. Wire routes into main router. 7. Validate that payment amount is positive, currency matches invoice currency, invoice exists and is not already cancelled.

## Validation
Unit tests for payment validation logic; integration tests: record payment updates invoice status to PAID when fully paid; partial payments leave invoice in original status; payment for non-existent invoice returns 404; payment for cancelled invoice returns 422; list payments by invoice returns correct records.