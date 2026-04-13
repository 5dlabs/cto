<identity>
You are rex working on subtask 2001 of task 2.
</identity>

<context>
<scope>
Create the state/ module with all three PDA account structs, the TaskStatus enum, INIT_SPACE implementations, and unit tests verifying byte layout calculations.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-pay/src/state/mod.rs` re-exporting all three account modules.
2. Create `state/operator_config.rs`:
   - Define `OperatorConfig` with `#[account]` attribute: `authority: Pubkey`, `treasury: Pubkey`, `mint: Pubkey`, `protocol_fee_bps: u16`, `paused: bool`, `bump: u8`.
   - Implement `Space` trait or use `#[derive(InitSpace)]` to yield 100 bytes (excl. discriminator).
   - Document PDA seeds: `[OPERATOR_CONFIG_SEED]`.
3. Create `state/customer_balance.rs`:
   - Define `CustomerBalance` with `#[account]`: `customer: Pubkey`, `balance: u64`, `total_deposited: u64`, `total_spent: u64`, `task_count: u64`, `max_per_task: u64`, `max_per_day: u64`, `daily_spent: u64`, `daily_reset_slot: u64`, `created_at: i64`, `bump: u8`.
   - INIT_SPACE = 105 bytes. PDA seeds: `[CUSTOMER_BALANCE_SEED, customer.key().as_ref()]`.
4. Create `state/task_receipt.rs`:
   - Define `TaskStatus` enum with `#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]`: `Settled`, `Refunded`, `Disputed`.
   - Define `TaskReceipt` with `#[account]`: `task_id_hash: [u8; 32]`, `customer: Pubkey`, `amount: u64`, `fee_amount: u64`, `receipt_hash: [u8; 32]`, `operator: Pubkey`, `settled_at: i64`, `status: TaskStatus`, `bump: u8`.
   - INIT_SPACE = 154 bytes. PDA seeds: `[TASK_RECEIPT_SEED, &task_id_hash]`.
   - Ensure `task_id_hash` is `[u8; 32]` (SHA-256), NOT a String.
5. Add `#[cfg(test)]` module in each file with unit tests asserting INIT_SPACE matches manual calculation:
   - `assert_eq!(OperatorConfig::INIT_SPACE, 100);`
   - `assert_eq!(CustomerBalance::INIT_SPACE, 105);`
   - `assert_eq!(TaskReceipt::INIT_SPACE, 154);`
</implementation_plan>

<validation>
Run `cargo test` in the program crate — all 3 INIT_SPACE assertion tests pass. Run `anchor build` — IDL contains all 3 account types with correct field names and types. Verify TaskReceipt IDL shows `task_id_hash` as a fixed byte array, not string. Verify TaskStatus enum appears in IDL with 3 variants.
</validation>