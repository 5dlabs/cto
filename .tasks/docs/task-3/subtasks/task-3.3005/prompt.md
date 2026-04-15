<identity>
You are rex working on subtask 3005 of task 3.
</identity>

<context>
<scope>
Extend the settle_task instruction to handle Cases 2 and 3: when an agent_package account is provided, perform payment splitting for quality_met=true (Case 2) or zero-charge recording for quality_met=false (Case 3), with overflow-safe arithmetic for the split calculation.
</scope>
</context>

<implementation_plan>
1. Extend the `SettleTask` accounts struct in `settle_task.rs` to accept optional accounts for the agent package flow:
   - `agent_package` (mut, Option) — the AgentPackage PDA.
   - `author_token_account` (mut, Option) — the author's USDC ATA for receiving their split.
   These can be modeled using Anchor's optional account pattern (Option<Account<>>) or remaining accounts with manual deserialization.
2. Implement Case 2 (agent_package provided, quality_met=true):
   a. Validate agent_package is active: `require!(agent_package.active, AgentPackageInactive)`.
   b. Same balance/cap checks as Case 1 (amount, per-task, daily).
   c. Compute split with overflow-safe arithmetic:
      - `let author_share: u64 = (amount as u128).checked_mul(agent_package.split_bps as u128).ok_or(OverflowError)?.checked_div(10000u128).ok_or(OverflowError)? as u64;`
      - `let platform_share: u64 = amount.checked_sub(author_share).ok_or(OverflowError)?;`
   d. SPL transfer_checked: author_share from vault to author_token_account (via vault_authority PDA signer).
   e. SPL transfer_checked: platform_share from vault to treasury.
   f. Debit customer_balance same as Case 1.
   g. Update AgentPackage: `total_earned += author_share; task_count += 1; success_count += 1`.
   h. Write TaskReceipt with agent_package = Some(agent_package.key()), author_earned = author_share.
3. Implement Case 3 (agent_package provided, quality_met=false):
   a. No balance debit, no token transfers.
   b. Update AgentPackage: `task_count += 1` (success_count unchanged).
   c. Write TaskReceipt with amount=0, author_earned=0, quality_met=false, agent_package=Some(agent_package.key()).
   d. Still increment customer_balance.task_count but do NOT update daily_spent or total_spent.
4. Add branching logic in the handler: check if agent_package account is present, then branch on quality_met boolean.
5. Ensure both paths correctly populate all TaskReceipt fields.
</implementation_plan>

<validation>
Run `anchor test` — Case 2: register agent package with split_bps=3000, settle 100 USDC with quality_met=true → customer debited 100, author ATA receives 30 USDC, treasury receives 70 USDC, AgentPackage.total_earned=30, task_count=1, success_count=1, TaskReceipt.author_earned=30. Test split_bps=0 → author gets 0, treasury gets full amount. Test split_bps=10000 → author gets full amount, treasury gets 0. Case 3: settle with quality_met=false → customer balance unchanged, no token movements, TaskReceipt.amount=0, AgentPackage.task_count=1 but success_count=0. Verify rounding: settle 1 lamport (amount=1) with split_bps=3333 → author_share=0 (floor division), platform_share=1. Test with inactive agent package → fails with AgentPackageInactive.
</validation>