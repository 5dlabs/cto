Implement subtask 4004: Implement invoice status state machine and send endpoint

## Objective
Build the invoice status state machine (draft→sent→viewed→paid→overdue→cancelled) with validation, and implement the POST /api/v1/invoices/:id/send endpoint that marks an invoice as sent.

## Steps
1. Create `src/services/invoice_state.rs` with an InvoiceStateMachine that defines valid transitions:
   - draft → sent, cancelled
   - sent → viewed, paid, overdue, cancelled
   - viewed → paid, overdue, cancelled
   - overdue → paid, cancelled
   - paid → (terminal, no transitions)
   - cancelled → (terminal, no transitions)
2. Implement `fn transition(current: InvoiceStatus, target: InvoiceStatus) -> Result<InvoiceStatus, InvalidTransition>` that validates and returns the new status or an error.
3. Add DB function `update_invoice_status(pool, id, new_status)` that updates status and updated_at, returning the updated invoice. Use a CTE or WHERE clause to ensure the current status allows the transition (optimistic concurrency).
4. Implement `POST /api/v1/invoices/:id/send` handler:
   - Fetch invoice, validate current status is 'draft'.
   - Transition to 'sent', set issued_at = now().
   - Return 200 with updated invoice.
   - (Stripe integration for this endpoint will be added in subtask 4006.)
5. Implement `POST /api/v1/invoices/:id/cancel` handler:
   - Validate invoice is in a cancellable state.
   - Transition to 'cancelled'.
6. Add guards to prevent modification of non-draft invoices (e.g., adding line items to a sent invoice should be rejected).

## Validation
Unit test: state machine allows all valid transitions and rejects invalid ones (e.g., paid→draft returns error). Integration test: create invoice (draft) → send → verify status is 'sent' and issued_at is set. Integration test: attempt to send an already-sent invoice returns 400/409. Integration test: cancel a draft invoice succeeds; cancel a paid invoice returns error.