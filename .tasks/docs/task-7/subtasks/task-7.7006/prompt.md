Implement subtask 7006: Build dashboard page with pipeline status banner and task table

## Objective
Implement the main `/dashboard` page displaying a pipeline status banner at the top and a task table below. The banner shows pipeline stage, progress, and timestamps. The table renders delegation tasks with columns for Task ID, Title, Agent, Delegate ID, Status (color-coded badge), and Linear Issue Link.

## Steps
1. Create `src/app/dashboard/page.tsx` as a client component (`'use client'`).
2. Pipeline status banner:
   a. Use a shadcn/ui Card at the top of the page.
   b. Display: current stage, progress percentage, `started_at` and `updated_at` formatted timestamps.
   c. Status indicator: green for 'running', blue for 'complete', red for 'failed'.
3. Task table:
   a. Use shadcn/ui Table component.
   b. Columns: Task ID, Title, Agent, Delegate ID (show 'pending' in italic if null), Status, Linear Issue.
   c. Status badges using shadcn/ui Badge component:
     - `assigned` → green variant (or `default` with green class)
     - `pending` → yellow variant (or `secondary` with yellow class)
     - `failed` → `destructive` variant
   d. Linear Issue column: render as an external link (`<a>` with `target="_blank"` and `rel="noopener noreferrer"`), or 'N/A' if null.
   e. Make run_id in the table a clickable link to `/dashboard/runs/{run_id}` if applicable.
4. Loading state: show shadcn/ui Skeleton components while data is being fetched.
5. Use the typed API client from 7005 for data fetching.
6. Create a dashboard layout at `src/app/dashboard/layout.tsx` with a header showing 'Delegation Dashboard', the logout button, and a container wrapper.

## Validation
Dashboard page renders a pipeline status banner with stage, progress, and timestamps. Task table renders at least 5 rows with all specified columns. Status badges are color-coded correctly: green for assigned, yellow for pending, red for failed. Linear issue links open in new tabs. Skeleton loading states appear before data loads.