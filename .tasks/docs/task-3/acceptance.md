## Acceptance Criteria

- [ ] 1. Unit test: when Hermes endpoint is reachable and returns 200, HermesProvider is selected and research content is written to deliberation path. 2. Unit test: when Hermes is unreachable and NOUS_API_KEY is set, NousProvider is selected as fallback. 3. Unit test: when both Hermes is unreachable and NOUS_API_KEY is empty/missing, SkipProvider is selected — pipeline continues without error, and a warning log entry is emitted containing 'no research provider available'. 4. Unit test: HermesProvider timeout (simulate 30s+ response) triggers fallback to NousProvider. 5. Integration test: run deliberation path and verify a research memo file exists in the deliberation output directory — either with Hermes/NOUS content or with a skip notice. 6. Verify that the pipeline completes successfully even when both HERMES_URL is empty and NOUS_API_KEY is absent (pure skip scenario).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.