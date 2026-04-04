Implement subtask 7005: Build API client module with typed fetch helpers

## Objective
Create a shared API client module that handles fetching from the PM server endpoints, including typed responses, error handling, and base URL configuration from the `PM_SERVER_URL` environment variable. This module is used by both the dashboard and detail pages.

## Steps
1. Create `src/lib/api.ts` with typed fetch functions:
   a. `fetchPipelineStatus(): Promise<PipelineStatus>` — calls `GET /api/pipeline/status`.
   b. `fetchDelegationStatus(): Promise<DelegationTask[]>` — calls `GET /api/delegation/status`.
   c. `fetchValidationReport(runId: string): Promise<ValidationReport>` — calls `GET /api/validation/report/{run_id}`.
2. Define TypeScript interfaces in `src/types/api.ts`:
   - `PipelineStatus`: `{ stage: string, progress: number, started_at: string, updated_at: string, status: 'running' | 'complete' | 'failed' }`
   - `DelegationTask`: `{ task_id: number, title: string, agent: string, delegate_id: string | null, status: 'assigned' | 'pending' | 'failed', linear_issue_url: string | null }`
   - `ValidationReport`: `{ run_id: string, total_tasks: number, assigned: number, pending: number, warnings: string[], linear_session_url: string | null, pr_url: string | null }`
3. Each fetch function should:
   a. Use the server-side `PM_SERVER_URL` env var for Next.js API routes, or the client-side proxy routes.
   b. Include proper error handling: throw typed errors with message and status code.
   c. Forward the auth cookie (use `credentials: 'include'` for client-side calls, or forward cookie headers for server-side calls).
4. Create Next.js API proxy routes under `src/app/api/pipeline/status/route.ts`, `src/app/api/delegation/status/route.ts`, and `src/app/api/validation/report/[run_id]/route.ts` that forward requests to the PM server with the auth token.

## Validation
TypeScript compiles without errors. Each fetch function returns correctly typed data when the PM server responds with valid JSON. Each function throws a descriptive error when the server returns 4xx/5xx. Proxy API routes forward the auth cookie to the PM server.