<identity>
You are rex, the Rust/Anchor implementation agent. You own task 2 end-to-end.
</identity>

<context>
<task_overview>
Task 2: Implement Anchor Program Core — Operator, Customer & Escrow Instructions (Rex - Rust/Anchor)
Implement the foundational Anchor program accounts (OperatorConfig, CustomerBalance) and instructions (initialize_operator, create_customer_account, deposit, withdraw, update_spending_caps, pause/unpause). This establishes the escrow-based billing primitive with USDC escrow vault, spending caps with daily reset logic, and circuit breaker — the foundation that all settlement logic depends on.
Priority: high
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Define account structures in `programs/cto-billing/src/state/mod.rs`:
   - `OperatorConfig` PDA (seeds: [b"operator_config"]): authority (Pubkey), treasury (Pubkey), protocol_fee_bps (u16), paused (bool). Size: 8 + 32 + 32 + 2 + 1 + padding = ~80 bytes.
   - `CustomerBalance` PDA (seeds: [b"customer_balance", customer.key().as_ref()]): customer (Pubkey), balance (u64), total_deposited (u64), total_spent (u64), task_count (u64), max_per_task (u64), max_per_day (u64), daily_spent (u64), daily_reset_slot (u64), created_at (i64). Size: 8 + 32 + (8 * 8) + 8 = ~112 bytes.
   - `Vault` — Associated token account for USDC, owned by the program PDA (seeds: [b"vault"]).
2. Implement instructions in `programs/cto-billing/src/instructions/`:
   - `initialize_operator.rs`: Creates OperatorConfig. Signer becomes authority. Validates treasury is not zero address. Sets protocol_fee_bps (max 10000). Sets paused = false.
   - `create_customer_account.rs`: Customer signs. Creates CustomerBalance PDA. Sets max_per_task and max_per_day from args. Sets daily_reset_slot to current slot.
   - `deposit.rs`: Customer transfers USDC from their ATA to the program vault via `transfer_checked`. Increments CustomerBalance.balance and total_deposited. Require !paused.
   - `withdraw.rs`: Customer requests withdrawal. Program vault transfers USDC back to customer ATA. Decrements balance. Require balance >= amount. Require !paused.
   - `update_spending_caps.rs`: Customer signs. Updates max_per_task and max_per_day. Validate max_per_task <= max_per_day.
   - `pause.rs` / `unpause.rs`: Operator authority signs. Toggles OperatorConfig.paused. All deposit/withdraw/settle instructions check `require!(!config.paused, BillingError::ProgramPaused)`.
3. Implement daily spending cap reset logic as a helper function: if current_slot > daily_reset_slot + SLOTS_PER_DAY (~216,000 slots for ~1 day), reset daily_spent to 0 and update daily_reset_slot.
4. Define custom errors in `programs/cto-billing/src/errors.rs`: InsufficientBalance, ExceedsPerTaskCap, ExceedsDailyCap, ProgramPaused, Unauthorized, InvalidTreasury, InvalidAmount, InvalidCaps.
5. Export IDL: ensure `anchor build` generates the IDL JSON in `target/idl/cto_billing.json`.
6. Write unit-level Rust tests in `programs/cto-billing/src/tests/` for:
   - Operator init with valid/invalid params.
   - Customer account creation.
   - Deposit increments balance correctly.
   - Withdraw decrements balance, fails on insufficient.
   - Spending cap enforcement.
   - Pause blocks operations.
   - Daily reset logic.
</implementation_plan>

<acceptance_criteria>
Run `anchor build` — program compiles with zero warnings (deny warnings). Run `cargo test --manifest-path programs/cto-billing/Cargo.toml` — all Rust unit tests pass. Run `anchor test` with TypeScript integration tests that: (1) Initialize operator — OperatorConfig PDA exists with correct authority and treasury. (2) Create customer account — CustomerBalance PDA exists with correct caps. (3) Deposit 1000 USDC — customer balance reads 1000, vault token balance reads
1000. (4) Withdraw 500 — customer balance reads 500, customer ATA increased by
500. (5) Withdraw 600 — fails with InsufficientBalance. (6) Pause — deposit fails with ProgramPaused. (7) Unpause — deposit succeeds again. (8) Update caps — new caps reflected in account. IDL file exists at `target/idl/cto_billing.json` and contains all 6 instruction definitions.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Define account state structures — OperatorConfig, CustomerBalance, and Vault PDA: Implement the core account data structures in programs/cto-billing/src/state/mod.rs with proper sizing, PDA seeds, and Anchor account serialization.
- Define custom error enum for all billing program errors: Create the BillingError enum in programs/cto-billing/src/errors.rs with all error variants needed by instructions.
- Implement daily spending cap reset helper function: Implement the slot-based daily spending cap reset logic as a standalone helper function that can be called from deposit, withdraw, and future settle instructions.
- Implement operator instructions — initialize_operator, pause, and unpause: Implement the three operator authority instructions: initialize_operator to create the OperatorConfig PDA, and pause/unpause to toggle the circuit breaker.
- Implement customer account management — create_customer_account and update_spending_caps: Implement instructions for customers to create their balance PDA and update their spending cap parameters.
- Implement escrow token operations — deposit and withdraw with SPL transfer_checked: Implement deposit (customer ATA → program vault) and withdraw (program vault → customer ATA) instructions with SPL token transfer_checked, pause checks, and balance tracking.
- Write comprehensive Rust unit tests and verify IDL generation: Write Rust unit tests for all instruction paths, edge cases, helper functions, and verify the generated IDL contains all instruction definitions.
</subtasks>