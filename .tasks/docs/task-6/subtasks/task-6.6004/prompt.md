Implement subtask 6004: Implement TaskList component with dependency-ordered rendering

## Objective
Build the TaskList component that topologically sorts tasks by their dependencies and renders TaskCards in correct order with visual dependency indicators.

## Steps
1. Create `components/pipeline/TaskList.tsx` accepting a `tasks: Task[]` prop.
2. Implement a topological sort utility in `lib/utils/topological-sort.ts` that orders tasks so no task appears before its dependencies. Handle circular dependency edge cases gracefully (log warning, render in original order).
3. Render sorted tasks as a vertical list of TaskCard components.
4. Add visual dependency indicators: indent tasks that have dependencies, or show a subtle left-border connector line using Tailwind CSS utilities.
5. Show dependency labels on each TaskCard: 'Depends on: Task 1, Task 3' as a small text below the card.
6. Use `key={task.id}` for React reconciliation.

## Validation
1. Component test: TaskList with 5 tasks renders them in dependency order — no task appears before its dependencies in the DOM. 2. Unit test: topological sort utility correctly orders tasks with diamond dependencies. 3. Component test: tasks with dependencies show indentation or visual connectors. 4. Edge case test: circular dependencies don't crash the component.