<identity>
You are rex working on subtask 2005 of task 2.
</identity>

<context>
<scope>
Implement instructions for customers to create their balance PDA and update their spending cap parameters.
</scope>
</context>

<implementation_plan>
1. Create `create_customer_account.rs`: Define `CreateCustomerAccount` accounts struct — customer (Signer, mut), customer_balance (init, PDA seeds [b"customer_balance", customer.key().as_ref()], payer = customer, space), system_program. Args: max_per_task (u64), max_per_day (u64). Handler validates max_per_task <= max_per_day (InvalidCaps), validates both > 0 (InvalidAmount). Sets customer = customer.key(), balance = 0, total_deposited = 0, total_spent = 0, task_count = 0, max_per_task, max_per_day, daily_spent = 0, daily_reset_slot = Clock::get()?.slot, created_at = Clock::get()?.unix_timestamp. 2. Create `update_spending_caps.rs`: Define `UpdateSpendingCaps` accounts struct — customer (Signer), customer_balance (mut, has_one = customer). Args: new_max_per_task, new_max_per_day. Validate new_max_per_task <= new_max_per_day. Update fields. 3. Register both instructions in the program module.
</implementation_plan>

<validation>
create_customer_account creates PDA with correct initial values (balance 0, caps as specified, daily_reset_slot = current slot). create_customer_account with max_per_task > max_per_day fails with InvalidCaps. update_spending_caps updates caps correctly. update_spending_caps by non-customer fails. PDA address is deterministic from customer pubkey.
</validation>