# Task 2: Anchor Program — Account Structures, Errors, Constants, and initialize_operator (Rex - Rust/Anchor 0.30+)

## Overview
Define all three PDA account structures (OperatorConfig, CustomerBalance, TaskReceipt), the program error enum, constants, and implement the initialize_operator instruction. This establishes the program's data model foundation that all subsequent instructions build upon.

## Implementation Details
1. **File structure** under `programs/cto-pay/src/`:
   ```
   lib.rs              ← declare_id!, program module, instruction imports
   state/
     mod.rs
     operator_config.rs
     customer_balance.rs
     task_receipt.rs
   instructions/
     mod.rs
     initialize_operator.rs
   errors.rs
   constants.rs
   ```

2. **Constants** (`constants.rs`):
   ```rust
   /// Approximate slots in one day at 400ms/slot. Used for daily spending cap reset.
   /// Actual wall-clock time varies ±10% due to slot rate fluctuation.
   pub const SLOTS_PER_DAY: u64 = 216_000;
   
   /// PDA seeds
   pub const OPERATOR_CONFIG_SEED: &[u8] = b"operator_config";
   pub const CUSTOMER_BALANCE_SEED: &[u8] = b"customer_balance";
   pub const TASK_RECEIPT_SEED: &[u8] = b"task_receipt";
   pub const VAULT_SEED: &[u8] = b"vault";
   ```

3. **OperatorConfig** (`state/operator_config.rs`):
   ```rust
   #[account]
   pub struct OperatorConfig {
       pub authority: Pubkey,          // operator wallet
       pub treasury: Pubkey,           // revenue wallet
       pub mint: Pubkey,               // USDC (or test SPL) mint — NEVER hardcoded
       pub protocol_fee_bps: u16,      // basis points (e.g., 500 = 5%)
       pub paused: bool,
       pub bump: u8,
   }
   ```
   - `INIT_SPACE` implementation: 32 + 32 + 32 + 2 + 1 + 1 = 100 bytes + 8 (discriminator).
   - PDA seeds: `[OPERATOR_CONFIG_SEED]`.

4. **CustomerBalance** (`state/customer_balance.rs`):
   ```rust
   #[account]
   pub struct CustomerBalance {
       pub customer: Pubkey,
       pub balance: u64,
       pub total_deposited: u64,
       pub total_spent: u64,
       pub task_count: u64,
       pub max_per_task: u64,
       pub max_per_day: u64,
       pub daily_spent: u64,
       pub daily_reset_slot: u64,
       pub created_at: i64,
       pub bump: u8,
   }
   ```
   - PDA seeds: `[CUSTOMER_BALANCE_SEED, customer.key().as_ref()]`.
   - INIT_SPACE: 32 + (8 * 8) + 8 + 1 = 105 bytes + 8 (discriminator).

5. **TaskReceipt** (`state/task_receipt.rs`):
   ```rust
   #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
   pub enum TaskStatus {
       Settled,
       Refunded,
       Disputed,  // reserved for future use
   }

   #[account]
   pub struct TaskReceipt {
       pub task_id_hash: [u8; 32],     // SHA-256 of task_id string — sole identifier
       pub customer: Pubkey,
       pub amount: u64,
       pub fee_amount: u64,            // protocol fee (informational for future splits)
       pub receipt_hash: [u8; 32],     // SHA-256 of off-chain receipt JSON
       pub operator: Pubkey,
       pub settled_at: i64,
       pub status: TaskStatus,
       pub bump: u8,
   }
   ```
   - PDA seeds: `[TASK_RECEIPT_SEED, &task_id_hash]`.
   - INIT_SPACE: 32 + 32 + 8 + 8 + 32 + 32 + 8 + 1 + 1 = 154 bytes + 8 (discriminator).
   - **Key decision (D10)**: `task_id_hash` is `[u8; 32]` SHA-256 — NOT a String. Human-readable task_id lives only in off-chain receipt JSON.

6. **Error enum** (`errors.rs`):
   ```rust
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

7. **initialize_operator instruction** (`instructions/initialize_operator.rs`):
   - Signer: `authority` (becomes the operator).
   - Accounts: `authority` (signer, mut), `operator_config` (init, PDA), `vault` (init, token account — PDA-derived ATA with program authority), `mint` (the SPL token mint — passed as parameter, stored in OperatorConfig), `treasury` (validated as token account with correct mint), `system_program`, `token_program`, `rent`.
   - Validation: `protocol_fee_bps <= 10_000`.
   - Logic: Initialize OperatorConfig with all fields. Create vault token account owned by program PDA.
   - **Key decision (D8)**: `mint` is a parameter, NOT hardcoded. Stored in `OperatorConfig.mint` and validated in every subsequent token instruction.
   - Vault PDA seeds: `[VAULT_SEED, operator_config.key().as_ref()]` with the program as authority.

8. **lib.rs**:
   ```rust
   use anchor_lang::prelude::*;
   
   declare_id!("<PROGRAM_ID>");
   
   pub mod constants;
   pub mod errors;
   pub mod instructions;
   pub mod state;
   
   #[program]
   pub mod cto_pay {
       use super::*;
       
       pub fn initialize_operator(
           ctx: Context<InitializeOperator>,
           protocol_fee_bps: u16,
       ) -> Result<()> {
           instructions::initialize_operator::handler(ctx, protocol_fee_bps)
       }
   }
   ```

9. **Clippy compliance**: Add `#![warn(clippy::pedantic)]` with appropriate allows (`module_name_repetitions`, `must_use_candidate`) matching CTO workspace conventions.

10. **No `init_if_needed`**: Security constraint — use only `init` for account creation.

11. All `u64` arithmetic must use `checked_add`, `checked_sub`, `checked_mul`, `checked_div` with `.ok_or(CtoPayError::ArithmeticOverflow)?` — zero raw arithmetic.

## Dependencies
Tasks: 1

## Subtasks
- **Implement state module — OperatorConfig, CustomerBalance, TaskReceipt structs with INIT_SPACE and TaskStatus enum**: Create the state/ module with all three PDA account structs, the TaskStatus enum, INIT_SPACE implementations, and unit tests verifying byte layout calculations.
- **Implement constants.rs and errors.rs modules**: Create the constants module with PDA seeds and SLOTS_PER_DAY, and the error enum with all 12 CtoPayError variants.
- **Implement initialize_operator instruction with vault PDA token account creation**: Create the initialize_operator instruction handler and its Accounts struct, including vault token account initialization via anchor_spl, protocol_fee_bps validation, and OperatorConfig PDA initialization.
- **Wire lib.rs entry point, configure clippy::pedantic, and verify full build + IDL output**: Complete lib.rs with declare_id!, all module imports, the program module entry point for initialize_operator, pedantic clippy configuration, and verify the full anchor build produces a correct IDL.