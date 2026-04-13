<identity>
You are rex, the Rust, Anchor 0.30.1, SPL Token implementation agent. You own task 4 end-to-end.
</identity>

<context>
<task_overview>
Task 4: Anchor Program — Settlement Engine: settle_task, refund_task, pause, unpause (Rex - Rust/Anchor 0.30+)
Implement the operator-side instructions that form the core settlement engine: settle_task with daily cap enforcement and fee calculation, refund_task for failed CodeRuns, and pause/unpause circuit breaker. This completes the full Anchor program.
Priority: high
Dependencies: 3
</task_overview>
</context>

<implementation_plan>
1. **settle_task** (`instructions/settle_task.rs`):
   - Signer: `operator` (must be `operator_config.authority`).
   - Accounts: `operator` (signer), `operator_config` (read — has_one = authority constraint), `customer_balance` (mut), `task_receipt` (init, PDA seeded by `[TASK_RECEIPT_SEED, &task_id_hash]`), `vault` (mut), `treasury` (mut, token account — validated mint matches `operator_config.mint`), `vault_authority` (PDA signer), `token_program`, `system_program`, `clock` (sysvar for timestamp/slot).
   - Args: `task_id_hash: [u8; 32]`, `amount: u64`, `receipt_hash: [u8; 32]`.
   - Logic (in order):
     a. Check `!operator_config.paused` → `CtoPayError::ProgramPaused`.
     b. Check `amount > 0` → `CtoPayError::ZeroAmount`.
     c. Check `operator.key() == operator_config.authority` (enforced via Anchor `has_one`).
     d. Validate `treasury.mint == operator_config.mint` → `CtoPayError::InvalidMint`.
     e. **Per-task cap**: `amount <= customer_balance.max_per_task` → `CtoPayError::SpendingCapPerTaskExceeded`.
     f. **Daily cap reset logic**:
        ```rust
        let clock = Clock::get()?;
        let current_slot = clock.slot;
        if current_slot >= customer_balance.daily_reset_slot
            .checked_add(SLOTS_PER_DAY)
            .ok_or(CtoPayError::ArithmeticOverflow)? {
            // New day — reset
            customer_balance.daily_spent = 0;
            customer_balance.daily_reset_slot = current_slot;
        }
        ```
     g. **Daily cap check**: `customer_balance.daily_spent.checked_add(amount)? <= customer_balance.max_per_day` → `CtoPayError::SpendingCapDailyExceeded`.
     h. **Balance check**: `customer_balance.balance >= amount` → `CtoPayError::InsufficientBalance`.
     i. **Compute fee**: `fee_amount = amount.checked_mul(u64::from(operator_config.protocol_fee_bps))?.checked_div(10_000)?` — multiply first, then divide (D4 resolution). Fee is informational only for hackathon.
     j. **Transfer**: SPL token CPI from vault → treasury, full `amount`, signed by vault PDA authority.
     k. **Update CustomerBalance**: debit balance, increment total_spent, increment task_count, increment daily_spent.
     l. **Initialize TaskReceipt**: task_id_hash, customer pubkey, amount, fee_amount, receipt_hash, operator pubkey, settled_at from Clock unix_timestamp, status = Settled, bump.

2. **refund_task** (`instructions/refund_task.rs`):
   - Signer: `operator`.
   - Accounts: `operator` (signer), `operator_config` (read — has_one = authority), `customer_balance` (mut), `task_receipt` (mut — validate customer matches customer_balance.customer), `treasury` (mut, token account), `vault` (mut), `vault_authority` (PDA signer — wait, refund goes treasury → vault → customer_balance credit? Or just credit balance?).
   - **Refund accounting model** (simplest correct approach per Open Question 1):
     - For hackathon: credit `customer_balance.balance` by `task_receipt.amount` and mark receipt as Refunded. The USDC physically stays in treasury — the customer's balance is a ledger credit. When the customer withdraws, the program vault must have sufficient funds.
     - **Alternative (safer)**: Transfer USDC from treasury back to vault, then credit balance. This requires the treasury wallet to co-sign or be a PDA. Since the treasury is an operator-controlled wallet (not a PDA), the simplest approach is: the operator sends tokens from treasury back to vault as a separate operation, and `refund_task` only updates ledger state.
     - **Chosen approach**: `refund_task` updates ledger only (credits customer_balance.balance, marks receipt Refunded). The program trusts the operator to maintain vault solvency. Document this accounting model clearly.
   - Logic:
     a. Check `task_receipt.status == TaskStatus::Settled` → `CtoPayError::TaskNotSettled`.
     b. **Refunds work during pause** (Open Question 2 — refunds return funds to customers, same safety rationale as withdrawals).
     c. Credit `customer_balance.balance` by `task_receipt.amount`.
     d. Decrement `customer_balance.total_spent` by `task_receipt.amount`.
     e. Set `task_receipt.status = TaskStatus::Refunded`.
   - No SPL token transfer in this instruction — just ledger update.

