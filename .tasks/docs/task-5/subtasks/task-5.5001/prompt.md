Implement subtask 5001: Define PipelineEvent types and notification-dispatch facade interface

## Objective
Create the notification-dispatch module with the PipelineEvent type union ('pipeline.start' | 'pipeline.complete' | 'pipeline.error'), payload type definitions, and the `notify()` facade function signature. Design the transport abstraction layer (a NotificationTransport interface) so HTTP and NATS implementations can be swapped without changing callers.

## Steps
1. Create `src/notification-dispatch/types.ts` with PipelineEvent type: `{ event: 'pipeline.start' | 'pipeline.complete' | 'pipeline.error'; pipeline_id: string; status: string; task_count: number; assigned_count: number; pr_url?: string; linear_session_url?: string; timestamp: string }`.
2. Define a `NotificationTransport` interface with method `send(target: 'discord' | 'linear', payload: PipelineEvent): Promise<void>`.
3. Create `src/notification-dispatch/index.ts` exporting `async function notify(event: PipelineEvent): Promise<void>` that delegates to the configured transport implementation for both Discord and Linear targets.
4. Use dependency injection or a factory pattern so the transport can be swapped at initialization time (e.g., `createNotifier(transport: NotificationTransport)`).
5. Read `DISCORD_BRIDGE_URL` and `LINEAR_BRIDGE_URL` from environment (sourced via `envFrom` on `sigma-1-infra-endpoints` ConfigMap) and pass them to the transport constructor.

## Validation
Verify that the notify() function accepts a PipelineEvent and delegates to the injected transport's send() method for both 'discord' and 'linear' targets. A mock transport should receive exactly 2 send() calls per notify() invocation.