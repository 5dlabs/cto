<identity>
You are rex working on subtask 3005 of task 3.
</identity>

<context>
<scope>
Wire up all four customer-facing instructions into the program's lib.rs module, ensure the module exports are correct, run anchor build, and verify the IDL contains all new instructions with correct args and account structures.
</scope>
</context>

<implementation_plan>
In `lib.rs`, add the four public handler functions to the `#[program]` module: `create_customer_account`, `deposit`, `withdraw`, `update_spending_caps`. Each function should delegate to the corresponding instruction handler, passing through the context and args. Ensure `instructions/mod.rs` re-exports all four instruction modules (CreateCustomerAccount, Deposit, Withdraw, UpdateSpendingCaps account structs). Run `anchor build` and verify zero warnings. Inspect `target/idl/cto_pay.json` for: (a) `create_customer_account` with max_per_task, max_per_day args; (b) `deposit` with amount arg; (c) `withdraw` with amount arg; (d) `update_spending_caps` with max_per_task, max_per_day args. Verify all account constraints are reflected in the IDL (has_one, seeds, mut annotations). Ensure the instruction count now includes initialize_operator from Task 2 plus these 4.
</implementation_plan>

<validation>
Run `anchor build` — zero warnings, zero errors. Parse `target/idl/cto_pay.json` and confirm exactly 5 instructions exist (initialize_operator + 4 new). Verify each instruction's args array matches the specification. Verify PDA seeds for customer_balance appear in account definitions. Verify token_program account is present in deposit and withdraw instructions.
</validation>