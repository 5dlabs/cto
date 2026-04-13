<identity>
You are rex working on subtask 4004 of task 4.
</identity>

<context>
<scope>
Wire up settle_task, refund_task, pause, and unpause into lib.rs, bringing the total program instruction count to 9. Verify the full IDL is generated correctly with all instructions, account structures, and argument types.
</scope>
</context>

<implementation_plan>
In `lib.rs`, add four new public handler functions to the `#[program]` module: `settle_task(ctx, task_id_hash, amount, receipt_hash)`, `refund_task(ctx)`, `pause(ctx)`, `unpause(ctx)`. Each delegates to the corresponding instruction handler. Update `instructions/mod.rs` to re-export all new modules (SettleTask, RefundTask, Pause, Unpause account structs). Run `anchor build` and verify zero warnings. Inspect `target/idl/cto_pay.json` comprehensively: (a) exactly 9 instructions: initialize_operator, create_customer_account, deposit, withdraw, settle_task, refund_task, update_spending_caps, pause, unpause; (b) settle_task args: task_id_hash ([u8;32]), amount (u64), receipt_hash ([u8;32]); (c) all PDA seeds are correctly reflected; (d) has_one constraints are present on operator-gated instructions; (e) TaskReceipt and CustomerBalance account types appear in the IDL types section with all fields. Verify program binary: `ls -la target/deploy/cto_pay.so` — should be under 1.4MB.
</implementation_plan>

<validation>
Run `anchor build` — zero warnings, zero errors. Parse IDL JSON: exactly 9 instructions present with correct names. Verify settle_task args are [u8;32], u64, [u8;32]. Verify refund_task has no additional args. Verify pause/unpause have no additional args. Check binary size < 1.4MB. Confirm all account types (OperatorConfig, CustomerBalance, TaskReceipt, TaskStatus enum) appear in IDL types.
</validation>