<identity>
You are rex working on subtask 4002 of task 4.
</identity>

<context>
<scope>
Create the refund_task instruction that credits the customer's on-chain balance, decrements total_spent, and marks a TaskReceipt as Refunded. This is a ledger-only operation with no SPL token transfer — the operator is trusted to maintain vault solvency separately.
</scope>
</context>

<implementation_plan>
Create `instructions/refund_task.rs`. Define the `RefundTask` Anchor accounts struct with: `operator` (Signer), `operator_config` (has_one = authority), `customer_balance` (mut), `task_receipt` (mut, constraint `task_receipt.customer == customer_balance.customer @ CtoPayError::InvalidCustomer` — ensures refund goes to correct customer). Handler takes no additional args (all data comes from the TaskReceipt). Logic: (1) Do NOT check `operator_config.paused` — refunds work during pause (same safety rationale as withdrawals: returning funds to customers). Add a code comment explaining this. (2) Check `task_receipt.status == TaskStatus::Settled` → TaskNotSettled. (3) Credit: `customer_balance.balance = customer_balance.balance.checked_add(task_receipt.amount).ok_or(ArithmeticOverflow)?`. (4) Decrement: `customer_balance.total_spent = customer_balance.total_spent.checked_sub(task_receipt.amount).ok_or(ArithmeticOverflow)?`. (5) Set `task_receipt.status = TaskStatus::Refunded`. Add detailed doc comment explaining the ledger-only accounting model: no SPL transfer occurs, the operator must independently replenish the vault if needed to cover subsequent withdrawals. Export from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles. IDL contains `refund_task` instruction. Code review: NO pause check present — add comment explaining why. Code review: status guard checks `TaskStatus::Settled` before allowing refund. Code review: customer_balance.balance is credited by task_receipt.amount using checked_add. Code review: total_spent is decremented using checked_sub. Code review: no SPL token transfer CPI exists in this instruction. Code review: constraint validates task_receipt.customer == customer_balance.customer.
</validation>