<identity>
You are rex working on subtask 2004 of task 2.
</identity>

<context>
<scope>
Implement the refund_task instruction that reverses a settlement by transferring funds from treasury back to vault and crediting the customer balance.
</scope>
</context>

<implementation_plan>
1. Create `instructions/refund_task.rs`.
2. Define `RefundTask` accounts struct:
   - operator: Signer
   - operator_config: Account<OperatorConfig> (PDA)
   - task_receipt: Account<TaskReceipt> (mut, PDA)
   - customer_balance: Account<CustomerBalance> (mut, PDA — derive from task_receipt.customer)
   - vault: Account<TokenAccount> (mut, PDA)
   - treasury_ata: Account<TokenAccount> (mut)
   - treasury_authority: Signer (needed to sign the transfer FROM treasury_ata — this is the operator/treasury owner)
   - mint: Account<Mint>
   - token_program: Program<Token>
3. Validation:
   a. `require!(!operator_config.paused, ProgramPaused)`
   b. `require!(operator.key() == operator_config.authority, Unauthorized)`
   c. `require!(task_receipt.status == TaskStatus::Settled, ...)` — cannot refund already-refunded or disputed tasks. Add a custom error variant if needed.
4. Handler:
   a. Credit customer_balance.balance += task_receipt.amount.
   b. Execute SPL token transfer CPI: transfer task_receipt.amount from treasury_ata to vault. This requires the treasury owner to sign (the operator signs as authority).
   c. Set task_receipt.status = TaskStatus::Refunded.
5. Emit TaskRefunded event: { task_id: task_receipt.task_id, customer: task_receipt.customer, amount_refunded: task_receipt.amount, refunded_at: clock.unix_timestamp }.
6. Add code comment: `// HACKATHON SIMPLIFICATION: Refund comes from treasury. If an author was already paid, 5D Labs absorbs the loss. Post-hackathon: implement clawback or delayed settlement window.`
7. Register instruction in lib.rs.
</implementation_plan>

<validation>
Run `anchor build` — compiles. Test: deposit 100 USDC, settle task for 10 USDC (Case 1), verify balance=90. Call refund_task on the TaskReceipt. Verify customer_balance.balance restored to 100, treasury_ata balance decreased by 10, task_receipt.status == Refunded. Attempt second refund on same receipt → error.
</validation>