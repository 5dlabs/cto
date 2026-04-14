<identity>
You are rex, the Rust/Anchor/Solana implementation agent. You own task 2 end-to-end.
</identity>

<context>
<task_overview>
Task 2: Settlement Engine & Quality-Gated Marketplace Splits (Rex - Rust/Anchor/Solana)
Implement the settle_task and refund_task instructions — the core billing engine with three-case quality-gated payment splitting. This is the program's primary value proposition: billing and marketplace in one instruction.
Priority: high
Dependencies: 1
</task_overview>
</context>

<implementation_plan>
1. Implement `settle_task` instruction in `instructions/settle_task.rs`:
   - Accounts: operator (signer), operator_config, customer_balance, treasury_ata, vault, vault_authority, task_receipt (init), clock (sysvar). Optional: agent_package, author_ata.
   - Args: task_id (String), amount (u64), receipt_hash ([u8; 32]), quality_met (bool).
   - Validation checks (in order):
     a. `!operator_config.paused` → error ProgramPaused
     b. `operator.key() == operator_config.authority` → error Unauthorized
     c. `amount <= customer_balance.max_per_task` → error ExceedsPerTaskCap
     d. Daily cap logic: compute `current_day = clock.unix_timestamp / 86400`. If `current_day > customer_balance.daily_reset_day`, reset `daily_spent = 0` and `daily_reset_day = current_day`. Then check `daily_spent + amount <= max_per_day` → error ExceedsDailyCap (D9).
     e. `customer_balance.balance >= amount` → error InsufficientBalance

   - **Case 1: No agent package** (agent_package account not provided or is a sentinel):
     - Debit `amount` from customer_balance.balance.
     - Transfer `amount` from vault to treasury_ata via PDA signer seeds.
     - Write TaskReceipt: amount, author_earned=0, quality_met (passed through), agent_package=None.

   - **Case 2: Agent package provided, quality_met == true**:
     - Validate agent_package.active == true → error PackageInactive.
     - Compute `author_amount = (amount * agent_package.split_bps) / 10000`.
     - Compute `treasury_amount = amount - author_amount`.
     - Debit `amount` from customer_balance.
     - Transfer `author_amount` from vault to author_ata.
     - Transfer `treasury_amount` from vault to treasury_ata.
     - Update agent_package: total_earned += author_amount, task_count += 1, success_count += 1.
     - Write TaskReceipt: amount, author_earned=author_amount, quality_met=true, agent_package=Some(package_pda).

   - **Case 3: Agent package provided, quality_met == false**:
     - No debit from customer (amount effectively = 0 charged).
     - No transfers.
     - Update agent_package: task_count += 1 (success_count unchanged).
     - Write TaskReceipt: amount=0, author_earned=0, quality_met=false, agent_package=Some(package_pda).

   - In all cases: update customer_balance.daily_spent += actual_charged_amount, customer_balance.total_spent += actual_charged_amount, customer_balance.task_count += 1.
   - Emit `TaskSettled` event: task_id, customer, amount_charged, author_earned, quality_met, agent_package, timestamp.

2. Implement `refund_task` instruction in `instructions/refund_task.rs`:
   - Accounts: operator (signer), operator_config, task_receipt (mut), customer_balance (mut), vault, treasury_ata.
   - Validates operator is authorized and program is not paused.
   - Validates task_receipt.status == Settled → cannot refund already-refunded tasks.
   - Credits task_receipt.amount back to customer_balance.balance (from treasury — 5D Labs absorbs cost if author was paid, per open question #3).
   - Transfer task_receipt.amount from treasury_ata to vault.
   - Sets task_receipt.status = Refunded.
   - Emit `TaskRefunded` event.
   - Add code comment: `// HACKATHON SIMPLIFICATION: Refund comes from treasury. If an author was already paid, 5D Labs absorbs the loss. Post-hackathon: implement clawback or delayed settlement window.`

3. Use Anchor's `Optional<>` or remaining accounts pattern to handle the optional agent_package/author_ata. Consider using two separate instruction handlers (`settle_task_default` and `settle_task_with_package`) if Optional accounts add too much complexity — expose both under `settle_task` via a wrapper or let the client choose the right variant.

4. Add Anchor events:
   - `TaskSettled { task_id, customer, amount_charged, author_earned, quality_met, agent_package, settled_at }`
   - `TaskRefunded { task_id, customer, amount_refunded, refunded_at }`
   - `Deposited { customer, amount, new_balance }`
   - `Withdrawn { customer, amount, new_balance }`
   - `AgentPackageRegistered { package_id, author, split_bps }`

5. Ensure the TaskReceipt PDA's seed uses `hash_seed(&task_id)` — a duplicate task_id will fail at account creation (Anchor's `init` constraint), providing idempotent settlement at the protocol level.
</implementation_plan>

<acceptance_criteria>
Run `anchor build` — zero errors. Write and run targeted unit-level tests for the settle_task instruction covering all three cases:
(1) Settle with no agent package: deposit 100 USDC, settle task for 10 USDC, verify customer balance is 90, treasury received 10, TaskReceipt shows amount=10, author_earned=0.
(2) Settle with agent package, quality_met=true, split_bps=3000 (30%): deposit 100, settle for 10, verify author_ata received 3 USDC, treasury received 7 USDC, customer balance is 90, AgentPackage.total_earned=3, success_count=1.
(3) Settle with agent package, quality_met=false: deposit 100, settle, verify customer balance remains 100, no transfers, TaskReceipt.amount=0, AgentPackage.task_count incremented but success_count unchanged.
(4) Refund: settle a task, then refund it, verify customer balance restored, TaskReceipt.status=Refunded.
(5) Cap enforcement: set max_per_task=5, attempt settle for 10 → ExceedsPerTaskCap error.
(6) Daily cap: set max_per_day=15, settle 10, then settle 10 → ExceedsDailyCap error.
(7) Paused: pause program, attempt settle → ProgramPaused error.
(8) Duplicate task_id: settle task 'CTO-001', attempt second settle with same task_id → error (PDA already exists).
All 8 tests pass on local validator within 30 seconds.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- settle_task instruction scaffolding — account structs, validation chain, and optional accounts pattern: Define the settle_task account structs (handling optional agent_package/author_ata), implement the full validation chain (paused, auth, per-task cap, daily cap with day rollover, balance check), and decide on the optional accounts strategy.
- Settlement Case 1 — no agent package: debit and transfer to treasury: Implement the Case 1 settlement path where no agent package is provided: debit customer balance and transfer full amount from vault to treasury via PDA signing.
- Settlement Cases 2 & 3 — quality-gated split logic with agent package: Implement Case 2 (quality_met=true with split payment to author and treasury) and Case 3 (quality_met=false with zero charge and no transfers) in the settle_task handler.
- Implement refund_task instruction: Implement the refund_task instruction that reverses a settlement by transferring funds from treasury back to vault and crediting the customer balance.
- Define all Anchor events for the program: Define and emit all Anchor event structs: TaskSettled, TaskRefunded, Deposited, Withdrawn, AgentPackageRegistered — used for indexing and off-chain observability.
- Comprehensive 8-case test suite for settlement and refund: Write and execute all 8 test cases covering settle_task (3 cases, caps, pause, duplicate) and refund_task as specified in the test strategy.
</subtasks>