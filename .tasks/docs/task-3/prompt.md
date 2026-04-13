<identity>
You are rex, the Rust, Anchor 0.30.1, SPL Token implementation agent. You own task 3 end-to-end.
</identity>

<context>
<task_overview>
Task 3: Anchor Program — Customer Balance Operations: create, deposit, withdraw, update_spending_caps (Rex - Rust/Anchor 0.30+)
Implement the four customer-facing instructions: create_customer_account, deposit, withdraw, and update_spending_caps. These form the customer's self-service interface for managing their on-chain balance and spending limits.
Priority: high
Dependencies: 2
</task_overview>
</context>

<implementation_plan>
1. **create_customer_account** (`instructions/create_customer_account.rs`):
   - Signer: `customer` (the customer wallet).
   - Accounts: `customer` (signer, mut — pays rent), `customer_balance` (init, PDA seeded by `[CUSTOMER_BALANCE_SEED, customer.key().as_ref()]`), `operator_config` (read — to validate program is not paused), `system_program`.
   - Args: `max_per_task: u64`, `max_per_day: u64`.
   - Logic:
     - Check `!operator_config.paused` → `CtoPayError::ProgramPaused`.
     - Initialize `CustomerBalance` with: customer pubkey, balance 0, total_deposited 0, total_spent 0, task_count 0, the provided caps, daily_spent 0, daily_reset_slot set to current slot (from `Clock::get()?`), created_at from `Clock::get()?.unix_timestamp`, bump.
   - Validation: `max_per_task > 0`, `max_per_day > 0`, `max_per_task <= max_per_day`.

2. **deposit** (`instructions/deposit.rs`):
   - Signer: `customer`.
   - Accounts: `customer` (signer), `customer_balance` (mut, PDA — has_one = customer), `customer_token_account` (mut, token account — validated mint matches `operator_config.mint`), `vault` (mut, program vault token account), `operator_config` (read), `token_program`.
   - Args: `amount: u64`.
   - Logic:
     - Check `!operator_config.paused` → `CtoPayError::ProgramPaused`.
     - Check `amount > 0` → `CtoPayError::ZeroAmount`.
     - Validate `customer_token_account.mint == operator_config.mint` → `CtoPayError::InvalidMint`.
     - Execute SPL token transfer: customer_token_account → vault, amount, signed by customer.
     - Update CustomerBalance: `balance = balance.checked_add(amount)?`, `total_deposited = total_deposited.checked_add(amount)?`.
   - Use `anchor_spl::token::Transfer` with CPI.

3. **withdraw** (`instructions/withdraw.rs`):
   - Signer: `customer`.
   - Accounts: `customer` (signer), `customer_balance` (mut, PDA — has_one = customer), `customer_token_account` (mut, validated mint), `vault` (mut), `operator_config` (read), `vault_authority` (PDA signer for CPI), `token_program`.
   - Args: `amount: u64`.
   - Logic:
     - Withdrawals work even when paused (customers can always exit — PRD constraint).
     - Check `amount > 0` → `CtoPayError::ZeroAmount`.
     - Check `customer_balance.balance >= amount` → `CtoPayError::InsufficientBalance`.
     - Validate mint.
     - Execute SPL token transfer: vault → customer_token_account, amount, signed by vault PDA authority (program signer seeds).
     - Update: `balance = balance.checked_sub(amount)?`.
   - **Important**: Withdraw does NOT check paused state. This is a safety valve per PRD.

4. **update_spending_caps** (`instructions/update_spending_caps.rs`):
   - Signer: `customer`.
   - Accounts: `customer` (signer), `customer_balance` (mut, PDA — has_one = customer).
   - Args: `max_per_task: u64`, `max_per_day: u64`.
   - Logic:
     - Validate `max_per_task > 0`, `max_per_day > 0`, `max_per_task <= max_per_day`.
     - Update `customer_balance.max_per_task` and `customer_balance.max_per_day`.
     - Do NOT reset `daily_spent` — cap changes take effect on next settlement but don't clear history.

5. **Vault authority PDA**: The vault's authority is a PDA derived from `[VAULT_SEED, operator_config.key().as_ref()]`. When the program signs CPI transfers from the vault (for withdraw and refund), it uses these seeds plus the bump.

6. **Mint validation pattern** (used in deposit, withdraw, and all settlement instructions):
   ```rust
   #[account(
       constraint = customer_token_account.mint == operator_config.mint @ CtoPayError::InvalidMint
   )]
   pub customer_token_account: Account<'info, TokenAccount>,
   ```

7. **Register all 4 instructions** in `lib.rs` program module.

8. All arithmetic uses checked operations. No raw `+`, `-`, `*`, `/` on u64.
</implementation_plan>

<acceptance_criteria>
1. Run `anchor build` — compiles with zero warnings.
2. IDL contains all 4 new instructions: `create_customer_account`, `deposit`, `withdraw`, `update_spending_caps` with correct args.
3. Verify `create_customer_account` IDL shows `max_per_task` and `max_per_day` parameters.
4. Verify `deposit` and `withdraw` IDL show `amount` parameter.
5. Code review: confirm `withdraw` does NOT check `operator_config.paused`.
6. Code review: all SPL token transfers use `anchor_spl::token::Transfer` CPI, not raw invoke.
7. Code review: every `u64` operation uses `checked_*` with error propagation.
8. Code review: `customer_token_account.mint == operator_config.mint` validation present in deposit and withdraw account structs.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Implement create_customer_account instruction: Create the create_customer_account instruction that initializes a CustomerBalance PDA for a new customer with spending caps, pause validation, and Clock-based timestamps.
- Implement deposit instruction with SPL token CPI transfer: Create the deposit instruction that transfers SPL tokens from the customer's token account to the program vault and credits the customer's on-chain balance using checked arithmetic.
- Implement withdraw instruction with vault PDA authority signing: Create the withdraw instruction that transfers SPL tokens from the program vault back to the customer's token account, signed by the vault PDA authority. Crucially, this instruction does NOT check pause state — it is a safety valve allowing customers to always exit.
- Implement update_spending_caps instruction: Create the update_spending_caps instruction that allows a customer to modify their per-task and daily spending caps without resetting the current daily_spent counter.
- Register all 4 customer instructions in lib.rs and verify build/IDL: Wire up all four customer-facing instructions into the program's lib.rs module, ensure the module exports are correct, run anchor build, and verify the IDL contains all new instructions with correct args and account structures.
</subtasks>