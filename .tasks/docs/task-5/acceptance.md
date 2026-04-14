## Acceptance Criteria

- [ ] Run `cargo build --release --bin settlement-sidecar` — compiles with zero errors. Run `cargo test` in the sidecar crate: (1) PDA derivation unit tests — derive customer_balance, agent_package, task_receipt, and vault PDAs from known inputs, verify they match expected Pubkeys (cross-reference with TypeScript derivation from Task 3). (2) SettlementEvent deserialization from JSON — parse a known event payload, verify all fields. (3) Retry logic unit test — simulate 3 failures then success, verify exponential backoff delays. (4) DLQ routing — simulate max_retries exceeded, verify message is written to DLQ stream. (5) Integration test (requires local validator + Redis): push event → verify on-chain TaskReceipt created with matching task_id and amount. Integration test completes in under 30 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.