## Acceptance Criteria

- [ ] Run `anchor build` — program compiles with zero warnings (deny warnings). Run `cargo test --manifest-path programs/cto-billing/Cargo.toml` — all Rust unit tests pass. Run `anchor test` with TypeScript integration tests that: (1) Initialize operator — OperatorConfig PDA exists with correct authority and treasury. (2) Create customer account — CustomerBalance PDA exists with correct caps. (3) Deposit 1000 USDC — customer balance reads 1000, vault token balance reads 1000. (4) Withdraw 500 — customer balance reads 500, customer ATA increased by 500. (5) Withdraw 600 — fails with InsufficientBalance. (6) Pause — deposit fails with ProgramPaused. (7) Unpause — deposit succeeds again. (8) Update caps — new caps reflected in account. IDL file exists at `target/idl/cto_billing.json` and contains all 6 instruction definitions.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.