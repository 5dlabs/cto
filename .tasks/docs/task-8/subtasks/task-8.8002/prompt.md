Implement subtask 8002: Integrate SnapshotPR into pipeline dashboard page with data fetching

## Objective
Place the SnapshotPR component in the pipeline dashboard page below the summary header and above the task list, fetching PR data from the PM server API endpoint.

## Steps
1. In the pipeline dashboard page component, identify the location between the summary header and the task list. 2. Fetch PR data from the PM server API endpoint (e.g., `/api/pipeline/snapshot-pr` or wherever the Task 4 PR result is exposed). Use the existing data fetching pattern from the dashboard (SWR, fetch, or server component data loading). 3. Pass the fetched prResult (or null if the endpoint returns no PR) to `<SnapshotPR prResult={data} />`. 4. Handle loading state with a subtle skeleton or nothing (component is small). 5. Handle fetch error gracefully: treat as null prResult with reason 'Failed to load PR data'.

## Validation
Integration test: mock the API endpoint to return a valid PR result; verify SnapshotPR component renders on the page between summary header and task list. Mock API returning null; verify 'No snapshot PR created' message appears. Mock API error; verify graceful fallback.