Implement subtask 4012: Implement GDPR anonymization endpoint

## Objective
Build the GDPR endpoint `DELETE /api/v1/gdpr/customer/:id` that anonymizes customer-identifying data on invoices while preserving financial records for legal compliance.

## Steps
1. Create `services/rust/finance/src/routes/gdpr.rs`.
2. `DELETE /api/v1/gdpr/customer/:id`:
   - The `:id` parameter represents a customer identifier (could be email or customer_id depending on how customers are referenced).
   - Find all invoices where `customer_email = :id` or a customer reference field matches.
   - Update: set `customer_name = 'DELETED'`, `customer_email = 'DELETED'`, and any other PII fields to 'DELETED'.
   - Do NOT delete the invoice records — financial totals (subtotal_cents, tax_cents, total_cents, paid_amount_cents) must be preserved for accounting/legal compliance.
   - Do NOT delete payment records.
   - Return 200 with `{anonymized_invoices: count, message: "Customer data anonymized"}`.
   - If no invoices found for the customer, return 200 with `{anonymized_invoices: 0}`.
3. Log the anonymization event at info level with the number of affected records (but NOT the customer data being deleted).
4. Add utoipa annotation.
5. Wire into Axum router.

## Validation
Integration test: (1) Create 3 invoices with customer_email='test@example.com' (one paid, one sent, one draft) and associated payments. (2) Call DELETE /api/v1/gdpr/customer/test@example.com. (3) Verify all 3 invoices have customer_name='DELETED' and customer_email='DELETED'. (4) Verify financial fields (total_cents, paid_amount_cents) are unchanged. (5) Verify payment records are untouched. (6) Call DELETE again for same customer, verify returns 0 affected (idempotent). (7) Call for non-existent customer, verify 200 with 0 count.