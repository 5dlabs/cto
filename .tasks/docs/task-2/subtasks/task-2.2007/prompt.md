<identity>
You are rex working on subtask 2007 of task 2.
</identity>

<context>
<scope>
Write Rust unit tests for all instruction paths, edge cases, helper functions, and verify the generated IDL contains all instruction definitions.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/tests/mod.rs` (or use `#[cfg(test)]` modules). 2. Write tests for initialize_operator: valid initialization, invalid treasury (zero address), invalid fee (> 10000), double initialization fails. 3. Write tests for create_customer_account: valid creation, invalid caps (per_task > per_day), duplicate creation fails. 4. Write tests for deposit: valid deposit increments balance, zero amount rejected, paused state rejected. 5. Write tests for withdraw: valid withdrawal decrements balance, insufficient balance rejected, zero amount rejected, paused state rejected. 6. Write tests for update_spending_caps: valid update, invalid caps rejected, wrong signer rejected. 7. Write tests for pause/unpause: authority can toggle, non-authority rejected. 8. Write tests for daily reset helper: reset triggers at correct slot boundary, no reset within same day, spending cap checks pass/fail correctly, overflow handling. 9. Run `anchor build` and verify `target/idl/cto_billing.json` exists and contains all 7 instruction definitions (initialize_operator, create_customer_account, deposit, withdraw, update_spending_caps, pause, unpause). 10. Verify IDL accounts section includes OperatorConfig and CustomerBalance.
</implementation_plan>

<validation>
Run `cargo test --manifest-path programs/cto-billing/Cargo.toml` — all unit tests pass with zero failures. Coverage includes happy paths and error paths for every instruction. `anchor build` succeeds with `#[deny(warnings)]`. IDL at `target/idl/cto_billing.json` contains 7 instructions, 2 account types, and the BillingError enum with all variants.
</validation>