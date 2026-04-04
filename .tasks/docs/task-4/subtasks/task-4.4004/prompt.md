Implement subtask 4004: Implement invoice status state machine

## Objective
Build the invoice status state machine enforcing valid transitions (draft→sent→viewed→paid, draft→sent→overdue, any→cancelled) with rejection of invalid transitions.

## Steps
1. Create `services/rust/finance/src/models/invoice_status.rs`.
2. Define `InvoiceStatus` enum matching the DB enum: Draft, Sent, Viewed, Paid, Overdue, Cancelled.
3. Implement `InvoiceStatus::can_transition_to(&self, target: &InvoiceStatus) -> bool` with allowed transitions:
   - Draft → Sent, Cancelled
   - Sent → Viewed, Paid, Overdue, Cancelled
   - Viewed → Paid, Overdue, Cancelled
   - Overdue → Paid, Cancelled
   - Paid → (none, terminal state)
   - Cancelled → (none, terminal state)
4. Implement `InvoiceStatus::transition(&self, target: InvoiceStatus) -> Result<InvoiceStatus, FinanceError>` that returns the new status or an error with details about the invalid transition.
5. Derive `sqlx::Type`, `Serialize`, `Deserialize`, `utoipa::ToSchema` for the enum.
6. Write comprehensive unit tests for every valid transition and every invalid transition (e.g., Paid→Draft should fail, Cancelled→Sent should fail).

## Validation
Unit tests covering all valid transitions (Draft→Sent, Sent→Viewed, Sent→Paid, Sent→Overdue, Viewed→Paid, Viewed→Overdue, Overdue→Paid, any non-terminal→Cancelled). Unit tests verifying rejection of all invalid transitions (Paid→anything, Cancelled→anything, Draft→Paid, Draft→Overdue, Draft→Viewed). Verify error messages include current and target states.