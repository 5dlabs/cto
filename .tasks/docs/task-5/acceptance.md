## Acceptance Criteria

- [ ] Run `cargo build --workspace` — controller compiles with new billing module, zero errors. Run `cargo test -p controller -- billing` — unit tests pass: compute_billable_amount returns 750000 (0.75 USDC in lamports) for a Growth-tier 1-CodeRun task; build_receipt_json produces valid JSON with required fields (task_id, customer_pubkey, billing_items, total_amount_usdc). Integration test against devnet (can be manual or CI): deploy the Anchor program, create a mock CodeRun that transitions to merged state, verify the controller submits a settle_task transaction, confirm the transaction signature is valid on Solana devnet explorer, and the CodeRun annotation `cto.dev/solana-tx` is set. With `solana_billing_enabled=false`, no Solana transactions are submitted (feature flag works).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.