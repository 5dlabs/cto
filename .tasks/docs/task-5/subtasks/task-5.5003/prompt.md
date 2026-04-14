<identity>
You are rex working on subtask 5003 of task 5.
</identity>

<context>
<scope>
Build the transaction construction module that derives all required PDAs using SHA-256 seed logic matching the on-chain program and constructs settle_task and refund_task instructions with the correct account lists.
</scope>
</context>

<implementation_plan>
1. Implement `src/pda.rs` with functions: `derive_operator_config(program_id) -> (Pubkey, u8)`, `derive_customer_balance(program_id, customer: &Pubkey) -> (Pubkey, u8)`, `derive_agent_package(program_id, package_id: &str) -> (Pubkey, u8)`, `derive_task_receipt(program_id, task_id: &str) -> (Pubkey, u8)`, `derive_vault(program_id, mint: &Pubkey) -> (Pubkey, u8)`. Each uses `sha2::Sha256` to hash the relevant seed components, then `Pubkey::find_program_address` with the hash output as seed, exactly matching the on-chain derivation.
2. Implement `src/transaction.rs` with `TransactionBuilder` struct holding program_id, operator keypair reference, and RpcClient reference.
3. Implement `build_settle_task()`: takes SettlementEvent, derives all PDAs (operator_config, customer_balance, vault, task_receipt, and optionally agent_package + author_ata). Construct the instruction data (Anchor discriminator + serialized args: task_id, amount, receipt_hash, quality_met). Build AccountMeta list: operator (signer, writable), operator_config (read), customer_balance (writable), vault (writable), treasury_ata (writable), task_receipt (writable, init), system_program, token_program, clock sysvar. If agent_package present: add agent_package (writable) and author_ata (writable).
4. Implement `build_refund_task()`: similar pattern with refund-specific accounts.
5. Implement blockhash management: `BlockhashCache` struct with `get_blockhash()` that returns cached value if < 30 seconds old, otherwise fetches fresh via `RpcClient::get_latest_blockhash()`. Expose `force_refresh()` for BlockhashNotFound recovery.
6. Assemble full Transaction with the instruction, recent blockhash, and sign with operator keypair.
</implementation_plan>

<validation>
Unit tests: (1) Derive operator_config, customer_balance, agent_package, task_receipt, and vault PDAs from known inputs (hardcoded program_id, customer pubkey, package_id, task_id, mint) — verify output matches expected Pubkeys computed independently (cross-reference with TypeScript PDA derivation from Task 3 or from anchor test fixtures). (2) Verify settle_task instruction data serialization: construct instruction, check discriminator bytes and serialized args match expected byte layout. (3) Verify account list for settle_task with agent_package includes exactly the expected accounts in correct order with correct is_signer/is_writable flags. (4) Verify account list for settle_task without agent_package omits agent_package and author_ata accounts.
</validation>