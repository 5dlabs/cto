## Enable Discord and Linear Bridge Notifications (Nova - Bun/Elysia)

### Objective
Integrate pipeline event notifications with the existing in-cluster discord-bridge-http and linear-bridge services. Fire notifications for pipeline start, stage transitions, and completion events. Use HTTP calls to co-located bridge services with connection strings from the sigma-1-infra-endpoints ConfigMap.

### Ownership
- Agent: nova
- Stack: Bun/Elysia
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
Step-by-step implementation:

1. Create a `NotificationService` module in cto-pm with methods:
   - `notifyPipelineStart(runId, prdTitle, taskCount)`
   - `notifyPipelineComplete(runId, summary)` — summary includes: tasks created, agents assigned, PR URL, any warnings
   - `notifyPipelineError(runId, stage, error)` — for non-fatal warnings and fatal errors

2. Discord notification integration:
   - POST to `DISCORD_BRIDGE_URL` from ConfigMap (e.g., `http://discord-bridge-http.bots.svc.cluster.local`)
   - Discover the expected payload format by checking discord-bridge-http's API docs or source
   - Use rich embeds if supported: color-coded (green=start, blue=complete, red=error), with fields for run ID, task count, assignee summary
   - If bridge is unreachable, log warning and continue — notification failure must not block pipeline

3. Linear notification integration:
   - POST to `LINEAR_BRIDGE_URL` from ConfigMap
   - Discover the expected payload format from linear-bridge's API
   - Notifications should reference the Linear session/project where issues were created
   - Include a link to the PR if available (from Task 4 output)

4. Hook notifications into the pipeline lifecycle:
   - Pipeline start: fire `notifyPipelineStart` immediately after pipeline initialization
   - Pipeline complete: fire `notifyPipelineComplete` after all stages finish (issue creation, PR creation)
   - Pipeline error: fire `notifyPipelineError` on any stage failure (but don't prevent graceful degradation)

5. Add retry logic: 1 retry with 2-second backoff for bridge calls. After retry failure, log and move on.

6. Add structured logging for all notification events: bridge URL, payload size, response status, latency.

### Subtasks
- [ ] Implement base NotificationService module with resilient HTTP client and retry logic: Create the foundational NotificationService class/module that provides a resilient HTTP POST helper with 1-retry / 2-second-backoff semantics, structured logging for every call (bridge URL, payload size, response status, latency), and graceful degradation (never throws on notification failure). Read DISCORD_BRIDGE_URL and LINEAR_BRIDGE_URL from the sigma-1-infra-endpoints ConfigMap via envFrom.
- [ ] Implement Discord bridge notification integration with rich embeds: Add Discord-specific notification methods to NotificationService: notifyPipelineStart, notifyPipelineComplete, and notifyPipelineError. Each method constructs a color-coded rich embed payload and POSTs it to DISCORD_BRIDGE_URL using the resilient HTTP client from subtask 5001.
- [ ] Implement Linear bridge notification integration with session references: Add Linear-specific notification methods to NotificationService that POST to LINEAR_BRIDGE_URL. Include Linear session/project references and PR links in the payload.
- [ ] Wire notification methods into pipeline lifecycle hooks: Integrate the Discord and Linear notification methods into the pipeline execution flow so that notifications fire at pipeline start, completion, and error events. Ensure notification failures never block or abort the pipeline.