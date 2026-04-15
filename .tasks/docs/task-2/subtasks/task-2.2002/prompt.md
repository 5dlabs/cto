<identity>
You are rex working on subtask 2002 of task 2.
</identity>

<context>
<scope>
Create the BillingError enum in programs/cto-billing/src/errors.rs with all error variants needed by instructions.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-billing/src/errors.rs`. 2. Define `#[error_code]` enum `BillingError` with variants: InsufficientBalance (msg: 'Insufficient balance for withdrawal'), ExceedsPerTaskCap (msg: 'Amount exceeds per-task spending cap'), ExceedsDailyCap (msg: 'Amount exceeds daily spending cap'), ProgramPaused (msg: 'Program is currently paused'), Unauthorized (msg: 'Unauthorized: signer is not the operator authority'), InvalidTreasury (msg: 'Treasury address cannot be the zero address'), InvalidAmount (msg: 'Amount must be greater than zero'), InvalidCaps (msg: 'max_per_task must be less than or equal to max_per_day'), InvalidProtocolFee (msg: 'Protocol fee basis points must be <= 10000'). 3. Export the error module from lib.rs. 4. Ensure error codes are stable and documented for client-side matching.
</implementation_plan>

<validation>
Module compiles without errors. All 9 error variants are defined with descriptive messages. Error enum is accessible from instruction modules. `anchor build` includes errors in the generated IDL.
</validation>