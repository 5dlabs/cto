<identity>
You are rex working on subtask 3001 of task 3.
</identity>

<context>
<scope>
Create the create_customer_account instruction that initializes a CustomerBalance PDA for a new customer with spending caps, pause validation, and Clock-based timestamps.
</scope>
</context>

<implementation_plan>
Create `instructions/create_customer_account.rs`. Define the `CreateCustomerAccount` Anchor accounts struct with: `customer` (Signer, mut — pays rent), `customer_balance` (init, PDA seeded by `[CUSTOMER_BALANCE_SEED, customer.key().as_ref()]`, space = 8 + CustomerBalance size), `operator_config` (Account read-only), `system_program`. Define the handler function with args `max_per_task: u64` and `max_per_day: u64`. Validation: require `max_per_task > 0` (CtoPayError::InvalidSpendingCap), `max_per_day > 0`, and `max_per_task <= max_per_day`. Check `!operator_config.paused` or error ProgramPaused. Use `Clock::get()?` to obtain `unix_timestamp` for `created_at` and `slot` for `daily_reset_slot`. Initialize all CustomerBalance fields: customer pubkey, balance=0, total_deposited=0, total_spent=0, task_count=0, the two caps, daily_spent=0, daily_reset_slot, created_at, bump from ctx.bumps. All field assignments must use the Anchor init pattern. Export the module from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles with zero warnings. Inspect the generated IDL for a `create_customer_account` instruction with `max_per_task` (u64) and `max_per_day` (u64) args. Code review: PDA seed is `[CUSTOMER_BALANCE_SEED, customer.key().as_ref()]`. Code review: pause check is present. Code review: cap validation enforces max_per_task <= max_per_day.
</validation>