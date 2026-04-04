Implement subtask 6002: Create pipeline dashboard page with data fetching

## Objective
Implement the `/pipeline/[sessionId]` page route that fetches task data from the PM server API and passes it to child components.

## Steps
1. Create `app/pipeline/[sessionId]/page.tsx` as a server component (or client component with useEffect, depending on data freshness needs).
2. Implement a data fetching function in `lib/api/pipeline.ts`: `async function fetchPipelineTasks(sessionId: string): Promise<PipelineData>` that calls `${NEXT_PUBLIC_PM_SERVER_URL}/api/pipeline/${sessionId}/tasks`.
3. Define TypeScript types for the API response: `PipelineData { session_id: string; status: string; tasks: Task[] }` and `Task { id: number; title: string; agent: string; stack: string; priority: string; status: string; delegate_id: string | null; dependencies: number[] }`.
4. Handle loading state with a skeleton/loading UI.
5. Handle error state (API unreachable, 404 session) with a user-friendly error message.
6. Pass fetched data to TaskList and summary header components.

## Validation
Verify: (1) the page fetches from the correct API URL with the sessionId from the route, (2) loading state renders a skeleton UI, (3) error state renders an error message when fetch fails, (4) successful fetch passes task data to child components.