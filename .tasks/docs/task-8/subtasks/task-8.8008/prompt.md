<identity>
You are tess working on subtask 8008 of task 8.
</identity>

<context>
<scope>
Write the comprehensive E2E test that replicates the exact hackathon CLI demo flow as a single automated test: initialize operator, register agent package, create customer, deposit, settle with quality met, settle with quality not met, verify all receipts and state, then withdraw.
</scope>
</context>

<implementation_plan>
1. Create `tests/07-e2e-demo-flow.test.ts` with describe block 'E2E Demo Flow'.
2. Single comprehensive test: 'replicates full hackathon demo flow' — 
   a. **Step 1 - Initialize:** Call initialize_operator with protocol_fee_bps=500, set treasury. Assert operator PDA exists.
   b. **Step 2 - Register Agent:** Call register_package with package_id='demo-agent', split_bps=3000, source_uri. Assert AgentPackage PDA created, task_count=0.
   c. **Step 3 - Create Customer:** Call create_customer with max_per_task=200_000_000, max_per_day=1_000_000_000. Assert CustomerBalance PDA created.
   d. **Step 4 - Deposit:** Deposit 100 USDC. Assert balance=100_000_000.
   e. **Step 5 - Settle (quality met):** Settle task_id='demo-task-1', amount=50_000_000, quality_met=true, agent_package='demo-agent'. Assert: customer debited, author and treasury credited with correct split, TaskReceipt created with quality_met=true.
   f. **Step 6 - Settle (quality not met):** Settle task_id='demo-task-2', quality_met=false. Assert: customer NOT debited, TaskReceipt created with amount=0, quality_met=false.
   g. **Step 7 - Verify Receipts:** Fetch both TaskReceipt PDAs. Assert demo-task-1 has correct amount and quality_met=true. Assert demo-task-2 has amount=0 and quality_met=false.
   h. **Step 8 - Verify Agent Stats:** Fetch AgentPackage. Assert task_count=2, success_count=1, total_earned matches demo-task-1 author split.
   i. **Step 9 - Withdraw:** Withdraw remaining balance. Assert customer balance=0, customer ATA received correct amount.
   j. **Final assertions:** Customer final balance=0, AgentPackage.task_count=2, AgentPackage.success_count=1, two TaskReceipt PDAs exist.
3. Use descriptive console.log statements at each step (matching demo script output) so test output reads like a demo walkthrough.
</implementation_plan>

<validation>
Run `anchor test --skip-build -- --grep 'E2E Demo Flow'` — the single comprehensive test passes. Final state assertions: customer balance=0 after full withdraw, AgentPackage.task_count=2, AgentPackage.success_count=1, two TaskReceipt PDAs exist with correct quality_met flags. Test completes within 30 seconds. Console output reads as a clear step-by-step demo narrative.
</validation>