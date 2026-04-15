## Acceptance Criteria

- [ ] Run `anchor build` — program compiles cleanly with all new instructions. Run `anchor test` with integration tests covering: (1) Register agent package — PDA created with correct fields, package_id retrievable. (2) Update agent package — source_uri and split_bps change, only author can update. (3) **Case 1 settle_task** — no agent package: customer balance decremented by amount, treasury receives full amount, TaskReceipt written with author_earned=0. (4) **Case 2 settle_task** — agent package with quality_met=true and split_bps=3000: customer debited full amount, author ATA receives 30%, treasury receives 70%, AgentPackage.success_count incremented. (5) **Case 3 settle_task** — quality_met=false: customer balance unchanged, no transfers, TaskReceipt.amount=0, AgentPackage.task_count incremented but success_count unchanged. (6) settle_task exceeding per-task cap — fails with ExceedsPerTaskCap. (7) settle_task exceeding daily cap — fails with ExceedsDailyCap. (8) refund_task — TaskReceipt status changes to Refunded, customer balance restored. (9) Double-settle same task_id — fails (PDA already exists). (10) Refund non-settled task — fails. Verify all 10 instructions present in IDL.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.