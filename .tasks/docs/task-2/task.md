## Extend PM Server for Agent Delegation in Linear Issues (Nova - Bun/Elysia)

### Objective
Extend the existing PM server's task generation and Linear issue creation pipeline to resolve agent hints to Linear user IDs via resolve_agent_delegates() and set delegate_id on every created issue. This is the core delegation flow that the E2E validation is designed to prove. The task entity schema is extended with a delegate_id field, and fallback behavior is implemented for unmappable agents.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Locate the existing `resolve_agent_delegates()` function in the PM server codebase. Verify it accepts agent hints (e.g., 'bolt', 'nova', 'blaze') and returns Linear user IDs.
2. Extend the task entity/type definition to include `delegate_id: string | null` field.
3. In the task generation pipeline, after tasks are generated with agent hints, call `resolve_agent_delegates()` to batch-resolve all agent hints to Linear user IDs.
4. Populate `delegate_id` on each task object with the resolved Linear user ID.
5. In the Linear issue creation step, pass `assigneeId: task.delegate_id` when creating each issue via the Linear API.
6. Implement fallback behavior: if `resolve_agent_delegates()` cannot resolve an agent hint, log a warning with the agent hint and task ID, create the issue unassigned (delegate_id = null), and add an `agent:unresolved` label to the issue.
7. Add a summary log at the end of issue creation: `Created N issues, M assigned, K unresolved`.
8. Ensure the PM server reads secrets from the `sigma-1-secrets` Kubernetes secret and endpoints from `sigma-1-infra-endpoints` ConfigMap via `envFrom`.
9. Write unit tests for: resolve_agent_delegates mapping, fallback on unknown agent, delegate_id propagation to Linear API call.
10. Write an integration test that runs the full pipeline with a mock Linear API and verifies at least 5 issues are created with non-null assigneeId.

### Subtasks
- [ ] Extend task entity type definition with delegate_id field: Add the `delegate_id: string | null` field to the task entity/type definition used throughout the PM server's task generation and issue creation pipeline.
- [ ] Implement and verify resolve_agent_delegates() function: Locate or implement the resolve_agent_delegates() function that accepts an array of agent hint strings and returns a mapping of agent hints to Linear user IDs, with null for unresolvable agents.
- [ ] Integrate delegation into the task generation pipeline: Wire resolve_agent_delegates() into the task generation pipeline: after tasks are generated with agent hints, batch-resolve them and populate delegate_id on each task, then pass assigneeId to the Linear API issue creation calls.
- [ ] Implement fallback behavior for unresolvable agents and summary logging: Add fallback logic for when resolve_agent_delegates() returns null for an agent hint: log a warning, create the issue unassigned, add an agent:unresolved label, and emit a summary log line after all issues are created.
- [ ] Write unit tests for resolve_agent_delegates and delegate_id propagation: Create comprehensive unit tests covering resolve_agent_delegates mapping, fallback on unknown agents, and delegate_id propagation to the Linear API call layer.
- [ ] Write integration test with mock Linear API validating full delegation pipeline: Create an integration test that runs the full task generation and issue creation pipeline against a mock Linear API, verifying at least 5 issues are created with non-null assigneeId and the summary log is correct.