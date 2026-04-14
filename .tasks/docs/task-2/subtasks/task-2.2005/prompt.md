<identity>
You are rex working on subtask 2005 of task 2.
</identity>

<context>
<scope>
Define and emit all Anchor event structs: TaskSettled, TaskRefunded, Deposited, Withdrawn, AgentPackageRegistered — used for indexing and off-chain observability.
</scope>
</context>

<implementation_plan>
1. Create `events.rs` (or add to an existing events module).
2. Define Anchor event structs using `#[event]` macro:
   - `TaskSettled { task_id: String, customer: Pubkey, amount_charged: u64, author_earned: u64, quality_met: bool, agent_package: Option<Pubkey>, settled_at: i64 }`
   - `TaskRefunded { task_id: String, customer: Pubkey, amount_refunded: u64, refunded_at: i64 }`
   - `Deposited { customer: Pubkey, amount: u64, new_balance: u64 }`
   - `Withdrawn { customer: Pubkey, amount: u64, new_balance: u64 }`
   - `AgentPackageRegistered { package_id: String, author: Pubkey, split_bps: u16 }`
3. Add `emit!()` calls in the appropriate instruction handlers:
   - TaskSettled → settle_task handler (all three cases)
   - TaskRefunded → refund_task handler
   - Deposited → deposit handler (Task 1's instruction — add emit if not already present)
   - Withdrawn → withdraw handler (Task 1's instruction — add emit if not already present)
   - AgentPackageRegistered → register_agent_package handler (Task 1's instruction — add emit if not already present)
4. Export events module from lib.rs.
5. Verify events appear in the generated IDL.
</implementation_plan>

<validation>
Run `anchor build` — compiles. Verify the generated IDL (target/idl/cto_billing.json) contains all 5 event definitions with correct field names and types. Verify `emit!()` calls are present in each corresponding instruction handler.
</validation>