## Acceptance Criteria

- [ ] Run `bun run demo` from the `cli/` directory against Solana devnet — the script completes all 10 steps without error and exits with code 0. Verify on-chain state after execution: (1) OperatorConfig PDA exists and has correct treasury. (2) AgentPackage PDA has package_id="rex-code-agent", split_bps=3000, task_count=2, success_count=1, total_earned=3_000_000 (3 USDC). (3) CustomerBalance PDA has balance=0, total_deposited=100_000_000, total_spent=10_000_000, task_count=2. (4) TaskReceipt for TASK-001: amount=10_000_000, author_earned=3_000_000, quality_met=true. (5) TaskReceipt for TASK-002: amount=0, author_earned=0, quality_met=false. (6) Both Arweave receipt URLs are accessible and SHA-256 hashes match on-chain receipt_hash values. Full script runtime completes within 120 seconds on devnet.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.