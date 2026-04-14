<identity>
You are rex working on subtask 2001 of task 2.
</identity>

<context>
<scope>
Define the settle_task account structs (handling optional agent_package/author_ata), implement the full validation chain (paused, auth, per-task cap, daily cap with day rollover, balance check), and decide on the optional accounts strategy.
</scope>
</context>

<implementation_plan>
1. Create `instructions/settle_task.rs`.
2. Decide on optional accounts approach: If using two variants, create `SettleTaskDefault` and `SettleTaskWithPackage` account structs. If using Anchor's `Option<Account<>>` (available in v0.30+), create a single `SettleTask` struct with optional fields. Document the choice.
3. Define the core accounts struct:
   - operator: Signer
   - operator_config: Account<OperatorConfig> (PDA)
   - customer_balance: Account<CustomerBalance> (mut, PDA)
   - treasury_ata: Account<TokenAccount> (mut)
   - vault: Account<TokenAccount> (mut, PDA)
   - vault_authority: UncheckedAccount (PDA for signing, seeds=[b"vault", mint.key()])
   - task_receipt: Account<TaskReceipt> (init, PDA seeds=[b"task_receipt", &hash_seed(&task_id)], payer=operator)
   - clock: Sysvar<Clock>
   - mint: Account<Mint>
   - token_program: Program<Token>
   - system_program: Program<System>
   - Optional: agent_package (Account<AgentPackage>, mut), author_ata (Account<TokenAccount>, mut)
4. Define args: task_id (String), amount (u64), receipt_hash ([u8; 32]), quality_met (bool).
5. Implement the validation chain as a helper function `validate_settlement(...)` or inline:
   a. `require!(!operator_config.paused, ProgramPaused)`
   b. `require!(operator.key() == operator_config.authority, Unauthorized)`
   c. `require!(amount <= customer_balance.max_per_task, ExceedsPerTaskCap)`
   d. Daily cap rollover: `let current_day = clock.unix_timestamp as u64 / 86400; if current_day > customer_balance.daily_reset_day { customer_balance.daily_spent = 0; customer_balance.daily_reset_day = current_day; } require!(customer_balance.daily_spent + amount <= customer_balance.max_per_day, ExceedsDailyCap);`
   e. `require!(customer_balance.balance >= amount, InsufficientBalance)`
6. Register instruction(s) in lib.rs.
7. Leave the settlement case logic (token transfers, receipt writing) as TODOs for the next subtasks — this subtask focuses on the account/validation scaffolding.
</implementation_plan>

<validation>
Run `anchor build` — settle_task instruction compiles with placeholder logic. Verify the account struct includes all required accounts. Verify the daily cap rollover logic correctly computes current_day from unix_timestamp. Manual code review of validation ordering matches specification.
</validation>