Implement subtask 6003: Implement TaskCard component with agent avatar and color-coded badges

## Objective
Build the TaskCard component using shadcn/ui Card, Badge, and Avatar primitives to display task metadata with agent assignment visualization and color-coded status indicators.

## Steps
1. Create `components/pipeline/TaskCard.tsx` accepting a `Task` prop.
2. Use shadcn/ui `Card` as the outer container with `CardHeader` and `CardContent`.
3. Display task title in `CardHeader`, and in `CardContent` show: agent name, stack, priority, status, and dependency IDs.
4. Agent visualization: use shadcn/ui `Avatar` with a colored background based on agent name (e.g., bolt=blue, nova=purple, rex=orange, grizz=green, blaze=red, cipher=gray). Display agent name initial in the avatar.
5. Assignment status badge using shadcn/ui `Badge`: if `delegate_id` is non-null, render a green badge with the agent name (e.g., 'Nova'). If `delegate_id` is null, render an amber badge with 'Unresolved'. If status is 'pending', render a gray badge with 'Pending'.
6. Show priority as a small secondary badge (high=red, medium=amber, low=gray).
7. Ensure all text has sufficient contrast (WCAG AA 4.5:1 ratio).

## Validation
1. Component test: TaskCard with delegate_id='user_123' and agent='nova' displays 'Nova' badge in green variant. 2. Component test: TaskCard with delegate_id=null displays 'Unresolved' badge in amber variant. 3. Component test: TaskCard with status='pending' and no delegate_id displays gray 'Pending' badge. 4. Snapshot test: TaskCard renders all metadata fields (title, agent, stack, priority, status).