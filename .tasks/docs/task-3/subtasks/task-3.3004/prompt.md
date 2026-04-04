Implement subtask 3004: Integrate fetchResearchMemo into the deliberation pipeline stage

## Objective
Wire the hermes-research module into the existing deliberation pipeline so that fetchResearchMemo is called for each task after initial task context assembly, and the returned memo is stored on the task's research_memo field.

## Steps
1. Locate the deliberation pipeline stage in the PM server codebase where tasks are assembled with their context.
2. After task context assembly, iterate over the tasks and call `fetchResearchMemo(taskContext)` for each one.
3. Assign the result (ResearchMemo or null) to `task.research_memo`.
4. Decide on sequential vs. concurrent execution (consider using Promise.all or a concurrency-limited approach if there are many tasks).
5. Ensure the pipeline continues even if all research memo calls return null.
6. Verify that downstream pipeline stages (e.g., notification, PR creation) can access `research_memo` from the task objects.

## Validation
Integration test: Run the deliberation pipeline with NOUS_API_KEY set and a mocked Hermes API returning valid content. Verify that at least one task in the pipeline output has a non-null research_memo with all three fields (content, source, timestamp) populated. Also verify that when the API key is unset, the pipeline completes successfully with all research_memo fields set to null.