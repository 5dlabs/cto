<identity>
You are rex working on subtask 1007 of task 1.
</identity>

<context>
<scope>
Implement the update_spending_caps (customer-only), pause (operator-only), and unpause (operator-only) instructions.
</scope>
</context>

<implementation_plan>
1. Create `instructions/update_spending_caps.rs`:
   - Define `UpdateSpendingCaps` accounts struct: customer (Signer), customer_balance (mut, PDA).
   - Constraint: customer.key() == customer_balance.customer → Unauthorized.
   - Args: max_per_task (u64), max_per_day (u64).
   - Handler: set customer_balance.max_per_task = max_per_task, customer_balance.max_per_day = max_per_day.

2. Create `instructions/pause.rs`:
   - Define `Pause` accounts struct: authority (Signer), operator_config (mut, PDA).
   - Constraint: authority.key() == operator_config.authority → Unauthorized.
   - Handler: set operator_config.paused = true.

3. Create `instructions/unpause.rs`:
   - Define `Unpause` accounts struct: authority (Signer), operator_config (mut, PDA).
   - Constraint: authority.key() == operator_config.authority → Unauthorized.
   - Handler: set operator_config.paused = false.

4. Register all three instructions in lib.rs `#[program]` module.
5. Export from `instructions/mod.rs`.
</implementation_plan>

<validation>
Run `anchor build` — all three instructions compile. Verify the IDL contains update_spending_caps, pause, and unpause. Verify authorization constraints use the correct account comparisons.
</validation>