## Acceptance Criteria

- [ ] Run `anchor test` (which starts solana-test-validator, deploys the program, and runs the Mocha suite). All test groups (12 groups, 30+ individual test cases) pass with zero failures. Total execution time is under 60 seconds. Coverage includes: all 11 instructions (initialize_operator, create_customer_account, register_agent_package, update_agent_package, deposit, withdraw, update_spending_caps, pause, unpause, settle_task, refund_task), all 3 settlement cases, all error codes (InsufficientBalance, ExceedsPerTaskCap, ExceedsDailyCap, Unauthorized, ProgramPaused, InvalidSplitBps, DuplicateTaskId, PackageInactive), and the refund lifecycle. Run `npm run test:ci` to verify the CI-compatible invocation also passes.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.