## Acceptance Criteria

- [ ] Run `anchor build` — zero errors. Write and run targeted unit-level tests for the settle_task instruction covering all three cases:
- [ ] (1) Settle with no agent package: deposit 100 USDC, settle task for 10 USDC, verify customer balance is 90, treasury received 10, TaskReceipt shows amount=10, author_earned=0.
- [ ] (2) Settle with agent package, quality_met=true, split_bps=3000 (30%): deposit 100, settle for 10, verify author_ata received 3 USDC, treasury received 7 USDC, customer balance is 90, AgentPackage.total_earned=3, success_count=1.
- [ ] (3) Settle with agent package, quality_met=false: deposit 100, settle, verify customer balance remains 100, no transfers, TaskReceipt.amount=0, AgentPackage.task_count incremented but success_count unchanged.
- [ ] (4) Refund: settle a task, then refund it, verify customer balance restored, TaskReceipt.status=Refunded.
- [ ] (5) Cap enforcement: set max_per_task=5, attempt settle for 10 → ExceedsPerTaskCap error.
- [ ] (6) Daily cap: set max_per_day=15, settle 10, then settle 10 → ExceedsDailyCap error.
- [ ] (7) Paused: pause program, attempt settle → ProgramPaused error.
- [ ] (8) Duplicate task_id: settle task 'CTO-001', attempt second settle with same task_id → error (PDA already exists).
- [ ] All 8 tests pass on local validator within 30 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.