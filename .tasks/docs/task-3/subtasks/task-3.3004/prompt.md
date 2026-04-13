<identity>
You are rex working on subtask 3004 of task 3.
</identity>

<context>
<scope>
Create the update_spending_caps instruction that allows a customer to modify their per-task and daily spending caps without resetting the current daily_spent counter.
</scope>
</context>

<implementation_plan>
Create `instructions/update_spending_caps.rs`. Define the `UpdateSpendingCaps` Anchor accounts struct with: `customer` (Signer), `customer_balance` (mut, has_one = customer). Handler args: `max_per_task: u64`, `max_per_day: u64`. Logic: (1) Validate `max_per_task > 0` → InvalidSpendingCap. (2) Validate `max_per_day > 0` → InvalidSpendingCap. (3) Validate `max_per_task <= max_per_day` → InvalidSpendingCap. (4) Set `customer_balance.max_per_task = max_per_task`. (5) Set `customer_balance.max_per_day = max_per_day`. (6) Explicitly do NOT modify daily_spent or daily_reset_slot — add a code comment explaining this design choice: cap changes take effect on next settlement but don't retroactively clear spending history. No pause check needed for this instruction (it doesn't move funds). Export from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles. IDL contains `update_spending_caps` instruction with `max_per_task` (u64) and `max_per_day` (u64) args. Code review: validation enforces max_per_task <= max_per_day. Code review: daily_spent is NOT modified. Code review: no pause check present (intentional — this is a non-financial operation).
</validation>