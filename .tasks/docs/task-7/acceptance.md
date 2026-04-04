## Acceptance Criteria

- [ ] 1. Next.js build completes without errors (`next build` exits 0). 2. Login page renders and submits credentials — successful auth sets httpOnly cookie (verify via browser dev tools or test utility). 3. Dashboard page fetches and renders a task table with at least 5 rows when API returns mock data. 4. Each task row displays: task ID, title, agent name, delegate_id (or 'pending'), status badge, and clickable Linear issue link. 5. Polling is active: mock API returns updated status after 5 seconds, and the dashboard reflects the change without manual refresh. 6. Error state: when API returns 500, an inline Alert banner appears (not a modal) with error message. 7. Lighthouse accessibility score >= 80 on the dashboard page (shadcn/ui + Radix should provide this baseline).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.