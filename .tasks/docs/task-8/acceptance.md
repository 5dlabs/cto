## Acceptance Criteria

- [ ] Run `anchor test` (which starts localnet, deploys, runs all test files) — all test suites pass: 01-operator (5 tests), 02-customer (8 tests), 03-agent-package (6 tests), 04-settlement (12 tests), 05-refund (4 tests), 06-daily-reset (2 tests), 07-e2e-demo-flow (1 comprehensive test). Total: minimum 38 test cases, all passing. Zero skipped tests. Test execution completes within 180 seconds on localnet. The E2E demo flow test (07) verifies all on-chain state matches expected values: customer final balance=0 after full withdraw, AgentPackage.task_count=2, AgentPackage.success_count=1, two TaskReceipt PDAs exist with correct quality_met flags. `scripts/run-tests.sh` exits with code 0.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.