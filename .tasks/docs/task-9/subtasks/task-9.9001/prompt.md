Implement subtask 9001: Implement PipelineStatus badge component with three states and error Alert

## Objective
Create the PipelineStatus component using shadcn/ui Badge that renders color-coded status indicators (blue=running, green=complete, red=error). When status is 'complete', render links to Linear session and GitHub PR. When status is 'error', render a shadcn/ui Alert component with role='alert' and the error message on a red background.

## Steps
1. Create `src/components/PipelineStatus.tsx`. Accept props: `status: 'running' | 'complete' | 'error'`, `errorMessage?: string`, `linearSessionUrl?: string`, `prUrl?: string`.
2. Use shadcn/ui `Badge` component with variant mapping: running → blue outline badge with text 'Running', complete → green badge with text 'Complete', error → red destructive badge with text 'Error'.
3. When status is 'complete', render below the badge: a link to the Linear session URL and a link to the GitHub PR URL. Include text indicating Discord/Linear notifications were sent.
4. When status is 'error', render a shadcn/ui `Alert` component with `variant='destructive'` containing the `errorMessage`. Ensure the Alert has `role='alert'` for accessibility.
5. Place the component export so it can be imported into the dashboard header layout alongside summary counts from Task 6.

## Validation
Component test: Render PipelineStatus with status='running' and assert a blue 'Running' badge is present. Render with status='complete' plus linearSessionUrl and prUrl and assert green badge plus both links are rendered. Render with status='error' and errorMessage and assert red Alert with role='alert' contains the error text.