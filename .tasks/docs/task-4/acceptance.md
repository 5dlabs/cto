## Acceptance Criteria

- [ ] Run `cargo build --features solana-billing` in the controller crate — compiles with zero errors. Run `cargo test --features solana-billing` — unit tests pass for: (1) SettlementEvent serialization/deserialization round-trip, (2) ReceiptJson generation with correct SHA-256 hash computation (hash a known receipt, verify against precomputed hash), (3) Redis publisher correctly formats XADD command (mock Redis or use testcontainers with Redis), (4) Billing amount calculation from pod duration (120s) and tier (standard at $0.75/run) produces expected USDC lamport value. Run `cargo build` without the feature flag — compiles cleanly with no Solana-related code included (verify via binary size or grep).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.