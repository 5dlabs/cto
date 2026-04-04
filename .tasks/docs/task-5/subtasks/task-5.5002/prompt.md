Implement subtask 5002: Implement Discord bridge notification integration with rich embeds

## Objective
Add Discord-specific notification methods to NotificationService: notifyPipelineStart, notifyPipelineComplete, and notifyPipelineError. Each method constructs a color-coded rich embed payload and POSTs it to DISCORD_BRIDGE_URL using the resilient HTTP client from subtask 5001.

## Steps
1. In `notification.service.ts` (or a dedicated `discord-notifier.ts` that composes the base service), implement:
   - `notifyDiscordPipelineStart(runId: string, prdTitle: string, taskCount: number)`: green embed (color 0x00FF00), title 'Pipeline Started', fields for Run ID, PRD title, task count.
   - `notifyDiscordPipelineComplete(runId: string, summary: {tasksCreated: number; agentsAssigned: string[]; prUrl?: string; warnings?: string[]})`: blue embed (color 0x0000FF), title 'Pipeline Complete', fields for task count, agents, PR URL link, warnings if any.
   - `notifyDiscordPipelineError(runId: string, stage: string, error: string)`: red embed (color 0xFF0000), title 'Pipeline Error', fields for run ID, stage name, error message.
2. Payload format: construct the payload matching discord-bridge-http's expected schema. If the bridge expects raw Discord webhook format, use `{embeds: [{title, color, fields: [{name, value}]}]}`. If simpler, adapt accordingly. Add a TODO/comment noting the format may need adjustment after bridge API discovery.
3. Call `postWithRetry(DISCORD_BRIDGE_URL, payload)` for each.
4. Export these methods for use by the lifecycle hook layer.

## Validation
Unit test: notifyDiscordPipelineStart builds a payload with green color, runId, prdTitle, and taskCount in fields, and calls postWithRetry with DISCORD_BRIDGE_URL. Unit test: notifyDiscordPipelineComplete includes PR URL and agent list in the payload. Unit test: notifyDiscordPipelineError builds a red embed with stage and error fields. Unit test: verify each method returns gracefully (no throw) even when postWithRetry returns {ok: false}.