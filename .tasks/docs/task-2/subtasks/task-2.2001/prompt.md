<identity>
You are rex working on subtask 2001 of task 2.
</identity>

<context>
<scope>
Implement the core account data structures in programs/cto-billing/src/state/mod.rs with proper sizing, PDA seeds, and Anchor account serialization.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/state/mod.rs` with pub mod exports. 2. Define `OperatorConfig` struct with `#[account]` derive: authority (Pubkey), treasury (Pubkey), protocol_fee_bps (u16), paused (bool). Add `SEED_PREFIX` const = b"operator_config". Calculate and document space = 8 (discriminator) + 32 + 32 + 2 + 1 + 5 (padding) = 80 bytes. 3. Define `CustomerBalance` struct with `#[account]` derive: customer (Pubkey), balance (u64), total_deposited (u64), total_spent (u64), task_count (u64), max_per_task (u64), max_per_day (u64), daily_spent (u64), daily_reset_slot (u64), created_at (i64). Add `SEED_PREFIX` const = b"customer_balance". Calculate space = 8 + 32 + (8 * 9) = 112 bytes. 4. Document vault PDA seeds [b"vault"] as a constant for the program-owned associated token account. 5. Add `impl` blocks with helper methods: `CustomerBalance::has_sufficient_balance(amount: u64) -> bool`, `OperatorConfig::is_active() -> bool`. 6. Update `programs/cto-billing/src/lib.rs` to include state module.
</implementation_plan>

<validation>
Module compiles without errors via `anchor build`. Account sizes match documented calculations. PDA seed constants are exported and accessible. Helper methods return correct values for edge cases (zero balance, paused state).
</validation>