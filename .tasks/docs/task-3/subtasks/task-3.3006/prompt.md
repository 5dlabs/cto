<identity>
You are rex working on subtask 3006 of task 3.
</identity>

<context>
<scope>
Build the refund_task instruction that reverses a settled task: validates the TaskReceipt is in Settled status, credits the amount back to the customer's balance, transfers USDC from treasury back to vault, and updates the receipt status to Refunded.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/instructions/refund_task.rs`:
   - Define `RefundTask` accounts struct:
     - `operator` (Signer)
     - `operator_config` (seeds=[b"operator_config"], bump, has_one=operator)
     - `customer_balance` (mut, seeds=[b"customer_balance", customer_balance.customer.as_ref()], bump)
     - `task_receipt` (mut, seeds=[b"task_receipt", task_receipt.task_id.as_bytes()], bump)
     - `vault` (mut, token account)
     - `treasury` (mut, token account)
     - `treasury_authority` (PDA signer for treasury transfers — define the PDA seeds, e.g., [b"treasury_authority"])
     - `token_program`
   - No instruction args (all data comes from the existing TaskReceipt).
   - Implementation:
     a. `require!(!operator_config.paused, ContractPaused)`.
     b. `require!(task_receipt.status == TaskStatus::Settled, TaskNotSettled)`.
     c. `require!(task_receipt.customer == customer_balance.customer, InvalidCustomer)` — verify receipt belongs to this customer.
     d. Credit customer: `customer_balance.balance += task_receipt.amount; customer_balance.total_spent -= task_receipt.amount`.
     e. SPL transfer_checked: task_receipt.amount from treasury to vault, using treasury_authority PDA as signer.
     f. Set `task_receipt.status = TaskStatus::Refunded`.
   - Note: This design transfers from treasury back to vault. If author_earned > 0 (Case 2 settle), the refund only restores the customer — the author/platform split handling for refunds is a future concern (document this limitation with a TODO comment).
2. Register the instruction in `lib.rs` and export from `instructions/mod.rs`.
3. Ensure the treasury_authority PDA is properly defined and its seeds are consistent with how the treasury account is set up in OperatorConfig initialization.
</implementation_plan>

<validation>
Run `anchor test` — (1) Settle a task for 50 USDC (Case 1), then refund: customer balance restored by 50, total_spent decremented, treasury sends 50 USDC back to vault, TaskReceipt.status = Refunded. (2) Attempt to refund a task that was never settled → fails with TaskNotSettled. (3) Attempt to refund the same task twice → first succeeds, second fails with TaskNotSettled (status is now Refunded). (4) Refund when contract is paused → fails with ContractPaused. (5) Verify customer_balance.customer matches task_receipt.customer — mismatched customer fails.
</validation>