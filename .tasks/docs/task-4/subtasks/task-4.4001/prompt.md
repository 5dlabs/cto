Implement subtask 4001: Create the /dashboard/design-prs route and wire up navigation entry point

## Objective
Add a new Next.js page route for the design snapshot PR section and integrate a navigation link (sidebar item, tab, or breadcrumb) into the existing dashboard layout consistent with other sections.

## Steps
1. Examine the existing Next.js `app/` or `pages/` directory to identify the dashboard layout structure and routing convention (App Router vs Pages Router).
2. Create a new page file at the appropriate path (e.g., `app/dashboard/design-prs/page.tsx` or `pages/dashboard/design-prs.tsx`).
3. The page should accept a `pipelineRunId` query parameter from the URL (e.g., `/dashboard/design-prs?pipelineRunId=abc123`) using `useSearchParams` or the equivalent router hook.
4. Add a navigation entry (sidebar link or tab) in the shared dashboard layout component, matching the icon/label style of sibling navigation items.
5. Export a placeholder layout that renders a heading and will host the list and detail components built in subsequent subtasks.
6. Ensure the route is protected by any existing auth guards applied to other dashboard routes.

## Validation
Navigating to `/dashboard/design-prs` renders the page without errors. The sidebar/tab navigation contains a 'Design PRs' (or equivalent) link that routes correctly. The `pipelineRunId` query param is readable from the page context.