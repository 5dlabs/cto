<identity>
You are rex working on subtask 3002 of task 3.
</identity>

<context>
<scope>
Add all new error variants needed by the marketplace instructions to the program's error enum, covering validation failures, state conflicts, and agent package errors.
</scope>
</context>

<implementation_plan>
1. In `programs/cto-billing/src/errors.rs` (or the existing error module), add the following error variants to the program's `#[error_code]` enum:
   - `TaskAlreadySettled` — msg: "Task has already been settled (PDA exists)"
   - `TaskNotSettled` — msg: "Task receipt is not in Settled status"
   - `InvalidAgentPackage` — msg: "Agent package account is invalid"
   - `AgentPackageInactive` — msg: "Agent package is not active"
   - `InvalidReceiptHash` — msg: "Receipt hash does not match expected value"
   - `PackageIdTooLong` — msg: "Package ID exceeds maximum length of 64 characters"
   - `SourceUriTooLong` — msg: "Source URI exceeds maximum length of 256 characters"
   - `InvalidSplitBps` — msg: "Split basis points must be between 0 and 10000"
2. Ensure error codes do not collide with existing error variants from Task 2's instruction set (check the starting offset).
3. Verify all variants are accessible from instruction modules by importing the error enum.
4. Run `anchor build` to confirm the error codes compile and appear in the IDL.
</implementation_plan>

<validation>
Run `anchor build` — all error variants compile. Inspect the generated IDL JSON and confirm all 8 new error variants are present with unique error codes. Write a trivial Rust test that constructs each error variant to ensure they are importable from the instructions module.
</validation>