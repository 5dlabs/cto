<identity>
You are rex working on subtask 2002 of task 2.
</identity>

<context>
<scope>
Implement the Case 1 settlement path where no agent package is provided: debit customer balance and transfer full amount from vault to treasury via PDA signing.
</scope>
</context>

<implementation_plan>
1. In the settle_task handler (or settle_task_default variant), implement Case 1 logic:
2. Debit customer_balance.balance -= amount.
3. Execute SPL token transfer CPI: transfer `amount` from vault to treasury_ata. Use PDA signer seeds: `&[b"vault", mint.key().as_ref(), &[vault_bump]]`. Derive or pass the vault bump.
4. Write TaskReceipt fields:
   - task_id = args.task_id
   - customer = customer_balance.customer
   - amount = args.amount
   - author_earned = 0
   - quality_met = args.quality_met (passed through — in Case 1 this is informational)
   - agent_package = None
   - receipt_hash = args.receipt_hash
   - operator = operator.key()
   - settled_at = clock.unix_timestamp
   - status = TaskStatus::Settled
5. Update customer counters: daily_spent += amount, total_spent += amount, task_count += 1.
6. This provides the complete working path for the simplest settlement case.
</implementation_plan>

<validation>
Run `anchor build` — compiles. Write a focused test: deposit 100 USDC, call settle_task with amount=10, no agent package. Verify customer_balance.balance == 90, treasury_ata balance increased by 10, TaskReceipt shows amount=10 and author_earned=0 and status=Settled.
</validation>