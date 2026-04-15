<identity>
You are rex working on subtask 3004 of task 3.
</identity>

<context>
<scope>
Implement the settle_task instruction handler for Case 1: no agent package provided, full amount transfers from vault to treasury, with balance and spending cap enforcement, daily reset logic, and TaskReceipt PDA creation.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/instructions/settle_task.rs`:
   - Define `SettleTask` accounts struct:
     - `operator` (Signer)
     - `operator_config` (seeds=[b"operator_config"], bump, has_one=operator)
     - `customer_balance` (mut, seeds=[b"customer_balance", customer_balance.customer.as_ref()], bump)
     - `task_receipt` (init, payer=operator, space=TaskReceipt::SIZE, seeds=[b"task_receipt", task_id.as_bytes()], bump)
     - `vault` (mut, token account — the program-owned vault from OperatorConfig)
     - `treasury` (mut, token account — the treasury from OperatorConfig)
     - `vault_authority` (PDA signer for vault transfers)
     - `token_program` (Program<Token>)
     - `system_program`
     - Optional remaining accounts for agent package flow (handled in subtask 3005).
   - Instruction args: `task_id: String`, `amount: u64`, `quality_met: bool`, `receipt_hash: [u8; 32]`.
   - Implementation for Case 1 (when no agent_package remaining account is provided):
     a. `require!(!operator_config.paused, ContractPaused)`.
     b. Call the daily_spent reset helper on customer_balance (check if current day != last_reset_day, reset daily_spent to 0).
     c. `require!(amount <= customer_balance.balance, InsufficientBalance)`.
     d. `require!(amount <= operator_config.max_per_task, ExceedsPerTaskCap)`.
     e. `require!(customer_balance.daily_spent.checked_add(amount).ok_or(...)? <= operator_config.max_per_day, ExceedsDailyCap)`.
     f. Debit: `customer_balance.balance -= amount; customer_balance.daily_spent += amount; customer_balance.total_spent += amount; customer_balance.task_count += 1`.
     g. SPL transfer_checked: amount from vault to treasury, using vault_authority PDA as signer, USDC mint decimals = 6.
     h. Write TaskReceipt: task_id, customer=customer_balance.customer, amount, author_earned=0, quality_met=true, agent_package=None, receipt_hash, operator=operator.key(), settled_at=Clock::get()?.unix_timestamp, status=TaskStatus::Settled.
2. Use `anchor_spl::token::transfer_checked` with CpiContext::new_with_signer for the vault PDA authority.
3. Register the instruction in `lib.rs`.
4. Note: The agent_package path (Cases 2 & 3) will be added in subtask 3005 by extending this handler with conditional logic based on remaining accounts.
</implementation_plan>

<validation>
Run `anchor test` — Case 1 settle_task: (1) Customer with 100 USDC balance, settle 25 USDC with no agent package → customer balance = 75, treasury token account increased by 25 USDC, TaskReceipt PDA exists with amount=25, author_earned=0, quality_met=true, agent_package=None, status=Settled. (2) Settle exceeding per-task cap → fails with ExceedsPerTaskCap. (3) Settle exceeding daily cap (two settles in same day that sum over max_per_day) → second fails with ExceedsDailyCap. (4) Settle same task_id twice → second fails because PDA already initialized (TaskAlreadySettled or Anchor init constraint). (5) Settle when paused → fails with ContractPaused.
</validation>