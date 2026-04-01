## Acceptance Criteria

- [ ] 1. All 6 test cases pass in CI with real infrastructure. 2. Test Case 1: pipeline completes within 5-minute timeout with status 'completed'. 3. Test Case 2: >= 5 tasks with non-empty agent fields and >= 3 distinct agents. 4. Test Case 3: >= 5 Linear issues each with non-null assigneeId verified against Linear API. 5. Test Case 4: Hermes section present in research memo when NOUS_API_KEY is configured. 6. Test Case 5: PR exists in 5dlabs/sigma-1 with >= 5 scaffold files. 7. Test Case 6: Discord webhook collector received >= 2 messages with correct content. 8. Suite completes in under 10 minutes total.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.