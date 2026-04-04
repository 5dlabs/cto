Implement task 7: Implement Web Frontend for Delegation Status Dashboard (Blaze - React/Next.js)

## Goal
Build a minimal read-only web dashboard using Next.js and shadcn/ui that displays pipeline delegation status: task list with agent assignments, delegate_id, pipeline status, and Linear issue links. This is a stretch/deferred task — lower priority than core pipeline validation (Tasks 1–6, 8).

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: low
- Dependencies: 6

## Implementation Plan
Step-by-step implementation:

1. Scaffold a Next.js app (App Router) with:
   - Tailwind CSS configured
   - shadcn/ui components installed via CLI (consistent with organizational tweakcn patterns)
   - TypeScript enabled

2. Implement JWT-based authentication:
   - Login page with credentials form (shadcn/ui Input + Button)
   - JWT token stored in httpOnly cookie (NOT localStorage)
   - Next.js middleware to check auth on protected routes
   - Short-lived tokens (15-minute expiry) with refresh mechanism
   - Auth endpoint: `POST /api/auth/login` on the PM server (may need to be added to cto-pm)

3. Dashboard page (`/dashboard`):
   - Fetch from `GET /api/pipeline/status` and `GET /api/delegation/status` (from Task 2)
   - Display pipeline status banner at top: stage, progress, timestamps
   - Task table using shadcn/ui Table component:
     - Columns: Task ID, Title, Agent, Delegate ID, Status, Linear Issue Link
     - Color-coded status badges: assigned (green), pending (yellow), failed (red)
   - Inline error banner (shadcn/ui Alert) for API errors — not modal dialogs (per D7)

4. Polling implementation:
   - 5-second periodic refresh using `setInterval` or SWR with `refreshInterval: 5000`
   - Show last-updated timestamp
   - Disable polling when pipeline status is 'complete' or 'failed'

5. Pipeline run detail view (`/dashboard/runs/{run_id}`):
   - Fetch from `GET /api/validation/report/{run_id}` (from Task 6)
   - Display full validation report: total tasks, assigned vs pending, warnings
   - Link to Linear session URL
   - Link to PR URL

6. Styling:
   - Use shadcn/ui default theme (no custom design artifacts available — Stitch failed)
   - Responsive layout for desktop viewing (internal tool, no mobile requirement)
   - Prioritize functional correctness over visual polish

7. Containerize with Dockerfile for potential in-cluster deployment:
   - Multi-stage build: build Next.js → serve with Node.js
   - Expose on port 3000
   - Read API base URL from environment variable `PM_SERVER_URL`

## Acceptance Criteria
1. Next.js build completes without errors (`next build` exits 0). 2. Login page renders and submits credentials — successful auth sets httpOnly cookie (verify via browser dev tools or test utility). 3. Dashboard page fetches and renders a task table with at least 5 rows when API returns mock data. 4. Each task row displays: task ID, title, agent name, delegate_id (or 'pending'), status badge, and clickable Linear issue link. 5. Polling is active: mock API returns updated status after 5 seconds, and the dashboard reflects the change without manual refresh. 6. Error state: when API returns 500, an inline Alert banner appears (not a modal) with error message. 7. Lighthouse accessibility score >= 80 on the dashboard page (shadcn/ui + Radix should provide this baseline).

## Subtasks
- Scaffold Next.js App Router project with TypeScript and Tailwind CSS: Initialize a new Next.js application using the App Router with TypeScript enabled and Tailwind CSS configured. Set up the project structure with `src/app` directory layout, configure `tsconfig.json`, `tailwind.config.ts`, and `postcss.config.js`. Ensure `next build` passes cleanly with no errors.
- Install and configure shadcn/ui component library: Set up shadcn/ui via its CLI, initializing the component system with the default theme. Pre-install the specific shadcn/ui components needed across the dashboard: Table, Button, Input, Alert, Badge, Card, and Skeleton (for loading states).
- Implement login page with credentials form: Build the `/login` page with a username/password form using shadcn/ui Input and Button components. The form submits credentials to `POST /api/auth/login` (proxied through a Next.js API route to avoid CORS). On success, the API route sets an httpOnly cookie containing the JWT. On failure, display an inline error message.
- Implement Next.js auth middleware and token refresh: Create Next.js middleware that intercepts requests to protected routes (`/dashboard/*`) and verifies the JWT from the httpOnly cookie. If the token is expired or missing, redirect to `/login`. Implement a refresh mechanism via a Next.js API route that requests a new token before the current one expires.
- Build API client module with typed fetch helpers: Create a shared API client module that handles fetching from the PM server endpoints, including typed responses, error handling, and base URL configuration from the `PM_SERVER_URL` environment variable. This module is used by both the dashboard and detail pages.
- Build dashboard page with pipeline status banner and task table: Implement the main `/dashboard` page displaying a pipeline status banner at the top and a task table below. The banner shows pipeline stage, progress, and timestamps. The table renders delegation tasks with columns for Task ID, Title, Agent, Delegate ID, Status (color-coded badge), and Linear Issue Link.
- Implement SWR-based polling with 5-second refresh interval: Add automatic 5-second polling to the dashboard page using SWR's `refreshInterval` option. Display a 'Last updated' timestamp, and automatically disable polling when the pipeline status reaches 'complete' or 'failed'.
- Build pipeline run detail view page: Implement the `/dashboard/runs/[run_id]` page that displays a full validation report for a specific pipeline run, including total tasks, assigned vs pending counts, warnings list, and links to the Linear session and PR.
- Containerize Next.js app with multi-stage Dockerfile: Create a multi-stage Dockerfile that builds the Next.js application and serves it with a minimal Node.js runtime. Configure the container to read `PM_SERVER_URL` from an environment variable and expose port 3000.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.