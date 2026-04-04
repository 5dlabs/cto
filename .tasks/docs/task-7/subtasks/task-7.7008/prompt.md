Implement subtask 7008: Build pipeline run detail view page

## Objective
Implement the `/dashboard/runs/[run_id]` page that displays a full validation report for a specific pipeline run, including total tasks, assigned vs pending counts, warnings list, and links to the Linear session and PR.

## Steps
1. Create `src/app/dashboard/runs/[run_id]/page.tsx`.
2. Read `run_id` from the route params.
3. Fetch validation report using `fetchValidationReport(run_id)` from the API client.
4. Display:
   a. Run ID as page heading.
   b. Summary card (shadcn/ui Card): total tasks count, assigned count, pending count.
   c. Progress indicator: `assigned / total` as a fraction and percentage.
   d. Warnings section: if warnings array is non-empty, render each warning as a shadcn/ui Alert (warning variant) in a list.
   e. Linear session URL: render as an external link button (shadcn/ui Button variant='outline') or 'Not available' if null.
   f. PR URL: render as an external link button or 'Not available' if null.
5. Add a 'Back to Dashboard' link at the top using `next/link`.
6. Loading state with Skeleton components.
7. Error state with inline Alert if the fetch fails (e.g., invalid run_id returns 404).

## Validation
Navigating to `/dashboard/runs/{valid_run_id}` renders the validation report with total tasks, assigned, pending counts. Warnings are displayed as individual alert items. Linear session and PR links are clickable and open in new tabs. Navigating to an invalid run_id shows an error Alert. 'Back to Dashboard' link navigates to `/dashboard`.