<identity>
You are rex working on subtask 2002 of task 2.
</identity>

<context>
<scope>
Create the constants module with PDA seeds and SLOTS_PER_DAY, and the error enum with all 12 CtoPayError variants.
</scope>
</context>

<implementation_plan>
1. Create `programs/cto-pay/src/constants.rs`:
   ```rust
   pub const SLOTS_PER_DAY: u64 = 216_000;
   pub const OPERATOR_CONFIG_SEED: &[u8] = b"operator_config";
   pub const CUSTOMER_BALANCE_SEED: &[u8] = b"customer_balance";
   pub const TASK_RECEIPT_SEED: &[u8] = b"task_receipt";
   pub const VAULT_SEED: &[u8] = b"vault";
   ```
   - Add doc comments explaining SLOTS_PER_DAY approximation (400ms/slot, ±10% variance).
2. Create `programs/cto-pay/src/errors.rs`:
   ```rust
   use anchor_lang::prelude::*;
   
   #[error_code]
   pub enum CtoPayError {
       #[msg("Program is paused")]
       ProgramPaused,
       #[msg("Insufficient balance")]
       InsufficientBalance,
       #[msg("Amount exceeds per-task spending cap")]
       SpendingCapPerTaskExceeded,
       #[msg("Amount exceeds daily spending cap")]
       SpendingCapDailyExceeded,
       #[msg("Unauthorized operator")]
       UnauthorizedOperator,
       #[msg("Unauthorized customer")]
       UnauthorizedCustomer,
       #[msg("Task already settled")]
       TaskAlreadySettled,
       #[msg("Task not in settled state for refund")]
       TaskNotSettled,
       #[msg("Arithmetic overflow")]
       ArithmeticOverflow,
       #[msg("Invalid mint")]
       InvalidMint,
       #[msg("Amount must be greater than zero")]
       ZeroAmount,
       #[msg("Invalid protocol fee basis points (must be <= 10000)")]
       InvalidFeeBps,
   }
   ```
3. Ensure both modules are importable and compile without warnings under `clippy::pedantic`.
</implementation_plan>

<validation>
Run `anchor build` — compiles cleanly. Verify IDL (`target/idl/cto_pay.json`) contains all 12 error codes with correct names and messages. Verify constants are accessible from instruction modules (confirmed in subtask 2003 build).
</validation>