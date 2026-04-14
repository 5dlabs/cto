<identity>
You are rex, the Rust/Anchor/Solana implementation agent. You own task 1 end-to-end.
</identity>

<context>
<task_overview>
Task 1: Anchor Program Core Account Structures & Foundation Instructions (Rex - Rust/Anchor/Solana)
Create the Anchor workspace and implement all PDA account structures (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt) plus all non-settlement instructions. This task produces the program IDL consumed by every downstream TypeScript task.
Priority: high
Dependencies: None
</task_overview>
</context>

<implementation_plan>
1. Initialize Anchor workspace at `programs/cto-billing/` with `anchor init` (Anchor v0.30+). Set program ID placeholder in `lib.rs` and `Anchor.toml`.

2. Define account structures in `state/` modules:
   - `OperatorConfig` (PDA seed: `[b"operator_config"]`): authority (Pubkey), treasury (Pubkey), mint (Pubkey — mint-agnostic per D4), protocol_fee_bps (u16), paused (bool). Singleton PDA.
   - `CustomerBalance` (PDA seed: `[b"customer_balance", customer.key().as_ref()]`): customer (Pubkey), balance (u64), total_deposited (u64), total_spent (u64), task_count (u64), max_per_task (u64), max_per_day (u64), daily_spent (u64), daily_reset_day (u64 — renamed from daily_reset_slot per D9), created_at (i64).
   - `AgentPackage` (PDA seed: `[b"agent_package", sha256(package_id)[..32]]`): package_id (String, #[max_len(64)] per D8), author (Pubkey), split_bps (u16), source_uri (String, #[max_len(256)]), total_earned (u64), task_count (u64), success_count (u64), registered_at (i64), active (bool).
   - `TaskReceipt` (PDA seed: `[b"task_receipt", sha256(task_id)[..32]]`): task_id (String, #[max_len(64)]), customer (Pubkey), amount (u64), author_earned (u64), quality_met (bool), agent_package (Option<Pubkey>), receipt_hash ([u8; 32]), operator (Pubkey), settled_at (i64), status (TaskStatus enum: Settled/Refunded/Disputed).

3. Implement a `utils/` module with `hash_seed(input: &str) -> [u8; 32]` helper using SHA-256 for PDA derivation (D2).

4. Implement instructions:
   - `initialize_operator(authority, treasury, mint, protocol_fee_bps)` — Creates OperatorConfig. Validate signer is the initial authority. Store mint Pubkey for token validation.
   - `create_customer_account(max_per_task, max_per_day)` — Creates CustomerBalance PDA for signing customer. Initialize all counters to 0.
   - `register_agent_package(package_id, split_bps, source_uri)` — Permissionless. Creates AgentPackage PDA with signer as author. Validate split_bps ≤ 10000. **Also create/verify author's ATA for the configured mint exists (passed as account, validated but NOT created — per D11).** Store registered_at from Clock sysvar.
   - `update_agent_package(source_uri, split_bps, active)` — Author-only (verify signer == agent_package.author). Update mutable fields.
   - `deposit(amount)` — Transfer USDC from customer's ATA to program vault (PDA-controlled token account). Increment customer.balance and customer.total_deposited. Emit Deposited event.
   - `withdraw(amount)` — Transfer from vault to customer's ATA. Validate amount ≤ balance. Decrement balance. Emit Withdrawn event.
   - `update_spending_caps(max_per_task, max_per_day)` — Customer-only. Update cap fields.
   - `pause()` / `unpause()` — Operator-only (verify signer == operator_config.authority). Toggle paused flag.

5. Define program vault as a token account PDA (seed: `[b"vault", mint.key().as_ref()]`) with the program as authority.

6. Add comprehensive Anchor error enum: `InsufficientBalance`, `ExceedsPerTaskCap`, `ExceedsDailyCap`, `Unauthorized`, `ProgramPaused`, `InvalidSplitBps`, `DuplicateTaskId`, `InvalidMint`, `PackageInactive`, `InvalidAmount`.

7. Ensure `anchor build` produces `target/idl/cto_billing.json` and `target/types/cto_billing.ts` — these are consumed by Tasks 3, 5, 6, 7, 8.
</implementation_plan>

<acceptance_criteria>
Run `anchor build` — compiles with zero errors and zero warnings. Verify `target/idl/cto_billing.json` exists and contains instruction definitions for all 9 instructions (initialize_operator, create_customer_account, register_agent_package, update_agent_package, deposit, withdraw, update_spending_caps, pause, unpause). Verify `target/types/cto_billing.ts` is generated. Run `anchor test` with a minimal smoke test that: (1) initializes operator config and reads it back with correct authority/treasury/mint, (2) creates a customer account with caps and verifies fields, (3) registers an agent package and verifies author/split_bps/source_uri, (4) deposits 100 USDC and verifies customer balance == 100, (5) withdraws 50 USDC and verifies balance == 50, (6) pauses and verifies paused == true. All 6 smoke tests pass on local validator in under 30 seconds.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Initialize Anchor workspace and project configuration: Scaffold the Anchor workspace at programs/cto-billing/ with Anchor v0.30+, configure Cargo.toml dependencies, Anchor.toml settings, and lib.rs entry point with program ID placeholder and module declarations.
- Implement PDA account structures in state/ modules: Define all four PDA account structures (OperatorConfig, CustomerBalance, AgentPackage, TaskReceipt) with correct seeds, fields, space calculations, and the TaskStatus enum.
- Implement utils module (hash_seed helper) and program vault PDA definition: Create the utils/ module with the SHA-256 hash_seed helper for PDA derivation and define the program vault token account PDA.
- Implement error enum for all program errors: Define the comprehensive Anchor error enum covering all validation failure cases across all instructions.
- Implement initialization and account creation instructions: Implement the initialize_operator, create_customer_account, register_agent_package, and update_agent_package instructions with full validation and Anchor constraints.
- Implement token flow instructions (deposit, withdraw) with vault PDA: Implement the deposit and withdraw instructions with SPL token transfers between customer ATAs and the program vault PDA, including vault initialization logic.
- Implement admin and customer management instructions (update_spending_caps, pause, unpause): Implement the update_spending_caps (customer-only), pause (operator-only), and unpause (operator-only) instructions.
- Smoke test suite — 6 integration tests on local validator: Write and execute the 6 smoke tests as specified in the test strategy: initialize operator, create customer, register agent package, deposit, withdraw, and pause.
</subtasks>