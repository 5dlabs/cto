Implement subtask 6005: Implement pipeline summary header component

## Objective
Build a summary header component that displays aggregate pipeline statistics: total tasks, assigned count, unresolved count, and pipeline status.

## Steps
1. Create `components/pipeline/PipelineSummary.tsx` accepting `tasks: Task[]` and `pipelineStatus: string` props.
2. Compute aggregate counts: `totalTasks = tasks.length`, `assignedCount = tasks.filter(t => t.delegate_id !== null).length`, `unresolvedCount = tasks.filter(t => t.delegate_id === null).length`.
3. Render a header bar with: pipeline status badge (running=blue, completed=green, error=red), and text showing '{totalTasks} tasks, {assignedCount} assigned, {unresolvedCount} unresolved'.
4. Use shadcn/ui Badge for the pipeline status indicator.
5. Style with Tailwind for clear visual hierarchy — larger text for counts, status badge prominent.

## Validation
Component test: Given 5 tasks with 4 having delegate_id and 1 with null, the summary header displays '5 tasks, 4 assigned, 1 unresolved'. Pipeline status badge shows correct color for 'running' vs 'completed' states.