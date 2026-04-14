<identity>
You are rex working on subtask 1004 of task 1.
</identity>

<context>
<scope>
Define the comprehensive Anchor error enum covering all validation failure cases across all instructions.
</scope>
</context>

<implementation_plan>
1. Create `errors.rs` with `#[error_code]` derive macro.
2. Define error variants with descriptive messages:
   - `InsufficientBalance` — "Customer balance insufficient for this operation"
   - `ExceedsPerTaskCap` — "Amount exceeds per-task spending cap"
   - `ExceedsDailyCap` — "Amount would exceed daily spending cap"
   - `Unauthorized` — "Signer is not authorized for this operation"
   - `ProgramPaused` — "Program is currently paused"
   - `InvalidSplitBps` — "Split basis points must be <= 10000"
   - `DuplicateTaskId` — "Task ID already exists"
   - `InvalidMint` — "Token mint does not match configured mint"
   - `PackageInactive` — "Agent package is not active"
   - `InvalidAmount` — "Amount must be greater than zero"
3. Export the error enum from lib.rs.
4. Ensure error codes are unique and match Anchor conventions.
</implementation_plan>

<validation>
Run `anchor build` — error enum compiles. Verify all 10 error variants are present in the generated IDL under the errors section. Verify each variant has a unique error code.
</validation>