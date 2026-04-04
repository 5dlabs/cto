Implement subtask 2003: Integrate delegation into the task generation pipeline

## Objective
Wire resolve_agent_delegates() into the task generation pipeline: after tasks are generated with agent hints, batch-resolve them and populate delegate_id on each task, then pass assigneeId to the Linear API issue creation calls.

## Steps
1. Locate the task generation pipeline code where tasks are created with agent hints.
2. After the task generation step completes, collect all unique agent hints from the generated tasks.
3. Call `resolve_agent_delegates(uniqueAgentHints)` to get the mapping.
4. Iterate over all generated tasks and set `task.delegate_id = mapping[task.agent] ?? null` for each.
5. Locate the Linear issue creation step in the pipeline.
6. When creating each Linear issue, pass `assigneeId: task.delegate_id` if `delegate_id` is non-null. If `delegate_id` is null, omit `assigneeId` from the Linear API call (or pass undefined).
7. Ensure the pipeline awaits the resolution before proceeding to issue creation.
8. Ensure the PM server reads `LINEAR_API_KEY` from `sigma-1-secrets` and endpoints from `sigma-1-infra-endpoints` ConfigMap via `envFrom` environment variables.

## Validation
Run the pipeline with a sample PRD containing at least 5 tasks with known agent hints. Verify each task object has a non-null `delegate_id` after resolution. Verify the Linear API create-issue calls include `assigneeId` matching the task's `delegate_id`. Trace logs show resolve_agent_delegates was called once with the batch of agent hints.