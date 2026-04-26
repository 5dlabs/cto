# Linear-CTO Integration Workflow

Current-state runbook for PM + Linear integration as implemented in `crates/pm` and ACP runtime foundations (`crates/acp-runtime`).

## Scope

This document covers:
- Linear webhook intake and routing in PM.
- Session tracking and state transitions.
- Intake and play workflow triggering paths.
- ACP runtime selection metadata now tracked for PM sessions.

This document does not define future orchestration design; it reflects shipped behavior.

## Architecture

| Component | Responsibility | Primary codepaths |
|---|---|---|
| PM HTTP server | Receives webhooks and callback traffic | `crates/pm/src/bin/pm-server.rs`, `crates/pm/src/server.rs` |
| Webhook validation/routing | Signature verification, timestamp checks, event dispatch | `crates/pm/src/webhooks.rs`, `crates/pm/src/server.rs` |
| Agent session handlers | Handle `created` and `prompted` session events | `crates/pm/src/handlers/agent_session.rs` |
| Intake/play handlers | Submit intake CodeRuns and play workflows | `crates/pm/src/handlers/intake.rs`, `crates/pm/src/handlers/play.rs` |
| Session tracker | Tracks session/workflow/pod mappings and ACP metadata | `crates/pm/src/state/session_tracker.rs` |
| ACP registry + types | Runtime selection and common ACP state model | `crates/acp-runtime/src/registry.rs`, `crates/acp-runtime/src/types.rs` |

## Runtime Flow

### 1. Linear webhook arrives

`POST /webhooks/linear`:
- Rejects/ignores if `LINEAR_ENABLED` is false.
- Validates `linear-signature` against multi-app webhook secrets, with optional legacy fallback.
- Validates freshness via `webhook_timestamp` max age.
- Routes by event type/action.

### 2. `AgentSessionEvent: created`

`handle_agent_session_created`:
- Emits an initial thought to satisfy Linear interaction expectations.
- Moves issue to first workflow state of type `started` (lowest position).
- Registers session in in-memory `SessionTracker`.
- If ACP is enabled for PM via config, resolves and stores selected ACP runtime ID in session metadata.

Special-case intake path:
- If agent is Morgan and issue has label `prd` or `task:intake`, PM triggers intake CodeRun submission and returns `intake_triggered`.

### 3. `AgentSessionEvent: prompted`

`handle_agent_session_prompted`:
- If stop signal is present, emits stop response and marks ACP run state `cancelled`.
- Otherwise forwards message to routing layer and marks ACP run state `running`.

Current limitation:
- Stop handling updates state and emits response, but actual sidecar/process interruption is still TODO in code.

## Session Tracking Contract

`SessionInfo` tracks:
- `session_id`, `agent_name`, issue IDs.
- Optional `workflow_name`, `pod_name`, `pod_ip`.
- Status (`Pending`, `Running`, `Completed`, `Failed`, `TimedOut`).
- `acp` metadata:
  - `runtimeId`
  - `sessionId`
  - `runState` (`pending`, `initialized`, `running`, `waiting_for_permission`, `completed`, `failed`, `cancelled`)
  - `lastEventCursor`

Notes:
- Tracker state is in-memory only.
- Default timeout is 1 hour unless overridden in `SessionTracker::with_timeout`.

## ACP Runtime Configuration

ACP defaults are defined in config types and template:
- `defaults.acp.enabled`
- `defaults.acp.defaultRuntime`
- `defaults.acp.runtimes`
- `defaults.acp.services.pm`

Default runtime entry is `stakpak` (`command: "stakpak"`, `args: ["acp"]` in template).

Important implementation detail:
- PM currently records ACP runtime selection in session metadata.
- PM does not yet execute runtime prompts through `acp-runtime::run_oneshot_prompt`; that utility exists for runtime clients but is not wired into PM session execution flow.

## PM HTTP Interface (Operational)

Primary endpoints:
- `POST /webhooks/linear`: Linear webhook ingress.
- `POST /webhooks/github/events` and `POST /github/webhook`: GitHub event ingress.
- `POST /api/intake/setup`: Create project + PRD setup artifacts.
- `POST /trigger/intake`: Manual intake trigger.
- `POST /api/sessions/{session_id}/input`: Route message to running agent session(s).
- `GET /health`: Liveness (`{"status":"healthy"}`).
- `GET /ready`: Readiness (`503` when `LINEAR_ENABLED` is false).
- `GET /health/tokens`: Per-agent token installation/expiry health.

## Required Environment Inputs

Core:
- `LINEAR_ENABLED=true`
- `NAMESPACE` (defaults to `cto` when unset)

Multi-agent OAuth (per agent, examples):
- `LINEAR_APP_MORGAN_CLIENT_ID`
- `LINEAR_APP_MORGAN_CLIENT_SECRET`
- `LINEAR_APP_MORGAN_WEBHOOK_SECRET`
- `LINEAR_APP_MORGAN_ACCESS_TOKEN` (set after OAuth install)
- `LINEAR_APP_MORGAN_REFRESH_TOKEN` and `LINEAR_APP_MORGAN_EXPIRES_AT` (optional but recommended)

Optional compatibility:
- Legacy single-app webhook secret and token env vars are still supported as fallback paths.

## Troubleshooting

1. Webhooks ignored:
- Check `LINEAR_ENABLED`.
- Check PM logs for signature/timestamp failures.

2. Session created but no workflow activity:
- Verify issue transition to a `started` state succeeded.
- Check `SessionTracker` logs for registration and workflow mapping.

3. Prompted messages not reaching agents:
- Test `POST /api/sessions/{session_id}/input`.
- Verify pod/service discovery and routing layer logs.

4. Stop request appears successful but work continues:
- Expected with current implementation; state is updated to `cancelled`, but hard stop of agent process is not yet wired.

5. Token issues:
- Use `GET /health/tokens` to identify `not_installed`, `expiring`, or `expired` agent apps.
