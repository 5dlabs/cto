<identity>
You are rex working on subtask 1002 of task 1.
</identity>

<context>
<scope>
Define all four PDA account structures (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt) with correct seeds, fields, space calculations, and the TaskStatus enum.
</scope>
</context>

<implementation_plan>
1. Create `state/operator_config.rs`: Define `OperatorConfig` account struct with `#[account]` derive. Fields: authority (Pubkey), treasury (Pubkey), mint (Pubkey), protocol_fee_bps (u16), paused (bool). PDA seed: `[b"operator_config"]`. Calculate and document INIT_SPACE using Anchor's space macro or manual calculation (8 discriminator + 32 + 32 + 32 + 2 + 1 = 107 bytes).
2. Create `state/customer_balance.rs`: Define `CustomerBalance` with fields: customer (Pubkey), balance (u64), total_deposited (u64), total_spent (u64), task_count (u64), max_per_task (u64), max_per_day (u64), daily_spent (u64), daily_reset_day (u64), created_at (i64). PDA seed: `[b"customer_balance", customer.key().as_ref()]`. Calculate space.
3. Create `state/agent_package.rs`: Define `AgentPackage` with fields: package_id (String #[max_len(64)]), author (Pubkey), split_bps (u16), source_uri (String #[max_len(256)]), total_earned (u64), task_count (u64), success_count (u64), registered_at (i64), active (bool). PDA seed: `[b"agent_package", hash_seed(&package_id)]`. Calculate space including string prefix lengths.
4. Create `state/task_receipt.rs`: Define `TaskReceipt` with fields: task_id (String #[max_len(64)]), customer (Pubkey), amount (u64), author_earned (u64), quality_met (bool), agent_package (Option<Pubkey>), receipt_hash ([u8; 32]), operator (Pubkey), settled_at (i64), status (TaskStatus). PDA seed: `[b"task_receipt", hash_seed(&task_id)]`.
5. Define `TaskStatus` enum with variants: Settled, Refunded, Disputed. Derive AnchorSerialize, AnchorDeserialize, Clone, PartialEq.
6. Export all structs from `state/mod.rs`.
</implementation_plan>

<validation>
Run `anchor build` — all account structs compile. Verify each struct has correct #[account] derive. Verify space calculations match field sizes. Verify PDA seeds are correctly documented in comments. Check that TaskStatus enum serializes correctly.
</validation>