<identity>
You are rex working on subtask 5005 of task 5.
</identity>

<context>
<scope>
Implement async functions submit_settlement and submit_refund that construct, sign, and send Solana transactions for the settle_task and refund_task instructions to the on-chain cto_billing program.
</scope>
</context>

<implementation_plan>
1. In `crates/controller/src/tasks/code/billing.rs`, implement:
   `async fn submit_settlement(config: &BillingConfig, task: &BillableTask, receipt_hash: [u8; 32]) -> Result<Signature>`
2. Build settle_task instruction:
   - Derive PDAs: operator_config (seeds: [b"operator", operator.key]), customer_balance (seeds: [b"customer", customer.key]), task_receipt (seeds: [b"receipt", task_id bytes]), agent_package (seeds: [b"agent", package_id bytes]), vault (ATA of operator_config PDA for USDC mint).
   - Construct instruction data matching Anchor discriminator + args: task_id (String), amount (u64), receipt_hash ([u8; 32]), quality_met (bool).
   - Include all required accounts: operator (signer), customer_balance, task_receipt (init), agent_package, vault, author_ata, treasury_ata, usdc_mint, token_program, system_program.
3. Sign with operator keypair loaded from BillingConfig.
4. Send via `RpcClient::send_and_confirm_transaction_with_spinner` with commitment `confirmed`.
5. Implement `async fn submit_refund(config: &BillingConfig, task_id: &str, customer_pubkey: &Pubkey) -> Result<Signature>`:
   - Build refund_task instruction with appropriate PDAs and accounts.
   - Sign and send similarly.
6. Error handling: map Solana RPC errors (insufficient funds, account not found, simulation failure) to descriptive error types. Return the error for caller to decide retry.
7. Log transaction signature at info level for observability.
</implementation_plan>

<validation>
Unit test: verify PDA derivation produces expected addresses for known inputs (operator pubkey, customer pubkey, task_id). Verify instruction data serialization matches expected byte layout (compare against TypeScript Anchor client output for same inputs). Integration test against devnet: with a deployed cto_billing program, call submit_settlement with a valid BillableTask, verify returned Signature is confirmable on devnet, and the TaskReceipt PDA account exists with correct data. Call submit_refund and verify the refund transaction succeeds.
</validation>