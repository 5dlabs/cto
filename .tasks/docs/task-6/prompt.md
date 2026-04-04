Implement task 6: Scaffold Web Frontend with Agent Assignment Visualization (Blaze - React/Next.js)

## Goal
Create or extend the web frontend application to display pipeline task lists with agent assignment visualization. Each task card shows the delegate_id as an agent avatar/badge, assignment status, and task metadata. Uses shadcn/ui (per D6) with Radix UI primitives for accessibility. Note: This task is contingent on D5 resolution — if D5 resolves to defer frontend tasks, this task should be skipped.

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: medium
- Dependencies: 2

## Implementation Plan
1. Initialize or extend the Next.js application in the project with shadcn/ui components (per D6, using the team's tweakcn configuration if accessible, otherwise shadcn/ui defaults).
2. Create a pipeline dashboard page at `/pipeline/[sessionId]` that fetches task data from the PM server API.
3. Implement a `TaskCard` component using shadcn/ui Card, Badge, and Avatar components: display task title, agent name, stack, priority, status, and dependencies. Show `delegate_id` as an agent avatar with the agent name (e.g., 'Bolt', 'Nova') as a badge. Use color-coded badges: green for assigned, amber for unresolved (`agent:unresolved`), gray for pending.
4. Implement a `TaskList` component that renders all tasks in dependency order with a visual dependency graph (simple indentation or connecting lines).
5. Add a summary header showing: total tasks, assigned count, unresolved count, pipeline status.
6. Use `envFrom` to read the PM server API URL from `sigma-1-infra-endpoints` ConfigMap.
7. Ensure all components meet WCAG 2.1 AA accessibility standards (Radix primitives handle focus management, keyboard navigation, and ARIA attributes).
8. No authentication implementation — this is deferred per D7 recommendation (Cloudflare Access handles auth at ingress layer).
9. Write component tests for: TaskCard rendering with assigned agent, TaskCard rendering with unresolved agent, TaskList ordering by dependencies, summary header counts.

## Acceptance Criteria
1. Component test: TaskCard rendered with `delegate_id='user_123'` and `agent='nova'` displays 'Nova' badge in green and shows the avatar. 2. Component test: TaskCard rendered with `delegate_id=null` displays 'Unresolved' badge in amber. 3. Component test: TaskList with 5 tasks renders them in dependency order (no task appears before its dependencies). 4. Component test: Summary header shows correct counts — given 5 tasks with 4 assigned and 1 unresolved, displays '5 tasks, 4 assigned, 1 unresolved'. 5. Accessibility test: All interactive elements in TaskCard and TaskList are keyboard-navigable and have appropriate ARIA labels (tested via @testing-library/jest-dom axe integration).

## Subtasks
- Initialize Next.js application with shadcn/ui setup: Set up or extend the Next.js project with shadcn/ui component library, Tailwind CSS configuration, and project structure for the pipeline dashboard feature.
- Create pipeline dashboard page with data fetching: Implement the `/pipeline/[sessionId]` page route that fetches task data from the PM server API and passes it to child components.
- Implement TaskCard component with agent avatar and color-coded badges: Build the TaskCard component using shadcn/ui Card, Badge, and Avatar primitives to display task metadata with agent assignment visualization and color-coded status indicators.
- Implement TaskList component with dependency-ordered rendering: Build the TaskList component that topologically sorts tasks by their dependencies and renders TaskCards in correct order with visual dependency indicators.
- Implement pipeline summary header component: Build a summary header component that displays aggregate pipeline statistics: total tasks, assigned count, unresolved count, and pipeline status.
- Write component tests and accessibility tests: Write comprehensive component tests for TaskCard, TaskList, and PipelineSummary, plus accessibility tests verifying WCAG 2.1 AA compliance using axe-core.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.