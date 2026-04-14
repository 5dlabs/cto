<identity>
You are rex working on subtask 2003 of task 2.
</identity>

<context>
<scope>
Implement Case 2 (quality_met=true with split payment to author and treasury) and Case 3 (quality_met=false with zero charge and no transfers) in the settle_task handler.
</scope>
</context>

<implementation_plan>
1. In the settle_task handler (or settle_task_with_package variant), implement Case 2 (quality_met == true):
   a. Validate agent_package.active == true → error PackageInactive.
   b. Compute author_amount = (amount * agent_package.split_bps as u64) / 10000.
   c. Compute treasury_amount = amount - author_amount.
   d. Debit customer_balance.balance -= amount.
   e. SPL token transfer CPI: transfer `author_amount` from vault to author_ata using PDA signer seeds.
   f. SPL token transfer CPI: transfer `treasury_amount` from vault to treasury_ata using PDA signer seeds.
   g. Update agent_package: total_earned += author_amount, task_count += 1, success_count += 1.
   h. Write TaskReceipt: amount=amount, author_earned=author_amount, quality_met=true, agent_package=Some(agent_package.key()).
   i. Update customer counters: daily_spent += amount, total_spent += amount, task_count += 1.

2. Implement Case 3 (quality_met == false):
   a. No debit from customer balance (charged amount = 0).
   b. No token transfers.
   c. Update agent_package: task_count += 1 (success_count unchanged).
   d. Write TaskReceipt: amount=0, author_earned=0, quality_met=false, agent_package=Some(agent_package.key()), receipt_hash=args.receipt_hash, status=Settled.
   e. Update customer counters: daily_spent += 0, total_spent += 0, task_count += 1.

3. Handle the branching: if agent_package is provided (Some), check quality_met to branch between Case 2 and Case 3. If agent_package is None, use Case 1 logic (already implemented).

4. Ensure the vault PDA signer seeds are reused correctly across both transfer CPIs in Case 2.
</implementation_plan>

<validation>
Run `anchor build` — compiles. Test Case 2: deposit 100 USDC, register agent package with split_bps=3000, settle 10 USDC with quality_met=true. Verify author_ata received 3, treasury received 7, customer balance is 90, AgentPackage.total_earned=3, success_count=1. Test Case 3: deposit 100 USDC, settle with quality_met=false. Verify customer balance remains 100, no transfers, TaskReceipt.amount=0, AgentPackage.task_count incremented but success_count unchanged.
</validation>