## Acceptance Criteria

- [ ] 1. Component test: TaskCard with a non-null `research_memo` shows a 'Research' badge; clicking it reveals content, source, and timestamp. 2. Component test: TaskCard with `research_memo=null` does not render the Research badge or collapsible section. 3. Component test: Markdown content in research memo (e.g., headers, links, code blocks) renders correctly via react-markdown. 4. Component test: Timestamp '2024-01-15T10:30:00Z' renders as a human-readable relative time string. 5. Component test: Summary header with 3 of 5 tasks having memos displays '3 of 5 tasks have research memos'. 6. Accessibility test: Collapsible section toggles with Enter and Space keys; expanded state is announced via aria-expanded.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.