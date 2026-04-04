## Enable Discord and Linear Bridge Notifications (Nova - Bun/Elysia)

### Objective
Implement the notification dispatch layer in the PM server that sends pipeline lifecycle events (start, complete, error) to the existing in-cluster discord-bridge-http and linear-bridge services. Per D3, notifications are dispatched via the existing bridge services. The API paradigm (HTTP vs NATS) is pending D2 resolution; this task implements HTTP POST dispatch as the recommended default, with the interface designed to swap to NATS if D2 resolves differently.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
1. Create an internal module `notification-dispatch` with interface: `async function notify(event: PipelineEvent): Promise<void>` where PipelineEvent has type `'pipeline.start' | 'pipeline.complete' | 'pipeline.error'` and payload fields.
2. Read bridge service URLs from `sigma-1-infra-endpoints` ConfigMap: `DISCORD_BRIDGE_URL` and `LINEAR_BRIDGE_URL`.
3. Implement HTTP POST dispatch (recommended per D2 escalation): POST to discord-bridge-http with payload `{ event, pipeline_id, status, task_count, assigned_count, pr_url, linear_session_url, timestamp }`. POST to linear-bridge with similar payload for Linear comment/update.
4. Design the `notify()` interface as a facade so the underlying transport (HTTP or NATS) can be swapped without changing callers. If D2 resolves to NATS, a NATS publisher implementation can replace the HTTP implementation behind the same interface.
5. Notification payload must include at minimum: pipeline status, link to Linear session, link to PR (from Task 4), and task count summary.
6. Error handling: if a bridge service is unreachable, log a warning with the service name and error, but do NOT fail the pipeline. Notifications are best-effort.
7. Integrate notification calls into the pipeline: call `notify('pipeline.start')` at pipeline initiation, `notify('pipeline.complete')` after all issues are created and PR is submitted, `notify('pipeline.error')` if the pipeline encounters a fatal error.
8. Write unit tests for: HTTP payload formatting, error handling on bridge unavailability, facade interface contract.
9. Write an integration test that verifies both Discord and Linear bridge services receive POST requests during a full pipeline run.

### Subtasks
- [ ] Define PipelineEvent types and notification-dispatch facade interface: Create the notification-dispatch module with the PipelineEvent type union ('pipeline.start' | 'pipeline.complete' | 'pipeline.error'), payload type definitions, and the `notify()` facade function signature. Design the transport abstraction layer (a NotificationTransport interface) so HTTP and NATS implementations can be swapped without changing callers.
- [ ] Implement HTTP POST transport for discord-bridge-http: Implement the NotificationTransport interface for HTTP POST dispatch to the discord-bridge-http service. Format the payload per the Discord bridge's expected schema and POST to the URL from the ConfigMap.
- [ ] Implement HTTP POST transport for linear-bridge: Implement the NotificationTransport interface for HTTP POST dispatch to the linear-bridge service. Format the payload per the Linear bridge's expected schema and POST to the URL from the ConfigMap.
- [ ] Implement best-effort error handling for notification dispatch: Wrap the transport send() calls with error handling that catches connection refused, timeouts, and HTTP error responses (4xx/5xx), logging warnings without throwing or failing the pipeline.
- [ ] Integrate notify() calls into the pipeline lifecycle: Wire the notify() function into the existing pipeline orchestration code, calling it at pipeline start, successful completion, and on fatal error.
- [ ] Write unit tests for notification-dispatch module: Write comprehensive unit tests covering payload formatting, facade contract, and error handling for the notification-dispatch module.