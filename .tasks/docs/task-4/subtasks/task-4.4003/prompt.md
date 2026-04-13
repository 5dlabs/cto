<identity>
You are rex working on subtask 4003 of task 4.
</identity>

<context>
<scope>
Create the pause and unpause instructions that allow the operator authority to toggle the circuit breaker on OperatorConfig, gating settlement and deposits while preserving customer exit (withdraw) and refund capabilities.
</scope>
</context>

<implementation_plan>
Create `instructions/pause.rs`. Define the `Pause` Anchor accounts struct with: `operator` (Signer), `operator_config` (mut, has_one = authority — ensures only the operator authority can pause). Handler: set `operator_config.paused = true`. Create `instructions/unpause.rs`. Define the `Unpause` Anchor accounts struct with: `operator` (Signer), `operator_config` (mut, has_one = authority). Handler: set `operator_config.paused = false`. Both are minimal single-line mutations. Add doc comments explaining: (a) when paused, settle_task, deposit, and create_customer_account will fail with ProgramPaused; (b) withdraw and refund_task continue to work during pause as safety valves; (c) the operator can unpause to resume normal operations. Export both from `instructions/mod.rs`.
</implementation_plan>

<validation>
Verify `anchor build` compiles. IDL contains both `pause` and `unpause` instructions. Code review: both use `has_one = authority` constraint. Code review: pause sets `operator_config.paused = true`, unpause sets `false`. Code review: doc comments describe which instructions are affected by pause state.
</validation>