## Acceptance Criteria

- [ ] 1. Pipeline completes all 5 stages without throwing a fatal error — exit status is success. 2. Validation report shows `total_tasks >= 5`. 3. Validation report shows `assigned_tasks >= 5` (delegate_id is non-null for at least 5 tasks). 4. For each assigned task, query Linear API GET issue and confirm `assignee.id` matches the expected delegate_id — at least 5 issues pass this check. 5. No issues have ONLY the `agent:pending` label when their agent hint has a known mapping — verify by cross-referencing the agent mapping. 6. `GET /api/validation/report/{run_id}` returns 200 with valid JSON containing all required fields. 7. If research was available (Hermes or NOUS), `research_included` is true and deliberation output contains non-empty research memos.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.