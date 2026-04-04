## Acceptance Criteria

- [ ] 1. Unit test: With a mocked Hermes API returning valid content, `fetchResearchMemo()` returns a ResearchMemo with non-empty content, source, and valid timestamp. 2. Unit test: With NOUS_API_KEY unset, `fetchResearchMemo()` returns null and logs 'Hermes integration skipped'. 3. Unit test: With a mocked Hermes API that times out after 30s, `fetchResearchMemo()` returns null and logs a warning without throwing. 4. Unit test: With a mocked Hermes API returning 500, `fetchResearchMemo()` returns null and logs the error status. 5. Integration test: Run the deliberation pipeline with NOUS_API_KEY set and a mocked Hermes API; verify at least one task in the output has a non-null `research_memo` with all three fields populated.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.