3. **pause** (`instructions/pause.rs`):
   - Signer: `operator` (validated as authority).
   - Accounts: `operator` (signer), `operator_config` (mut — has_one = authority).
   - Logic: Set `operator_config.paused = true`.

4. **unpause** (`instructions/unpause.rs`):
   - Signer: `operator` (validated as authority).
   - Accounts: `operator` (signer), `operator_config` (mut — has_one = authority).
   - Logic: Set `operator_config.paused = false`.

5. **Register all 4 instructions** in `lib.rs`.

6. **Final lib.rs** should have 8 instructions total: initialize_operator, create_customer_account, deposit, withdraw, settle_task, refund_task, update_spending_caps, pause, unpause (9 total — pause and unpause are separate).

7. Verify the full program compiles and the IDL includes all 9 instructions with correct account constraints.

8. Add doc comments to every public function and struct explaining the instruction's purpose and security model.
</implementation_plan>

<acceptance_criteria>
1. Run `anchor build` — compiles with zero warnings under pedantic clippy.
2. IDL contains all 9 instructions: initialize_operator, create_customer_account, deposit, withdraw, settle_task, refund_task, update_spending_caps, pause, unpause.
3. Verify `settle_task` IDL args include `task_id_hash` (32-byte array), `amount` (u64), `receipt_hash` (32-byte array).
4. Code review: `settle_task` performs 5 validation checks in order (paused, amount > 0, per-task cap, daily cap with reset, balance).
5. Code review: fee_amount computed as `amount * protocol_fee_bps / 10_000` using checked arithmetic (multiply first).
6. Code review: `refund_task` does NOT check paused state.
7. Code review: `withdraw` (from Task 3) does NOT check paused state.
8. Code review: daily cap reset uses slot comparison with `SLOTS_PER_DAY = 216_000`.
9. Program binary size: `ls -la target/deploy/cto_pay.so` — under 1.4MB.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Implement settle_task instruction with 5-step validation chain, fee computation, and TaskReceipt initialization: Create the settle_task instruction — the most complex instruction in the program — with a strict 5-step validation chain (pause, zero amount, per-task cap, daily cap with slot-based reset, balance check), fee computation using multiply-first-then-divide, SPL CPI transfer from vault to treasury, CustomerBalance state updates, and TaskReceipt PDA initialization.
- Implement refund_task instruction with ledger-only accounting model: Create the refund_task instruction that credits the customer's on-chain balance, decrements total_spent, and marks a TaskReceipt as Refunded. This is a ledger-only operation with no SPL token transfer — the operator is trusted to maintain vault solvency separately.
- Implement pause and unpause circuit breaker instructions: Create the pause and unpause instructions that allow the operator authority to toggle the circuit breaker on OperatorConfig, gating settlement and deposits while preserving customer exit (withdraw) and refund capabilities.
- Register all settlement instructions in lib.rs and verify complete 9-instruction IDL: Wire up settle_task, refund_task, pause, and unpause into lib.rs, bringing the total program instruction count to 9. Verify the full IDL is generated correctly with all instructions, account structures, and argument types.
- Add comprehensive doc comments to all public functions, structs, and security-critical code paths: Add rustdoc comments to every public instruction handler, every Anchor accounts struct, and every state struct explaining purpose, security model, and design rationale — particularly for security-critical decisions like pause bypass on withdraw/refund and the ledger-only refund model.
</subtasks>