<identity>
You are rex working on subtask 3001 of task 3.
</identity>

<context>
<scope>
Create the Anchor account structures for AgentPackage and TaskReceipt PDAs, along with the TaskStatus enum with Borsh serialization, in the program's state module.
</scope>
</context>

<implementation_plan>
1. In `programs/cto-billing/src/state/`, create `agent_package.rs` defining the `AgentPackage` account struct with fields: package_id (String, max 64), author (Pubkey), split_bps (u16), source_uri (String, max 256), total_earned (u64), task_count (u64), success_count (u64), registered_at (i64), active (bool). Add `#[account]` derive macro. Define SEED_PREFIX as `b"agent_package"`. Calculate and document the account size as a const: 8 (discriminator) + (4+64) + 32 + 2 + (4+256) + 8 + 8 + 8 + 8 + 1 = 403 bytes, round up to 408 for alignment safety.
2. Create `task_receipt.rs` defining the `TaskReceipt` account struct with fields: task_id (String, max 64), customer (Pubkey), amount (u64), author_earned (u64), quality_met (bool), agent_package (Option<Pubkey>), receipt_hash ([u8; 32]), operator (Pubkey), settled_at (i64), status (TaskStatus). Size: 8 + (4+64) + 32 + 8 + 8 + 1 + (1+32) + 32 + 32 + 8 + 1 = 231 bytes, round to 240.
3. Define `TaskStatus` enum in `task_receipt.rs` with variants: Settled = 0, Refunded = 1, Disputed = 2. Derive `AnchorSerialize`, `AnchorDeserialize`, `Clone`, `PartialEq`, `Eq`. Implement Default as Settled.
4. Export both modules from `state/mod.rs`. Ensure PDA seed constants are public and documented with comments specifying the seed pattern.
5. Run `anchor build` to verify the new account types compile and the discriminators are generated.
</implementation_plan>

<validation>
Run `anchor build` — compiles without errors. Verify the IDL JSON includes AgentPackage and TaskReceipt account types with all specified fields and correct types. Verify TaskStatus enum appears in the IDL with three variants. Confirm account size constants match hand-calculated values by writing a Rust compile-time assertion (`const_assert!`).
</validation>