## Implement Anchor Program Marketplace — Agent Packages, Settlement & Receipts (Rex - Rust/Anchor)

### Objective
Implement the marketplace and settlement layer: AgentPackage registration/update, the settle_task instruction with quality-gated payment splitting (all three cases from PRD §6.3), refund_task, and TaskReceipt on-chain accounts. This completes the Anchor program with the full instruction set specified in hackathon deliverable #1.

### Ownership
- Agent: rex
- Stack: Rust/Anchor
- Priority: high
- Status: pending
- Dependencies: 2

### Implementation Details
1. Define additional account structures in `programs/cto-billing/src/state/`:
   - `AgentPackage` PDA (seeds: [b"agent_package", package_id.as_bytes()]): package_id (String, max 64), author (Pubkey), split_bps (u16), source_uri (String, max 256), total_earned (u64), task_count (u64), success_count (u64), registered_at (i64), active (bool). Size: 8 + (4+64) + 32 + 2 + (4+256) + 8 + 8 + 8 + 8 + 1 = ~404 bytes.
   - `TaskReceipt` PDA (seeds: [b"task_receipt", task_id.as_bytes()]): task_id (String, max 64), customer (Pubkey), amount (u64), author_earned (u64), quality_met (bool), agent_package (Option<Pubkey>), receipt_hash ([u8; 32]), operator (Pubkey), settled_at (i64), status (enum: Settled=0, Refunded=1, Disputed=2). Size: 8 + (4+64) + 32 + 8 + 8 + 1 + (1+32) + 32 + 32 + 8 + 1 = ~232 bytes.
   - Define `TaskStatus` enum with Borsh serialization.
2. Implement instructions:
   - `register_agent_package.rs`: Author signs. Creates AgentPackage PDA. Validate split_bps <= 10000. Validate package_id length <= 64. Validate source_uri length <= 256. Set registered_at to Clock::get().unix_timestamp. Set active = true, counters to 0.
   - `update_agent_package.rs`: Author signs (must match AgentPackage.author). Can update source_uri, split_bps, active. Same validations.
   - `settle_task.rs`: This is the critical instruction. Operator authority signs. Accounts: OperatorConfig, CustomerBalance, TaskReceipt (init), vault token account, treasury token account, optional AgentPackage, optional author token account.
     - Check !paused.
     - Call daily reset helper on CustomerBalance.
     - **Case 1 (no agent_package provided):** Verify amount <= balance, amount <= max_per_task, daily_spent + amount <= max_per_day. Debit customer balance. Transfer `amount` USDC from vault to treasury. Write TaskReceipt with author_earned=0, quality_met=true (default for internal), agent_package=None.
     - **Case 2 (agent_package provided, quality_met=true):** Same balance/cap checks. Compute author_share = amount * agent_package.split_bps / 10000. platform_share = amount - author_share. Transfer author_share from vault to author's ATA. Transfer platform_share from vault to treasury. Update AgentPackage: total_earned += author_share, task_count += 1, success_count += 1. Write TaskReceipt.
     - **Case 3 (agent_package provided, quality_met=false):** No debit. amount set to 0 in receipt. Update AgentPackage: task_count += 1 (success_count unchanged). Write TaskReceipt with amount=0, author_earned=0, quality_met=false.
     - In all cases: increment CustomerBalance.task_count, update daily_spent (cases 1&2 only), set settled_at.
   - `refund_task.rs`: Operator signs. Load existing TaskReceipt (must be Settled status). Credit TaskReceipt.amount back to CustomerBalance.balance. Transfer USDC from treasury back to vault (or use a refund reserve). Set TaskReceipt.status = Refunded. Decrement total_spent.
3. Add new errors: TaskAlreadySettled, TaskNotSettled, InvalidAgentPackage, AgentPackageInactive, InvalidReceiptHash, PackageIdTooLong, SourceUriTooLong, InvalidSplitBps.
4. Use `anchor-spl` token::transfer_checked for all SPL transfers with proper mint decimals (USDC = 6).
5. Ensure all PDA seeds are deterministic and documented. Use `find_program_address` consistently.
6. Write Rust unit tests for split calculation edge cases: 0 bps, 10000 bps, rounding, overflow protection (use checked_mul/checked_div).

### Subtasks
- [ ] Define AgentPackage PDA, TaskReceipt PDA, and TaskStatus Enum Account Structures: Create the Anchor account structures for AgentPackage and TaskReceipt PDAs, along with the TaskStatus enum with Borsh serialization, in the program's state module.
- [ ] Define Custom Error Variants for Marketplace and Settlement Instructions: Add all new error variants needed by the marketplace instructions to the program's error enum, covering validation failures, state conflicts, and agent package errors.
- [ ] Implement register_agent_package and update_agent_package Instructions: Build the two instructions for AgentPackage lifecycle: register creates a new AgentPackage PDA with author validation, and update allows the author to modify source_uri, split_bps, and active status.
- [ ] Implement settle_task Instruction — Case 1 (No Agent Package, Full Treasury Transfer): Implement the settle_task instruction handler for Case 1: no agent package provided, full amount transfers from vault to treasury, with balance and spending cap enforcement, daily reset logic, and TaskReceipt PDA creation.
- [ ] Implement settle_task Instruction — Cases 2 & 3 (Agent Package with Quality-Gated Payment Splitting): Extend the settle_task instruction to handle Cases 2 and 3: when an agent_package account is provided, perform payment splitting for quality_met=true (Case 2) or zero-charge recording for quality_met=false (Case 3), with overflow-safe arithmetic for the split calculation.
- [ ] Implement refund_task Instruction with Balance Restoration and Status Update: Build the refund_task instruction that reverses a settled task: validates the TaskReceipt is in Settled status, credits the amount back to the customer's balance, transfers USDC from treasury back to vault, and updates the receipt status to Refunded.
- [ ] Rust Unit Tests for Split Calculation Edge Cases and Overflow Protection: Write comprehensive Rust unit tests covering the payment split arithmetic edge cases, overflow protection, rounding behavior, and boundary conditions across all settle_task and refund_task code paths.